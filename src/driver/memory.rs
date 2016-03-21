use driver::Driver;
use error::Error;
use query::{Range, SortRule, Condition, Query};
use schema::Type;

struct MemoryDriver;

impl Driver for MemoryDriver {
  fn connect(uri: &str) -> Result<Self, Error> {
    Ok(MemoryDriver)
  }

  fn read(
    &self,
    _: &Type,
    _: Condition,
    _: Vec<SortRule>,
    _: Range,
    _: Query
  ) {
    unimplemented!();
  }
}
