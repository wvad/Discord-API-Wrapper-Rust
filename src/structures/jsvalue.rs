use eetf::Term;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JSONValue {
  Object(HashMap<String, JSONValue>),
  Array(Vec<JSONValue>),
  Str(String),
  Number(f64),
  Bool(bool),
  Null
}

impl JSONValue {
  pub fn from_term(d: &Term) -> JSONValue {
    match d {
      Term::Atom(data) => match data.name.as_str() {
        "nil" => JSONValue::Null,
        "true" => JSONValue::Bool(true),
        "false" => JSONValue::Bool(false),
        _ => JSONValue::Str(data.name.clone())
      },
      Term::FixInteger(data) => JSONValue::Number(data.value.into()),
      Term::BigInteger(data) => JSONValue::Str(format!("{}", data.value)),
      Term::Float(data) => JSONValue::Number(data.value),
      Term::Binary(data) => {
        match String::from_utf8(data.bytes.clone()) {
          Ok(text) => JSONValue::Str(text),
          _ => JSONValue::Null
        }
      },
      Term::List(data) => {
        JSONValue::Array(data.elements.iter().map(|d| JSONValue::from_term(d)).collect())
      },
      Term::Tuple(data) => {
        JSONValue::Array(data.elements.iter().map(|d| JSONValue::from_term(d)).collect())
      },
      Term::Map(data) => {
        JSONValue::Object(data.entries.iter().map(|(key, value)| {
          (match key {
            Term::Atom(data) => data.name.clone(),
            Term::Binary(data) => match String::from_utf8(data.bytes.clone()) {
              Ok(text) => text,
              _ => "INVALID_KEY".to_string()
            }
            _ => "INVALID_KEY".to_string()
          }, JSONValue::from_term(value))
        }).collect())
      },
      _ => JSONValue::Null
    }
  }
  pub fn to_string(&self, initial_indent: u32, space: u32) -> String {
    match self {
      JSONValue::Object(data) => {
        let mut out = String::new();
        out.push_str("{\n");
        let nested = initial_indent + space;
        for (index, (key, value)) in data.iter().enumerate() {
          if index != 0 { out.push_str(",\n") }
          for _ in 0..nested { out.push_str(" ") }
          out.push_str(&key);
          out.push_str(": ");
          out.push_str(&value.to_string(nested, space))
        }
        out.push_str("\n");
        for _ in 0..initial_indent { out.push_str(" ") }
        out.push_str("}");
        out
      },
      JSONValue::Array(data) => {
        let mut out = String::new();
        out.push_str("[ ");
        for (i, d) in data.iter().enumerate() {
          if i != 0 { out.push_str(", ") }
          out.push_str(&d.to_string(initial_indent, space))
        }
        out.push_str(" ]");
        out
      },
      JSONValue::Str(data) => format!("\"{}\"", data),
      JSONValue::Number(data) => format!("{}", data),
      JSONValue::Bool(data) => format!("{}", data),
      JSONValue::Null => "null".to_string()
    }
  }
}



