use eetf::{DecodeError, Term};
use inflate::InflateStream;

#[derive(Debug)]
pub enum InflateError {
  DecommpressError(String),
  DecodeError(DecodeError)
}

#[derive(Debug)]
pub enum OptionalResult<T, U> {
  Ok(T),
  Err(U),
  None
}

pub struct Inflate {
  _inflate: InflateStream,
  _cache: Vec<u8>
}

impl Inflate {
  pub fn new() -> Inflate {
    Inflate {
      _inflate: InflateStream::from_zlib(),
      _cache: Vec::new()
    }
  }
  pub fn to_term(&mut self, raw: &[u8]) -> OptionalResult<Term, InflateError> {
    self._cache.extend(raw);
    if (raw.len()) < 4 || (raw[(raw.len() - 4)..] != [0x00, 0x00, 0xff, 0xff]) { 
      return OptionalResult::None;
    }
    let decompressed = {
      let mut cache = Vec::new();
      let mut n = 0;
      loop {
        let (read_bytes, bytes) = match self._inflate.update(&self._cache[n..]) {
          Err(e) => {
            self._cache.clear();
            return OptionalResult::Err(InflateError::DecommpressError(e));
          },
          Ok(r) => r
        };
        if bytes.is_empty() { break; }
        n += read_bytes;
        cache.extend_from_slice(bytes);
      }
      cache
    };
    self._cache.clear();
    match eetf::Term::decode(&decompressed[..]) {
      Ok(term) => OptionalResult::Ok(term),
      Err(e) => OptionalResult::Err(InflateError::DecodeError(e))
    }
  }
}
