use discors::{Client, SendGatewayMessage, SendOP, etf};

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
          client.shards.send(SendGatewayMessage {
            shard_id: message.shard_id,
            op: SendOP::Identify,
            data: etf!({
              "compress": true,
              "token": token,
              "intents": 32767,
              "properties": {
                "$os": "windows",
                "$browser": "discors",
                "$device": "discors"
              }
            })
          });
        },
        _ => ()
      }
    }
  }
  println!("[INFO] Finished Data: {:?}", client.shards.wait());
}
