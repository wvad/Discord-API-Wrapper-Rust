use crate::client::Client;
use std::sync::Mutex;

pub struct Request {
  _req_builder: reqwest::RequestBuilder
}

#[derive(Debug)]
pub enum SendResult {
  ReqwestError(reqwest::Error),
  JSON(serde_json::Value),
  Buffer(Vec<u8>)
}

impl SendResult {
  pub fn as_json(&self) -> &serde_json::Value {
    match self {
      SendResult::JSON(v) => v,
      _ => panic!("SendResult is not JSON")
    }
  }
}

impl Request {
  pub fn edit<F>(&mut self, f: F) -> &mut Self where F: FnOnce(&mut reqwest::RequestBuilder) {
    f(&mut self._req_builder); self
  }
  pub async fn send(self) -> SendResult {
    let res = match self._req_builder.send().await {
      Err(e) => return SendResult::ReqwestError(e),
      Ok(v) => v
    }; 
    if is_json(&res) {
      match res.json().await {
        Err(e) => return SendResult::ReqwestError(e),
        Ok(v) => SendResult::JSON(v)
      }
    } else {
      match res.bytes().await {
        Err(e) => return SendResult::ReqwestError(e),
        Ok(v) => SendResult::Buffer(v.to_vec())
      }
    }
  }
}

fn is_json(response: &reqwest::Response) -> bool {
  match match response.headers().get("content-type") {
    None => return false,
    Some(content_type) => content_type
  }.to_str() {
    Err(_) => return false,
    Ok(v) => v
  }.contains("application/json")
}

pub struct Router {
  _route: Mutex<String>,
  _token: String
}

impl Router {
  pub fn new(client: &Client) -> Self {
    Self {
      _route: Mutex::new("https://discord.com/api/v9".to_string()),
      _token: client.token.clone()
    }
  }
  pub fn get(&self) -> Request {
    let mut req_builder = reqwest::Client::new().get(self._route.lock().unwrap().clone());
    req_builder = req_builder.header("Authorization", format!("Bot {}", self._token));
    Request { _req_builder: req_builder }
  }
  pub fn post(&self) -> Request {
    let mut req_builder = reqwest::Client::new().post(self._route.lock().unwrap().clone());
    req_builder = req_builder.header("Authorization", format!("Bot {}", self._token));
    Request { _req_builder: req_builder }
  }
  pub fn patch(&self) -> Request {
    let mut req_builder = reqwest::Client::new().patch(self._route.lock().unwrap().clone());
    req_builder = req_builder.header("Authorization", format!("Bot {}", self._token));
    Request { _req_builder: req_builder }
  }
  pub fn delete(&self) -> Request {
    let mut req_builder = reqwest::Client::new().delete(self._route.lock().unwrap().clone());
    req_builder = req_builder.header("Authorization", format!("Bot {}", self._token));
    Request { _req_builder: req_builder }
  }
  pub fn put(&self) -> Request {
    let mut req_builder = reqwest::Client::new().put(self._route.lock().unwrap().clone());
    req_builder = req_builder.header("Authorization", format!("Bot {}", self._token));
    Request { _req_builder: req_builder }
  }
}

impl<S> std::ops::Index<S> for Router where S: std::string::ToString {
  type Output = Self;
  fn index(&self, index: S) -> &Self {
    let mut route = self._route.lock().unwrap();
    route.push_str("/");
    route.push_str(&index.to_string());
    self
  }
}
