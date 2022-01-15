use crate::rest;
use crate::websocket::WebSocketShard;
use std::collections::HashMap;
use surf::http::convert::{Deserialize, Serialize};

pub struct Client {
  shards: HashMap<u32, WebSocketShard>,
  pub token: String
}

#[derive(Serialize, Deserialize)]
struct GatewayInformation {
  pub url: String,
  pub shards: i32,
}

impl Client {
  pub fn new(token: String) -> Client {
    Client {
      shards: HashMap::new(),
      token: token
    }
  }
  pub async fn connect(&mut self) -> Result<(), surf::Error> {
    let mut resp = rest::request(&self, rest::Method::GET, "gateway/bot", |_| ()).await.unwrap();
    let info = resp.body_json::<GatewayInformation>().await.unwrap();
    for i in 0..info.shards {
      self.shards.insert(i as u32, WebSocketShard::new(i as u32, info.url.clone()));
    }
    Ok(())
  }
  pub fn wait_for_shards(&mut self) -> Vec<Result<Option<websocket::CloseData>, Box<dyn std::any::Any + Send>>> {
    let mut results = Vec::new();
    for i in 0..self.shards.len() {
      results.push(self.shards.remove(&(i as u32)).unwrap().wait());
    }
    results
  }
}
