use websocket::OwnedMessage;
use std::thread;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use crate::{ReceivedGatewayMessage, SendGatewayMessage, client::ws::{WebsocketManagerData, Inflate, OptionalResult}};
use websocket::sync::{stream::{TcpStream, TlsStream}, client::Client};

type SharedPtr<T> = Arc<Mutex<T>>;

#[derive(Debug)]
pub enum WebsocketStatus {
  Ready,
  Connecting,
  Reconnecting,
  Idle,
  Nearly,
  Disconnected,
  WaitingForGuilds,
  Identifying,
  Resuming
}

pub struct WebSocketShard {
  pub id: u64,
  pub data: SharedPtr<WebsocketManagerData>,
  pub connection: Option<SharedPtr<Client<TlsStream<TcpStream>>>>,
  pub inflate: SharedPtr<Inflate>,
  _handle: Option<thread::JoinHandle<Option<websocket::CloseData>>>,
  pub is_finished: Arc<AtomicBool>
}

impl WebSocketShard {
  pub fn new(manager_data_ptr: &SharedPtr<WebsocketManagerData>, shard_id: u64) -> WebSocketShard {
    WebSocketShard {
      id: shard_id,
      data: manager_data_ptr.clone(),
      connection: None,
      inflate: Arc::new(Mutex::new(Inflate::new())),
      _handle: None,
      is_finished: Arc::new(AtomicBool::new(false))
    }
  }
  pub fn connect(&mut self) {
    let manager_data = self.data.clone();
    let address = add_query_params(&manager_data.lock().unwrap().gateway);
    let mut builder = websocket::ClientBuilder::new(&address).unwrap();
    let ws_client = builder.connect_secure(None).unwrap();
    ws_client.set_nonblocking(true).unwrap();
    let client = Arc::new(Mutex::new(ws_client));
    self.connection = Some(client.clone());
    let inflate = self.inflate.clone();
    let shard_id = self.id;
    let is_finished = self.is_finished.clone();
    self._handle = Some(thread::spawn(move || {
      loop {
        let message = match client.lock().unwrap().recv_message() {
          Err(_) => continue,
          Ok(v) => v
        };
        match message {
          OwnedMessage::Binary(data) => {
            let term = match inflate.lock().unwrap().to_term(&data) {
              OptionalResult::Ok(v) => v,
              OptionalResult::Err(err) => {
                println!("[ERROR] Error on message decoding {:?}", err);
                manager_data.lock().unwrap().error_emitter.send(err).unwrap();
                continue;
              },
              OptionalResult::None => continue
            };
            let maybe_message = ReceivedGatewayMessage::from_term(shard_id, &term);
            match maybe_message {
              Err(err) => {
                println!("[ERROR] {:?}", err);
              },
              Ok(message) => {
                manager_data.lock().unwrap().message_emitter.send(message).unwrap();
              }
            }
          },
          OwnedMessage::Close(data) => {
            is_finished.store(true, Ordering::Relaxed);
            return data
          },
          _ => ()
        }
      }
    }));
  }
  pub fn wait(&mut self) -> thread::Result<Option<websocket::CloseData>> {
    match self._handle {
      None => return Ok(None),
      _ => ()
    }
    self._handle.take().unwrap().join()
  }
  pub fn send(&self, message: SendGatewayMessage) {
    let connection = match &self.connection {
      Some(connection) => connection,
      None => {
        println!("[ERROR] Shard {} is not connected", self.id);
        return
      }
    };
    match message.to_buffer() {
      Ok(data) => {
        let mut client = connection.lock().unwrap();
        client.send_message(&OwnedMessage::Binary(data)).unwrap();
      },
      Err(err) => {
        println!("[ERROR] {:?}", err);
      }
    }
  }
}

fn add_query_params(url: &str) -> String {  
  let mut url = url::Url::parse(url).unwrap();
  url.query_pairs_mut().append_pair("q", "9");
  url.query_pairs_mut().append_pair("encoding", "etf");
  url.query_pairs_mut().append_pair("compress", "zlib-stream");
  url.as_str().to_string()
}
