//! This module contains the common driver code. Specific implementations for
//! different databases exist elsewhere.

use std::collections::BTreeMap;
use error::Error;
use value::*;

pub trait Driver {
  /// Connects to a database and returns a driver instance. After calling this
  /// the driver is ready to roll!
  fn connect(url: &str) -> Self;

  /// Set a value at a certain point in the driver.
  fn set(&self, pointer: Pointer, value: Value) -> Result<(), Error>;

  /// Get a value from a certain point in the driver.
  fn get(&self, pointer: Pointer) -> Result<Value, Error>;

  /// Gets the schema for the driver. By default no schema is returned.
  fn get_schema(&self) -> Result<Schema, Error> {
    Ok(Schema::None)
  }
}
