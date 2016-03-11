//! The driver trait which all drivers will implement.

use error::Error;
use query::{Condition, Range, Query};
use schema::Type;
use value::ValueStream;

pub trait Driver {
  /// Connects to a driver and returns a driver instance. After calling this
  /// the driver is ready to roll!
  fn connect(uri: &str) -> Result<Self, Error> where Self: Sized;

  // TODO: doc.
  fn read(
    &self,
    type_: &Type,
    condition: Condition,
    range: Range,
    query: Query
  ) -> Result<ValueStream, Error>;
}
