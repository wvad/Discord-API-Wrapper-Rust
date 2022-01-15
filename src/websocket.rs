use websocket::OwnedMessage;
use std::sync::mpsc;
use std::thread;

use crate::gatewaymessage::{GatewayMessage,GatewayMessageDecodeError};

pub struct WebSocketShard {
  pub handle: thread::JoinHandle<Option<websocket::CloseData>>,
  sender: mpsc::Sender<GatewayMessage>,
  id: u32
}

impl WebSocketShard {
  pub fn new(id: u32, url: String) -> WebSocketShard {
    let (outer_sender, inner_receiver) = mpsc::channel::<GatewayMessage>();
    WebSocketShard {
      handle: connect(url, inner_receiver),
      sender: outer_sender,
      id
    }
  }
  pub fn wait(self) -> Result<Option<websocket::CloseData>, Box<(dyn std::any::Any + Send + 'static)>> {
    self.handle.join()
  }
  pub fn send(&self, message: GatewayMessage) -> Result<(), mpsc::SendError<GatewayMessage>> {
    self.sender.send(message)
  }
}

fn add_query_params(url: &str) -> String {  
  let mut url = url::Url::parse(url).unwrap();
  url.query_pairs_mut().append_pair("q", "9");
  url.query_pairs_mut().append_pair("encoding", "etf");
  url.query_pairs_mut().append_pair("compress", "zlib-stream");
  url.as_str().to_string()
}

fn connect(
  url_str: String,
  receiver: mpsc::Receiver<GatewayMessage>,
) -> thread::JoinHandle<Option<websocket::CloseData>> {
  thread::spawn(move || {
    let mut client = websocket::ClientBuilder::new(&add_query_params(&url_str)).unwrap().connect_secure(None).unwrap();
    let mut cache = Vec::<u8>::new();
    loop {
      while let Ok(sending_message) = receiver.try_recv() {
        if let Ok(data) = sending_message.as_buffer() {
          client.send_message(&OwnedMessage::Binary(data)).unwrap();
        }
      }
      let maybe_message = client.recv_message();
      if let Err(_) = maybe_message { continue }
      let message = maybe_message.unwrap();
      match message {
        OwnedMessage::Binary(data) => {
          cache.extend(&data);
          if (data.len()) < 4 || (data[(data.len() - 4)..] != [0x00, 0x00, 0xff, 0xff]) { continue }
          let maybe_message = GatewayMessage::from_zlib(&cache);
          cache.clear();
          match maybe_message {
            Err(e) => {
              println!("Error on decoding message: {:?}", match e {
                GatewayMessageDecodeError::DecommpressError(e) => e,
                GatewayMessageDecodeError::DecodeError(e) => format!("{}", e),
                GatewayMessageDecodeError::ParseError(e) => e.to_string()
              });
              continue;
            },
            Ok(message) => {
              println!("{:?}", message);
            }
          }
        },
        OwnedMessage::Close(data) => {
          return data; 
        },
        _ => (),
      }
    }
  })
}