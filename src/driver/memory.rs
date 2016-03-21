use driver::Driver;
use error::Error;
use query::{Range, SortRule, Condition, Query};
use value::{Key, ValueIter};

struct MemoryDriver;

impl Driver for MemoryDriver {
  fn connect(_: &str) -> Result<Self, Error> {
    Ok(MemoryDriver)
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
