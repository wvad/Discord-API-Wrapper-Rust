use discors::Client;
use discors::GatewayMessage;
use discors::etf;

#[tokio::test]
async fn async_test() {
  let mut client = Client::new();
  let token = "TOKEN HERE";
  client.login(token).await;
  while !client.shards.is_finished() {
    for message in client.shards.get_messages() {
      println!("[INFO] Received Message: {:?}", message);
      match message.op {
        10 => {
          let message = GatewayMessage::new(message.shard_id, 2, etf!({
            "compress": true,
            "token": token,
            "intents": 32767,
            "properties": {
              "$os": "windows",
              "$browser": "discors",
              "$device": "discors"
            }
          }));
          client.shards.send(message);
        },
        _ => ()
      }
    }
  }
  println!("[INFO] Finished Data: {:?}", client.shards.wait());
}
