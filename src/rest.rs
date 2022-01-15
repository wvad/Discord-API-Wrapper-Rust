use crate::client::Client;

pub enum Method {
  GET,
  POST,
  PATCH,
  DELETE,
  PUT
}

pub async fn request<F>(
  client: &Client,
  method: Method,
  route: &str,
  builder: F
) -> surf::Result<surf::Response> where F: Fn(&mut surf::RequestBuilder) {
  let mut req_builder = match method {
    Method::GET => surf::get,
    Method::POST => surf::post,
    Method::PATCH => surf::patch,
    Method::DELETE => surf::delete,
    Method::PUT => surf::put
  }(format!("https://discord.com/api/v9/{}", route));
  builder(&mut req_builder);
  req_builder
  .header("Authorization", format!("Bot {}", client.token))
  .send()
  .await
}
