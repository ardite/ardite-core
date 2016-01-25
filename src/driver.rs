//! This module contains the common driver code. Specific implementations for
//! different databases exist elsewhere.

use error::Error;
use values::Value;
use structure::Structure;
use request::Request;

/// The trait to be implemented by all drivers.
pub trait Driver {
  /// Get‘s the structure of the underlying database. This will run whenever a
  /// service is starting up.
  fn get_structure() -> Result<Structure, Error>;

  /// Send a request to the database.
  fn request(request: Request) -> Result<Vec<Value>, Error>;

  /// Send a request to the database and guarantee it only ever effects a
  /// single document.
  fn request_one(request: Request) -> Result<Value, Error> {
    // TODO: default implementation which verifies that the request is only
    // affecting a single thing. This may be done by checking if the collection
    // primary key is specified in a filter.
    unimplemented!()
  }
}
