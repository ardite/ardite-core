//! The driver trait which all drivers will implement.

use schema::Definition;
use error::{Error, ErrorCode};
use patch::Patch;
use value::{Pointer, Value};
use query::Query;

pub trait Driver {
  /// Connects to a driver and returns a driver instance. After calling this
  /// the driver is ready to roll!
  fn connect(uri: &str) -> Result<Box<Self>, Error>;

  /// Validates an Ardite Schema Definition dependending on the driver’s
  /// contracts with the developer. Note that the definition object will not be
  /// associated with the driver object. This is intentional, the driver should
  /// not be able to influence the definition without an external tool.
  fn validate_definition(_: &Definition) -> Result<(), Error> {
    Ok(())
  }

  /// Performs a complex query on the driver. Returns a value whose shape
  /// matches the shape of the query.
  fn query(&self, query: Query) -> Result<Value, Error>;

  /// Applies multiple patches to the driver. If one patch fails, all other
  /// patches must also fail. Returns a value with all of the new patched
  /// values along with any driver generated values only.
  fn patch(&self, patch: Vec<Patch>) -> Result<Value, Error>;
  
  /// Get’s a value in the driver at a specific point and returns exactly that
  /// value.
  fn get(&self, pointer: Pointer) -> Result<Value, Error> {
    match try!(self.query(Query::from(pointer.clone()))).get(pointer) {
      Some(value) => Ok(value),
      None => Err(Error {
        code: ErrorCode::Internal,
        message: String::from("Driver failed to return a value with the requested data."),
        hint: None
      })
    }
  }
  
  /// Set a value at a certain point in the driver. Returns nothing.
  fn set(&self, pointer: Pointer, value: Value) -> Result<(), Error> {
    try!(self.patch(vec![Patch::Set(pointer, value)]));
    Ok(())
  }
}
