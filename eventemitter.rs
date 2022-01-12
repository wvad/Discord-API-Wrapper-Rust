use std::collections::HashMap;

pub struct EventEmitter<T> where T: Send + 'static {
  current_id: u128,
  handlers: HashMap<u128, Box<dyn FnMut(&T)>>,
}

impl<T> EventEmitter<T> where T: Send + 'static {
  pub fn new() -> Self {
    EventEmitter {
      current_id: 0,
      handlers: HashMap::new(),
    }
  }
  pub fn on(&mut self, callback: Box<dyn FnMut(&T)>) -> u128 {
    self.handlers.insert(self.current_id, callback);
    let current_id = self.current_id;
    self.current_id += 1;
    current_id
  }
  pub fn off(&mut self, id: u128) {
    self.handlers.remove(&id);
  }
  pub fn emit(&mut self, data: &T) {
    for callback in self.handlers.values_mut() { callback(&data) }
  }
}
