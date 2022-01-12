mod discors;
use discors::client::Client;


fn main() {
  let mut client = Client::new("TOKEN".to_string());
  async_std::task::block_on(client.connect()).unwrap();
  client.wait_for_shards();
}
