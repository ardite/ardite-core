//! Provides a default and reference driver implementation which stores all of
//! its information in memory.

use url::Url;

use driver::Driver;
use error::Error;
use query::{Range, SortRule, Condition, Query};
use value::{Key, Iter};

/// The default driver to be used by a service when no other driver is
/// specified. This driver, unlike the others, stores all of its data in
/// memory. The best usecase for this driver is in tests.
///
/// This driver also serves as a good reference implementation for those
/// looking to create a production-ready driver.
pub struct Memory;

impl Memory {
  /// Creates a new instance of the memory driver.
  pub fn new() -> Self {
    Memory
  }
}

impl Driver for Memory {
  /// Connecting the memory driver in this way will *always* be an error. This
  /// is because the memory driver doesn’t depend on any `url` (as its data
  /// *is* stored locally in memory). Instead use the `new` function provided
  /// on the struct.
  ///
  /// This method may eventually not return an error if a valid use case is
  /// shown for the memory driver in production, so do not depend on this
  /// functionality.
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
