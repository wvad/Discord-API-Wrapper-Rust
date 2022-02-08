use crate::client::ws::WebsocketManager;
use std::sync::{Arc, Mutex};

type SharedPtr<T> = Arc<Mutex<T>>;

pub struct ClientData {
  pub token: Option<String>
}

pub struct Client {
  pub shards: WebsocketManager,
  pub data: SharedPtr<ClientData>
}

#[derive(Debug)]
pub enum ClientConnectError {
  ReqwestError(reqwest::Error),
  InformationError(&'static str),
  None
}

impl Client {
  pub fn new() -> Client {
    let data = Arc::new(Mutex::new(ClientData {
      token: None
    }));
    Client {
      shards: WebsocketManager::new(&data),
      data
    }
  }
  pub async fn login<S>(&mut self, token: S) where S: std::string::ToString {
    self.data.lock().unwrap().token = Some(token.to_string());
    self.shards.connect().await;
  }
}
