use crate::rest;
use crate::websocket::WebsocketManager;

pub struct Client {
  pub shards: WebsocketManager,
  pub token: String
}

#[derive(Debug)]
pub enum ClientConnectError {
  ReqwestError(reqwest::Error),
  InformationError(&'static str),
  None
}

impl Client {
  pub fn new(token: String) -> Client {
    Client {
      shards: WebsocketManager::new(),
      token
    }
  }
  pub fn api(&self) -> rest::Router {
    rest::Router::new(&self)
  }
  pub async fn connect(&mut self) -> ClientConnectError {
    let info = match self.api()["gateway"]["bot"].get().send().await {
      rest::SendResult::ReqwestError(e) => return ClientConnectError::ReqwestError(e),
      rest::SendResult::Buffer(_) => return ClientConnectError::InformationError("Response is not JSON"),
      rest::SendResult::JSON(v) => v,
    };
    let shards = match info["shards"].as_i64() {
      None => return ClientConnectError::InformationError("Information missing: shrads count"),
      Some(v) => v as u64
    };
    let url = match info["url"].as_str() {
      None => return ClientConnectError::InformationError("Information missing: gateway url"),
      Some(v) => v
    };
    for _ in 0..shards { self.shards.add_shard(url) }
    ClientConnectError::None
  }
}
