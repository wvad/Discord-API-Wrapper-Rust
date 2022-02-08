use eetf::Term;
use crate::JSONValue;
use crate::etf;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum SendOP {
  Heartbeat = 1,
  Identify = 2,
  PresenceUpdate = 3,
  VoiceStateUpdate = 4,
  Resume = 6,
  RequestGuildMembers = 8
}

pub struct SendGatewayMessage {
  pub shard_id: u64,
  pub op: SendOP,
  pub data: Term
}

impl SendGatewayMessage {
  pub fn to_buffer(&self) -> Result<Vec<u8>, eetf::EncodeError> {
    let mut buf = Vec::new();
    etf!({
      "op": self.op as u8,
      "d": self.data
    }).encode(&mut buf).map(|_| buf)
  }
}

pub struct ReceivedGatewayMessage {
  pub shard_id: u64,
  pub op: u8,
  pub data: JSONValue,
  pub sequence: Option<i32>,
  pub title: Option<String>
}

impl ReceivedGatewayMessage {
  pub fn from_term(shard_id: u64, term: &Term) -> Result<ReceivedGatewayMessage, &'static str> {
    let map = match &term {
      Term::Map(d) => &d.entries,
      _ => return Err("Invalid format: Expected map")
    };
    let mut op: Option<u8> = None;
    let mut _data: Option<&Term> = None;
    let mut sequence: Option<i32> = None;
    let mut _title: Option<&str> = None;
    for (key, value) in map {
      let keystr = match key {
        Term::Atom(name) => name.name.as_str(),
        Term::Binary(name) => match std::str::from_utf8(&name.bytes) {
          Ok(s) => s,
          _ => { continue; }
        },
        _ => { continue; }
      };
      match keystr {
        "op" => {
          if let Term::FixInteger(data) = value {
            op = Some(data.value as u8);
          } else {
            return Err("The opcode must be an integer");
          }
        },
        "d" => {
          _data = Some(value);
        },
        "s" => {
          if let Term::FixInteger(data) = value {
            sequence = Some(data.value); 
          }
        },
        "t" => match value {
          Term::Atom(data) => {
            _title = Some(&data.name);
          },
          Term::Binary(data) => {
            if let Ok(title) = std::str::from_utf8(&data.bytes) {
              _title = Some(title);
            }
          },
          _ => ()
        },
        _ => ()
      }
    }
    let data = match _data {
      Some(d) => JSONValue::from_term(d),
      None => JSONValue::Null
    };
    match op {
      Some(op) => {
        if op != 0 {
          _title = None;
        }
        Ok(ReceivedGatewayMessage {
          shard_id, op, sequence, data,
          title: _title.map(|s|s.to_string())
        })
      },
      _ => Err("The opcode missing")
    }
  }
  pub fn to_string(&self) -> String {
    format!(
      "GatewayMessage (shard_id: {}) {{\n  op: {},\n  d: {},\n  s: {},\n  t: {}\n}}",
      self.shard_id,
      self.op,
      self.data.to_string(2, 2),
      match self.sequence {
        Some(seq) => format!("{}", seq),
        None => "null".to_string()
      },
      match &self.title {
        Some(t) => format!("{}", t),
        None => "null".to_string()
      }
    )
  }
}

impl std::fmt::Display for ReceivedGatewayMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

impl std::fmt::Debug for ReceivedGatewayMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}
