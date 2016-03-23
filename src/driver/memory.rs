use url::Url;

use driver::Driver;
use error::Error;
use query::{Range, SortRule, Condition, Query};
use value::{Key, Iter};

pub struct Memory;

impl Memory {
  pub fn new() -> Self {
    Memory
  }
}

impl Driver for Memory {
  fn connect(_: &Url) -> Result<Self, Error> {
    Err(Error::invalid("You can’t connect to memory silly.", "Use the `new` method instead for the memory driver."))
  }

  fn read(
    &self,
    _: &Key,
    _: Condition,
    _: Vec<SortRule>,
    _: Range,
    _: Query
  ) -> Result<Iter, Error> {
    unimplemented!();
  }
}
