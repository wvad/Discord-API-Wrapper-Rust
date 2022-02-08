use eetf::Term;

pub trait TermConvertable {
  fn to_term(&self) -> Term;
}

pub fn to_term<T>(v: &T) -> Term where T: TermConvertable {
  TermConvertable::to_term(v)
}

impl TermConvertable for Term {
  fn to_term(&self) -> Term {
    self.clone()
  }
}

impl<'a> TermConvertable for &'a str {
  fn to_term(&self) -> Term {
    Term::Binary(eetf::Binary{ bytes: self.as_bytes().to_vec() })
  }
}

impl TermConvertable for String {
  fn to_term(&self) -> Term {
    Term::Binary(eetf::Binary{ bytes: self.clone().into_bytes() })
  }
}

impl<'a> TermConvertable for &'a[u8] {
  fn to_term(&self) -> Term {
    Term::Binary(eetf::Binary{ bytes: self.to_vec() })
  }
}

impl TermConvertable for Vec<u8> {
  fn to_term(&self) -> Term {
    Term::Binary(eetf::Binary{ bytes: self.clone() })
  }
}

impl TermConvertable for u8 {
  fn to_term(&self) -> Term {
    Term::FixInteger(eetf::FixInteger{ value: *self as i32 })
  }
}

impl TermConvertable for i32 {
  fn to_term(&self) -> Term {
    Term::FixInteger(eetf::FixInteger{ value: *self })
  }
}

impl TermConvertable for u64 {
  fn to_term(&self) -> Term {
    Term::BigInteger(eetf::BigInteger::from(*self))
  }
}

impl TermConvertable for i64 {
  fn to_term(&self) -> Term {
    Term::BigInteger(eetf::BigInteger::from(*self))
  }
}

impl TermConvertable for usize {
  fn to_term(&self) -> Term {
    Term::BigInteger(eetf::BigInteger::from(*self))
  }
}

impl TermConvertable for isize {
  fn to_term(&self) -> Term {
    Term::BigInteger(eetf::BigInteger::from(*self))
  }
}

impl TermConvertable for f32 {
  fn to_term(&self) -> Term {
    Term::Float(eetf::Float{ value: *self as f64 })
  }
}

impl TermConvertable for f64 {
  fn to_term(&self) -> Term {
    Term::Float(eetf::Float{ value: *self })
  }
}
