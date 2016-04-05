//! This module contains the common driver code. Specific implementations for
//! different drivers exist elsewhere.

mod discover;
mod memory;
#[cfg(feature = "driver_mongodb")]
pub mod mongodb;

pub use self::discover::discover_driver;
pub use self::memory::Memory;

use url::Url;

use error::Error;
use query::{Condition, Sort, Range};
use value::{Value, Iter};

/// The driver trait which all drivers will implement. Designed to be
/// interoperable with any data source, however the driver also assumes a
/// collection based data model.
pub trait Driver: Send + Sync {
  /// Connects to a driver and returns a driver instance. After calling this
  /// the driver is ready to roll!
  ///
  /// No schema definition is provided to the driver in its construction step.
  fn connect(url: &Url) -> Result<Self, Error> where Self: Sized;

  /// Lazily read some values from the driver.
  ///
  /// Designed against a couple of database specifications. Including the
  /// following:
  ///
  /// - [SQL `SELECT` statement][1].
  /// - [MongoDB `find` command][2].
  ///
  /// [1]: http://www.postgresql.org/docs/current/static/sql-select.html
  /// [2]: https://docs.mongodb.org/manual/reference/command/find/
  // TODO: Enforce this to *only* return objects.
  fn read(
    &self,
    name: &str,
    condition: Condition,
    sorts: Vec<Sort>,
    range: Range
  ) -> Result<Iter, Error>;

  /// Read a single value from the driver. The default implementation uses the
  /// driver read method with a range of one.
  ///
  /// If a condition matches more than one value (while not recommended for
  /// this method) the first of these values, using the default sorting
  /// algorithm of the database, is returned.
  ///
  /// This method may be optionally optimized by the driver.
  fn read_one(
    &self,
    name: &str,
    condition: Condition
  ) -> Result<Value, Error> {
    let mut values = try!(self.read(
      name,
      condition,
      Default::default(),
      Range::new(None, Some(1))
    ));

    if let Some(value) = values.next() {
      if values.next().is_none() {
        Ok(value)
      } else {
        Err(Error::internal("Read with a limit of one returned more than one value."))
      }
    } else {
      Err(Error::not_found("No value was found for the condition."))
    }
  }
}
