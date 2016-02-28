//! This module contains the common driver code. Specific implementations for
//! different drivers exist elsewhere.

use error::Error;
use value::{Pointer, Value};
use query::Query;
use schema::Schema;

/// Gets the driver from a URL string using the protocol. For example a URL
/// of `postgres://localhost:5432/test_db` would look for a
/// `ardite-driver-postgres` crate, download the crate if it did not already
/// exist in the file system, and then return an instance initialized with the
/// `connect` static trait function.
#[allow(unused_variables)]
pub fn get_driver<D: Driver>(url: &str) -> D {
  // TODO: implement.
  unimplemented!();
}

pub trait Driver {
  /// Connects to a driver and returns a driver instance. After calling this
  /// the driver is ready to roll!
  fn connect(url: &str) -> Result<&Self, Error>;

  /// Set a value at a certain point in the driver.
  fn set(&self, pointer: Pointer, value: Value) -> Result<(), Error>;

  /// Query a subset of values in the database.
  fn query(&self, query: Query) -> Result<Value, Error>;

  /// Gets a value from a certain *exact* point in the driver. This method by
  /// default uses a `Query` based implementation, however driver authors may
  /// choose to optimize.
  fn get(&self, pointer: Pointer) -> Result<Value, Error> {
    self.query(Query::from(pointer)).get(pointer)
  }

  /// Gets the schema for the driver. By default no schema is returned.
  /// Whether or not this method gets memoized is the driver implementors
  /// decision.
  fn schema(&self) -> Result<Schema, Error> {
    Ok(Schema::None)
  }
}
