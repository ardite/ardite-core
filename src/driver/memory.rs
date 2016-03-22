use url::Url;

use driver::Driver;
use error::Error;
use query::{Range, SortRule, Condition, Query};
use value::{Key, ValueIter};

struct MemoryDriver;

impl MemoryDriver {
  pub fn new() -> Self {
    MemoryDriver
  }
}

impl Driver for MemoryDriver {
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
  ) -> Result<ValueIter, Error> {
    unimplemented!();
  }
}
