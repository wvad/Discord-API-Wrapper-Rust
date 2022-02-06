use discors::client::Client;
use discors::gatewaymessage::GatewayMessage;
use discors::etf;

#[tokio::test]
async fn async_test() {
  let mut client = Client::new("NzQxODg4NzQyNzc5OTEyMzIz.Xy-Huw.TtbpvL5-mUiJSshv7ElHdF8Y1Nc".to_string());
  println!("[INFO] Connection's error: {:?}", client.connect().await);
  while !client.shards.is_finished() {
    for message in client.shards.get_messages() {
      println!("[INFO] Received Message: {:?}", message);
      match message.op {
        10 => {
          let message = GatewayMessage::new(message.shard_id, 2, etf!({
            "compress": true,
            "token": client.token.clone(),
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
