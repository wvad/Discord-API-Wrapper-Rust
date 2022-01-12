use websocket::OwnedMessage;
use std::sync::mpsc;
use std::thread;
use eetf::{Term};
use inflate::inflate_bytes_zlib;

use crate::discors::gatewaymessage::GatewayMessage;

pub struct WebSocketShard {
  pub handle: thread::JoinHandle<Option<websocket::CloseData>>,
  sender: mpsc::Sender<GatewayMessage>,
}

impl WebSocketShard {
  pub fn new(url: String) -> WebSocketShard {
    let (outer_sender, inner_receiver) = mpsc::channel::<GatewayMessage>();
    WebSocketShard {
      handle: connect(url, inner_receiver),
      sender: outer_sender,
    }
  }
  pub fn send(&self, message: GatewayMessage) -> Result<(), mpsc::SendError<GatewayMessage>> {
    self.sender.send(message)
  }
}

fn connect(
  url_str: String,
  receiver: mpsc::Receiver<GatewayMessage>,
) -> thread::JoinHandle<Option<websocket::CloseData>> {
  thread::spawn(move || {
    let mut url = url::Url::parse(url_str.as_str()).unwrap();
    url.query_pairs_mut().append_pair("q", "9");
    url.query_pairs_mut().append_pair("encoding", "etf");
    url.query_pairs_mut().append_pair("compress", "zlib-stream");
    let mut client = websocket::ClientBuilder::new(url.as_str()).unwrap().connect_secure(None).unwrap();
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
          let maybe_decompressed = inflate_bytes_zlib(&cache);
          cache.clear();
          if let Err(_) = maybe_decompressed { continue }
          let maybe_decoded = Term::decode(&maybe_decompressed.unwrap()[..]);
          if let Err(_) = maybe_decoded { continue }
          let message = GatewayMessage::from_term(maybe_decoded.unwrap());
          println!("{}", message);
        },
        OwnedMessage::Close(data) => {
          return data; 
        },
        _ => (),
      }
    }
  })
}