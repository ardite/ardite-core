//! The driver trait which all drivers will implement.

use error::Error;
use query::{Condition, SortRule, Range, Query};
use schema::Type;
use value::ValueIter;

pub trait Driver {
  /// Connects to a driver and returns a driver instance. After calling this
  /// the driver is ready to roll!
  fn connect(uri: &str) -> Result<Self, Error> where Self: Sized;

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
  fn read(
    &self,
    type_: &Type,
    condition: Condition,
    sort: Vec<SortRule>,
    range: Range,
    query: Query
  ) -> Result<ValueIter, Error>;
}
