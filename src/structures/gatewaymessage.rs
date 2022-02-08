fn term_to_string(d: &eetf::Term, initial_indent: u32, space: u32) -> String {
  match d {
    eetf::Term::Atom(data) => format!("\"{}\"", data.name),
    eetf::Term::FixInteger(data) => format!("{}", data.value),
    eetf::Term::BigInteger(data) => format!("{}", data.value),
    eetf::Term::Float(data) => format!("{}", data.value),
    eetf::Term::Pid(data) => format!("Pid{}", data),
    eetf::Term::Port(data) => format!("{}", data),
    eetf::Term::Reference(data) => format!("{}", data),
    eetf::Term::ExternalFun(data) => format!("{}", data),
    eetf::Term::InternalFun(data) => format!("{}", data),
    eetf::Term::Binary(data) => {
      if let Ok(text) = String::from_utf8(data.bytes.clone()) {
        format!("\"{}\"", text)
      } else {
        let mut out = String::new();
        out.push_str("<Binary");
        for byte in &data.bytes {
          out.push_str(format!(" {:02x}", byte).as_str());
        }
        out.push_str(">");
        out
      }
    },
    eetf::Term::BitBinary(data) => format!("BitBinary{}", data),
    eetf::Term::List(data) => {
      let mut out = String::new();
      out.push_str("[ ");
      for (i, d) in data.elements.iter().enumerate() {
        if i != 0 { out.push_str(", ") }
        out.push_str(term_to_string(&d, initial_indent, space).as_str())
      }
      out.push_str(" ]");
      out
    },
    eetf::Term::ImproperList(data) => format!("{}", data),
    eetf::Term::Tuple(data) => {
      let mut out = String::new();
      out.push_str("Tuple( ");
      for (i, d) in data.elements.iter().enumerate() {
        if i != 0 { out.push_str(", ") }
        out.push_str(term_to_string(&d, initial_indent, space).as_str())
      }
      out.push_str(" )");
      out
    },
    eetf::Term::Map(data) => {
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
  pub data: Option<eetf::Term>,
  pub sequence: Option<i32>,
  pub title: Option<String>
}

impl GatewayMessage {
  pub fn new(
    shard_id: u64,
    op: u8,
    data: eetf::Term
  ) -> GatewayMessage {
    GatewayMessage {
      shard_id,
      op,
      data: Some(data),
      sequence: None,
      title: None
    }
  }
  pub fn from_term(shard_id: u64, term: eetf::Term) -> Result<GatewayMessage, &'static str> {
    match term {
      eetf::Term::Map(d) => parse_eetf_map(d.entries).map(|(op, data, sequence, title)| GatewayMessage {
        shard_id, op, data, sequence, title
      }),
      _ => Err("Invalid format: Expected map")
    }
  }
  pub fn to_buffer(&self) -> Result<Vec<u8>, eetf::EncodeError> {
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

fn parse_eetf_map(map: Vec<(eetf::Term, eetf::Term)>) -> Result<(u8, Option<eetf::Term>, Option<i32>, Option<String>), &'static str> {
  let mut op: Option<u8> = None;
  let mut data: Option<eetf::Term> = None;
  let mut sequence: Option<i32> = None;
  let mut _title: Option<String> = None;
  for (key, value) in map {
    match match key {
      eetf::Term::Atom(name) => name.name,
      eetf::Term::Binary(name) => match String::from_utf8(name.bytes.clone()) {
        Ok(s) => s,
        _ => { continue; }
      },
      _ => { continue; }
    }.as_str() {
      "op" => {
        if let eetf::Term::FixInteger(data) = value {
          op = Some(data.value as u8);
        } else {
          return Err("The opcode must be an integer");
        }
      },
      "d" => {
        data = Some(value);
      },
      "s" => {
        if let eetf::Term::FixInteger(data) = value {
          sequence = Some(data.value); 
        }
      },
      "t" => match value {
        eetf::Term::Atom(data) => {
          _title = Some(data.name);
        },
        eetf::Term::Binary(data) => {
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

fn gateway_message_to_term(op: u8, data: &Option<eetf::Term>) -> eetf::Term {
  let binary_string_op = eetf::Term::Binary(eetf::Binary{ bytes: vec![0x6f, 0x70] });
  let op = eetf::Term::FixInteger(eetf::FixInteger{ value: op as i32 });
  eetf::Term::Map(eetf::Map{
    entries: match data {
      Some(data) => vec![
        (binary_string_op, op),
        (eetf::Term::Binary(eetf::Binary{ bytes: vec![0x64] }), data.clone())
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
