use eetf::Term;
use inflate::InflateStream;

fn term_to_string(d: &Term, initial_indent: u32, space: u32) -> String {
  match d {
    Term::Atom(data) => format!("\"{}\"", data.name),
    Term::FixInteger(data) => format!("{}", data.value),
    Term::BigInteger(data) => format!("{}", data.value),
    Term::Float(data) => format!("{}", data.value),
    Term::Pid(data) => format!("Pid{}", data),
    Term::Port(data) => format!("{}", data),
    Term::Reference(data) => format!("{}", data),
    Term::ExternalFun(data) => format!("{}", data),
    Term::InternalFun(data) => format!("{}", data),
    Term::Binary(data) => {
      let mut out = String::new();
      out.push_str("<Binary");
      for byte in &data.bytes {
        out.push_str(format!(" {:02x}", byte).as_str());
      }
      out.push_str(">");
      out
    },
    Term::BitBinary(data) => format!("BitBinary{}", data),
    Term::List(data) => {
      let mut out = String::new();
      out.push_str("[ ");
      for (i, d) in data.elements.iter().enumerate() {
        if i != 0 { out.push_str(", ") }
        out.push_str(term_to_string(&d, initial_indent, space).as_str())
      }
      out.push_str(" ]");
      out
    },
    Term::ImproperList(data) => format!("{}", data),
    Term::Tuple(data) => {
      let mut out = String::new();
      out.push_str("Tuple( ");
      for (i, d) in data.elements.iter().enumerate() {
        if i != 0 { out.push_str(", ") }
        out.push_str(term_to_string(&d, initial_indent, space).as_str())
      }
      out.push_str(" )");
      out
    },
    Term::Map(data) => {
      let mut out = String::new();
      out.push_str("{\n");
      let nested = initial_indent + space;
      for (index, (key, value)) in data.entries.iter().enumerate() {
        if index != 0 { out.push_str(",\n") }
        for _ in 0..nested { out.push_str(" ") }
        out.push_str(term_to_string(&key, nested, space).as_str());
        out.push_str(": ");
        out.push_str(term_to_string(&value, nested, space).as_str())
      }
      out.push_str("\n");
      for _ in 0..initial_indent { out.push_str(" ") }
      out.push_str("}");
      out
    }
  }
}

pub struct GatewayMessage {
  pub shard_id: u64,
  pub op: u8,
  pub data: Option<Term>,
  pub sequence: Option<i32>,
  pub title: Option<String>
}

#[derive(Debug)]
pub enum GatewayMessageDecodeError {
  DecommpressError(String),
  DecodeError(eetf::DecodeError),
  ParseError(&'static str)
}

impl GatewayMessage {
  pub fn new(
    shard_id: u64,
    op: u8,
    data: Term
  ) -> GatewayMessage {
    GatewayMessage {
      shard_id,
      op,
      data: Some(data),
      sequence: None,
      title: None
    }
  }
  pub fn from_zlib(shard_id: u64, inflate: &mut InflateStream, raw: &[u8]) -> Result<GatewayMessage, GatewayMessageDecodeError> {
    let decompressed = {
      let mut decoded = Vec::<u8>::new();
      let mut n = 0;
      loop {
        let (num_bytes_read, bytes) = match inflate.update(&raw[n..]) {
          Err(e) => {
            return Err(GatewayMessageDecodeError::DecommpressError(e));
          },
          Ok(r) => r
        };
        if bytes.is_empty() { break; }
        n += num_bytes_read;
        decoded.extend_from_slice(bytes);
      }
      decoded
    };
    let decoded = match Term::decode(&decompressed[..]) {
      Err(e) => return Err(GatewayMessageDecodeError::DecodeError(e)),
      Ok(d) => d
    };
    match parse_websocket_message(shard_id, decoded) {
      Err(e) => Err(GatewayMessageDecodeError::ParseError(e)),
      Ok(d) => Ok(d)
    }
  }
  pub fn as_buffer(&self) -> Result<Vec<u8>, eetf::EncodeError> {
    let mut buf = Vec::new();
    gateway_message_to_term(self.op, &self.data).encode(&mut buf).map(|_| buf)
  }
  pub fn to_string(&self) -> String {
    format!(
      "GatewayMessage (shard_id: {}) {{\n  op: {},\n  d: {},\n  s: {},\n  t: {}\n}}",
      self.shard_id,
      self.op,
      match &self.data {
        Some(data) => term_to_string(&data, 2, 2),
        None => "null".to_string()
      },
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

fn parse_websocket_message(shard_id: u64, raw: Term) -> Result<GatewayMessage, &'static str> {
  match raw {
    Term::Map(d) => parse_eetf_map(d.entries).map(|(op, data, sequence, title)| GatewayMessage {
      shard_id, op, data, sequence, title
    }),
    _ => Err("Invalid format: Expected map")
  }
}

fn parse_eetf_map(map: Vec<(Term, Term)>) -> Result<(u8, Option<Term>, Option<i32>, Option<String>), &'static str> {
  let mut op: Option<u8> = None;
  let mut data: Option<Term> = None;
  let mut sequence: Option<i32> = None;
  let mut _title: Option<String> = None;
  for (key, value) in map {
    match match key {
      Term::Atom(name) => name.name,
      Term::Binary(name) => match String::from_utf8(name.bytes.clone()) {
        Ok(s) => s,
        _ => { continue; }
      },
      _ => { continue; }
    }.as_str() {
      "op" => {
        if let Term::FixInteger(data) = value {
          op = Some(data.value as u8);
        } else {
          return Err("The opcode must be an integer");
        }
      },
      "d" => {
        data = Some(value);
      },
      "s" => {
        if let Term::FixInteger(data) = value {
          sequence = Some(data.value); 
        }
      },
      "t" => match value {
        Term::Atom(data) => {
          _title = Some(data.name);
        },
        Term::Binary(data) => {
          if let Ok(title) = String::from_utf8(data.bytes.clone()) {
            _title = Some(title);
          }
        },
        _ => ()
      },
      _ => ()
    }
  }
  match op {
    Some(op) => Ok(if op == 0 {(op, data, sequence, _title)} else {(op, data, None, None)}),
    _ => Err("The opcode missing")
  }
}

fn gateway_message_to_term(op: u8, data: &Option<Term>) -> Term {
  let binary_string_op = Term::Binary(eetf::Binary{ bytes: vec![0x6f, 0x70] });
  let op = Term::FixInteger(eetf::FixInteger{ value: op as i32 });
  Term::Map(eetf::Map{
    entries: match data {
      Some(data) => vec![
        (binary_string_op, op),
        (Term::Binary(eetf::Binary{ bytes: vec![0x64] }), data.clone())
      ],
      None => vec![(binary_string_op, op)]
    }
  })
}

impl std::fmt::Display for GatewayMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

impl std::fmt::Debug for GatewayMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}