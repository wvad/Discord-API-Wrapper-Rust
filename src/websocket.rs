use websocket::OwnedMessage;
use std::sync::{mpsc, Arc, Mutex, atomic::{Ordering, AtomicBool}};
use std::thread;
use crate::gatewaymessage::{GatewayMessage,GatewayMessageDecodeError};
use inflate::InflateStream;

#[derive(Debug)]
pub enum WSSendResult {
  Ok,
  ShardMissing
}

pub struct WebsocketManager {
  _shards: std::collections::HashMap<u64, WebSocketShard>,
  _receiver: mpsc::Receiver<GatewayMessage>,
  _sender: Arc<Mutex<mpsc::Sender<GatewayMessage>>>
}

impl WebsocketManager {
  pub fn new() -> WebsocketManager {
    let (inner_sender, outer_receiver) = mpsc::channel::<GatewayMessage>();
    let inner_sender_ptr = Arc::new(Mutex::new(inner_sender));
    WebsocketManager {
      _shards: std::collections::HashMap::new(),
      _receiver: outer_receiver,
      _sender: inner_sender_ptr
    }
  }
  pub fn get_messages(&self) -> Vec<GatewayMessage> {
    let mut messages = Vec::new();
    loop {
      match self._receiver.try_recv() {
        Ok(msg) => messages.push(msg),
        Err(_) => break
      }
    }
    messages
  }
  pub fn add_shard(&mut self, url: &str) {
    let shard_id = self._shards.len() as u64;
    self._shards.insert(shard_id, create_shard(shard_id, url, self._sender.clone()));
  }
  pub fn wait(&mut self) -> Vec<thread::Result<Option<websocket::CloseData>>> {
    let mut results = Vec::new();
    for i in self._shards.keys().cloned().collect::<Vec<u64>>() {
      if let Some(shard) = self._shards.remove(&i) {
        results.push(shard.wait());
      }
    }
    results
  }
  pub fn is_finished(&self) -> bool {
    for shard in self._shards.values() {
      if !shard.is_finished.load(Ordering::Relaxed) {
        return false;
      }
    }
    true
  }
  pub fn send(&self, message: GatewayMessage) -> WSSendResult {
    if let Some(shard) = self._shards.get(&message.shard_id) {
      shard.send(message);
      WSSendResult::Ok
    } else {
      WSSendResult::ShardMissing
    }
  }
}

pub struct WebSocketShard {
  pub handle: thread::JoinHandle<Option<websocket::CloseData>>,
  _sender: Arc<Mutex<Vec<GatewayMessage>>>,
  pub is_finished: Arc<AtomicBool>
}

impl WebSocketShard {
  pub fn wait(self) -> thread::Result<Option<websocket::CloseData>> {
    self.handle.join()
  }
  pub fn send(&self, message: GatewayMessage) {
    self._sender.lock().unwrap().insert(0, message)
  }
}

fn create_shard(shard_id: u64, url: &str, received_msg_sender: Arc<Mutex<mpsc::Sender<GatewayMessage>>>) -> WebSocketShard {
  let outer_sender = Arc::new(Mutex::new(Vec::<GatewayMessage>::new()));
  let inner_receiver = outer_sender.clone();
  let is_finished = Arc::new(AtomicBool::new(false));
  let is_finished_ptr = is_finished.clone();
  let url_str = url.to_string();

  let handle = thread::spawn(move || {
    let address = add_query_params(&url_str);
    let mut builder = websocket::ClientBuilder::new(&address).unwrap();
    let mut client = builder.connect_secure(None).unwrap();
    let mut cache = Vec::<u8>::new();
    client.set_nonblocking(true).unwrap();
    let mut inflate = InflateStream::from_zlib();
    loop {
      loop {
        if let Some(sending_message) = inner_receiver.lock().unwrap().pop() {
          match sending_message.as_buffer() {
            Ok(data) => {
              client.send_message(&OwnedMessage::Binary(data)).unwrap();
            },
            Err(err) => {
              println!("[ERROR] Error on message encoding {:?}", err);
            }
          }
        } else {
          break;
        }
      }
      let message = match client.recv_message() {
        Err(_) => continue,
        Ok(v) => v
      };
      match message {
        OwnedMessage::Binary(data) => {
          cache.extend(&data);
          if (data.len()) < 4 || (data[(data.len() - 4)..] != [0x00, 0x00, 0xff, 0xff]) { continue }
          let maybe_message = GatewayMessage::from_zlib(shard_id, &mut inflate, &cache);
          cache.clear();
          match maybe_message {
            Err(e) => {
              match e {
                GatewayMessageDecodeError::DecommpressError(e) => {
                  println!("[ERROR] Error on decompressing message: {:?}" , e);
                },
                GatewayMessageDecodeError::DecodeError(e) => {
                  println!("[ERROR] Error on decoding message: {:?}" , e);
                },
                GatewayMessageDecodeError::ParseError(e) => {
                  println!("[ERROR] Error on parsing message: {:?}" , e);
                }
              }
            },
            Ok(message) => {
              received_msg_sender.lock().unwrap().send(message).unwrap();
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
  });

  WebSocketShard {
    handle,
    _sender: outer_sender,
    is_finished: is_finished_ptr
  }
}

fn add_query_params(url: &str) -> String {  
  let mut url = url::Url::parse(url).unwrap();
  url.query_pairs_mut().append_pair("q", "9");
  url.query_pairs_mut().append_pair("encoding", "etf");
  url.query_pairs_mut().append_pair("compress", "zlib-stream");
  url.as_str().to_string()
}
