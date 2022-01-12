use eetf::{Term};

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
    },
  }
}

pub struct GatewayMessage {
  pub op: u8,
  pub data: Option<Term>,
  pub sequence: Option<i32>,
  pub title: Option<String>,
}

static BINARY_STRING_OP: &[u8] = "op".as_bytes();
static BINARY_STRING_D: &[u8] = "d".as_bytes();

impl GatewayMessage {
  pub fn new(
    op: u8,
    data: Term
  ) -> GatewayMessage {
    GatewayMessage {
      op: op,
      data: Some(data),
      sequence: None,
      title: None
    }
  }
  pub fn from_term(raw: Term) -> GatewayMessage {
    if let Term::Map(data) = raw {
      let mut message = GatewayMessage { op: 0, data: None, sequence: None, title: None };
      for (key, value) in data.entries {
        if let Term::Atom(name) = key {
          match name.name.as_str() {
            "op" => if let Term::FixInteger(data) = value {
              message.op = data.value as u8;
            } else {
              panic!("The opcode must be an integer");
            },
            "d" => message.data = Some(value),
            "s" => if let Term::FixInteger(data) = value {
              message.sequence = Some(data.value);
            },
            "t" => {
              if let Term::Atom(data) = value {
                message.title = Some(data.name);
              } else if let Term::Binary(data) = value {
                if let Ok(title) = String::from_utf8(data.bytes.clone()) {
                  message.title = Some(title);
                }
              }
            }
            _ => (),
          }
        }
      }
      if message.op != 0 {
        message.title = None;
        message.sequence = None;
      }
      return message;
    }
    panic!("Invalid GatewayMessage");
  }
  pub fn as_buffer(&self) -> Result<Vec<u8>, eetf::EncodeError> {
    let mut buf = Vec::new();
    Term::from(eetf::Map::from(self.as_vec())).encode(&mut buf).map(|_| buf)
  }
  fn as_vec(&self) -> Vec<(Term, Term)> {
    match &self.data {
      Some(data) => vec![
        (to_term(BINARY_STRING_OP), self.get_op()),
        (to_term(BINARY_STRING_D), data.clone())
      ],
      None => vec![(to_term(BINARY_STRING_OP), self.get_op())]
    }
  }
  fn get_op(&self) -> Term {
    Term::from(eetf::FixInteger::from(self.op))
  }
  pub fn to_string(&self) -> String {
    format!(
      "GatewayMessage {{\n  op: {},\n  d: {},\n  s: {},\n  t: {}\n}}", 
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

fn to_term(string: &[u8]) -> Term {
  Term::from(eetf::Binary::from(string))
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