/*
  match message.op {
    0 => {
      // dispatch
    },
    7 => {
      // reconnect
    },
    9 => {
      // invalid session
    },
    10 => {
      // hello
    },
    11 => {
      // hertbeat ACK
    },
  }
*/
pub mod client;
pub mod gatewaymessage;
pub mod websocket;
