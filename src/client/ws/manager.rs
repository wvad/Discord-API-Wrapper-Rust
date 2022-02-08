use std::sync::{mpsc::{Sender, Receiver, channel}, Arc, Mutex, atomic::Ordering};
use std::thread;
use crate::{client::ws::{WebSocketShard, InflateError}, ClientData};
use crate::GatewayMessage;
use crate::rest::{APIRouter, SendResult};

type SharedPtr<T> = Arc<Mutex<T>>;

pub struct WebsocketManagerData {
  pub gateway: String,
  pub message_emitter: Sender<GatewayMessage>,
  pub error_emitter: Sender<InflateError>
}

#[derive(Debug)]
pub enum WSSendResult {
  Ok,
  ShardMissing
}

pub struct WebsocketManager {
  _data: SharedPtr<WebsocketManagerData>,
  _client_data: SharedPtr<ClientData>,
  _shards: std::collections::HashMap<u64, WebSocketShard>,
  _message_outer: Receiver<GatewayMessage>,
  _error_outer: Receiver<InflateError>
}

impl WebsocketManager {
  pub fn new(client_data: &SharedPtr<ClientData>) -> WebsocketManager {
    let (message_inner, message_outer) = channel::<GatewayMessage>();
    let (error_inner, error_outer) = channel::<InflateError>();
    WebsocketManager {
      _data: SharedPtr::new(Mutex::new(WebsocketManagerData {
        gateway: String::new(),
        message_emitter: message_inner,
        error_emitter: error_inner
      })),
      _client_data: client_data.clone(),
      _shards: std::collections::HashMap::new(),
      _message_outer: message_outer,
      _error_outer: error_outer
    }
  }
  pub async fn connect(&mut self) {
    let router = APIRouter::new(&self._client_data);
    let resp = router["gateway"]["bot"].get().send().await;
    let json = match resp {
      SendResult::JSON(json) => json,
      SendResult::Buffer(_) => {
        println!("[ERROR] The response is not JSON");
        return;
      },
      SendResult::Error(err) => {
        println!("[ERROR] An error occured while sending the request: {:?}", err);
        return;
      }
    };
    let url = match json["url"].as_str() {
      Some(v) => v,
      None => {
        println!("[ERROR] Information missing from the response");
        return;
      }
    };
    self._data.lock().unwrap().gateway = url.to_string();
    let shards = match json["shards"].as_i64() {
      Some(v) => v,
      None => {
        println!("[ERROR] Information missing from the response");
        return;
      }
    };
    for _ in 0..shards { self.add_shard(); }
  }
  pub fn get_messages(&self) -> Vec<GatewayMessage> {
    let mut messages = Vec::new();
    loop {
      match self._message_outer.try_recv() {
        Ok(msg) => messages.push(msg),
        Err(_) => break
      }
    }
    messages
  }
  pub fn add_shard(&mut self) {
    let shard_id = self._shards.len() as u64;
    let mut shard = WebSocketShard::new(&self._data, shard_id);
    shard.connect();
    self._shards.insert(shard_id, shard);
  }
  pub fn wait(&mut self) -> Vec<thread::Result<Option<websocket::CloseData>>> {
    let mut results = Vec::new();
    for i in self._shards.keys().cloned().collect::<Vec<u64>>() {
      if let Some(mut shard) = self._shards.remove(&i) {
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