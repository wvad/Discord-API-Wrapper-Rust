use crate::client::ClientData;
use reqwest::{RequestBuilder, Response};
use std::sync::{Arc, Mutex};

type SharedPtr<T> = Arc<Mutex<T>>;
pub struct RestRequest {
  _req_builder: RequestBuilder
}

#[derive(Debug)]
pub enum SendResult {
  Error(reqwest::Error),
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

impl RestRequest {
  pub fn edit<F>(&mut self, f: F) -> &mut Self where F: FnOnce(&mut RequestBuilder) {
    f(&mut self._req_builder); self
  }
  pub async fn send(self) -> SendResult {
    let res = match self._req_builder.send().await {
      Err(e) => return SendResult::Error(e),
      Ok(v) => v
    }; 
    if is_json(&res) {
      match res.json().await {
        Err(e) => return SendResult::Error(e),
        Ok(v) => SendResult::JSON(v)
      }
    } else {
      match res.bytes().await {
        Err(e) => return SendResult::Error(e),
        Ok(v) => SendResult::Buffer(v.to_vec())
      }
    }
  }
}

fn is_json(response: &Response) -> bool {
  match match response.headers().get("content-type") {
    None => return false,
    Some(content_type) => content_type
  }.to_str() {
    Err(_) => return false,
    Ok(v) => v
  }.contains("application/json")
}

pub struct APIRouter {
  _route: Mutex<String>,
  _data: SharedPtr<ClientData>
}

impl APIRouter {
  pub fn new(data: &SharedPtr<ClientData>) -> Self {
    Self {
      _route: Mutex::new("https://discord.com/api/v9".to_string()),
      _data: data.clone()
    }
  }
  fn _url(&self) -> String {
    self._route.lock().unwrap().clone()
  }
  fn _build_request(&self, req: RequestBuilder) -> RestRequest {
    RestRequest { 
      _req_builder: match  &self._data.lock().unwrap().token {
        Some(token) => req.header("Authorization", format!("Bot {}", token)),
        None => req
      }
    }
  }
  pub fn get(&self) -> RestRequest {
    self._build_request(reqwest::Client::new().get(self._url()))
  }
  pub fn post(&self) -> RestRequest {
    self._build_request(reqwest::Client::new().post(self._url()))
  }
  pub fn patch(&self) -> RestRequest {
    self._build_request(reqwest::Client::new().patch(self._url()))
  }
  pub fn delete(&self) -> RestRequest {
    self._build_request(reqwest::Client::new().delete(self._url()))
  }
  pub fn put(&self) -> RestRequest {
    self._build_request(reqwest::Client::new().put(self._url()))
  }
}

impl<S> std::ops::Index<S> for APIRouter where S: std::string::ToString {
  type Output = Self;
  fn index(&self, index: S) -> &Self {
    let mut route = self._route.lock().unwrap();
    route.push_str("/");
    route.push_str(&index.to_string());
    self
  }
}
