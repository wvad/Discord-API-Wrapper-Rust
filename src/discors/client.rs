use crate::discors::websocket::WebSocketShard;
use surf::http::convert::{Deserialize, Serialize};

pub enum HTTPMethod {
  GET,
  POST,
  PATCH,
  DELETE,
  PUT,
}

pub struct Client {
  shards: Vec<WebSocketShard>,
  token: String
}

#[derive(Serialize, Deserialize)]
struct GatewayInformation {
  pub url: String,
  pub shards: i32,
}

impl Client {
  pub fn new(token: String) -> Client {
    Client {
      shards: Vec::new(),
      token: token
    }
  }
  pub async fn connect(&mut self) -> Result<(), surf::Error> {
    let mut resp = self.request(HTTPMethod::GET, "gateway/bot", |_| ()).await.unwrap();
    let GatewayInformation { url, shards } = resp.body_json().await.unwrap();
    for _ in 0..shards {
      self.shards.push(WebSocketShard::new(url.clone()));
    }
    Ok(())
  }
  pub fn wait_for_shards(&mut self) -> Vec<Result<Option<websocket::CloseData>, Box<dyn std::any::Any + Send>>> {
    let mut results = Vec::new();
    for shard in self.shards.splice(0..self.shards.len(), Vec::new()) {
      results.push(shard.wait());
    }
    results
  }
  pub async fn request<T>(
    &self,
    method: HTTPMethod,
    route: &str,
    builder: T
  ) -> surf::Result<surf::Response> where T: Fn(&mut surf::RequestBuilder) {
    let mut req_builder = match method {
      HTTPMethod::GET => surf::get,
      HTTPMethod::POST => surf::post,
      HTTPMethod::PATCH => surf::patch,
      HTTPMethod::DELETE => surf::delete,
      HTTPMethod::PUT => surf::put,
    }(format!("https://discord.com/api/v9/{}", route));
    builder(&mut req_builder);
    req_builder
    .header("Authorization", format!("Bot {}", self.token))
    .send()
    .await
  }
}
