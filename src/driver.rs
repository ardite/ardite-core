//! This module contains the common driver code. Specific implementations for
//! different databases exist elsewhere.

use error::Error;
use document::Document;
use structure::Structure;
use request::Request;

/// The trait to be implemented by all drivers.
pub trait Driver {
  /// Get‘s the structure of the underlying database. This will run whenever a
  /// service is starting up.
  fn get_structure() -> Result<Structure, Error>;

  /// Send a request to the database.
  fn request(request: Request) -> Result<Vec<Document>, Error>;

  /// Send a request to the database and guarantee it only ever effects a
  /// single document.
  fn request_one(request: Request) -> Result<Document, Error> {
    // TODO: default implementation which verifies that the request is only
    // affecting a single thing. This may be done by checking if the collection
    // primary key is specified in a filter.
    unimplemented!()
  }

  /// Send multiple requests at once. If one request fails, **all other
  /// requests must also fail**. If a driver author wishes they may also
  /// optimize these requests.
  fn requests(requests: Vec<Request>) -> Result<Vec<Vec<Document>>, Error> {
    // TODO: default implementation which just runs all the requests and returns
    // a result.
    unimplemented!()
  }
}
