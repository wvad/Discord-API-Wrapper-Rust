use discors::client::Client;

#[test]
fn test() {
  futures::executor::block_on(async {
    let mut client = Client::new("TOKEN".to_string());
    client.connect().await.unwrap();
    client.wait_for_shards();
  });
}
