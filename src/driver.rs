//! This module contains the common driver code. Specific implementations for
//! different databases exist elsewhere.

use error::Error;
use values::*;
use structure::*;
use request::Request;

/// The trait to be implemented by all drivers.
pub trait Driver {
  /// Get‘s the structure of the underlying database. This will run whenever a
  /// service is starting up.
  fn get_structure() -> Result<Structure, Error>;

  /// The create request. Designed after a [SQL `INSERT` statmenet][1].
  ///
  /// Returns all of the values created in the database with any automatically
  /// generated properties. This is useful if you need the id of the just
  /// inserted value.
  ///
  /// [1]: http://www.postgresql.org/docs/current/static/sql-insert.html
  fn create<I>(&collection: Collection,
               values: Vec<Value>,
               returning: Vec<Pointer> /* = all_pointer */)
               -> Result<I, Error>
               where I: Iterator<Item=Value>;

  // TODO: NEEDS A GOOD RANGE IMPLEMENTATION.
  // /// The read request. Designed after a [SQL `SELECT` statement][1]. Also, a
  // /// read request (after much thought) is not recursive/join capable. Instead
  // /// join decomposition should be preferred. For more information, see “[big
  // /// query v. small query][2]”.
  // ///
  // /// Must return an iterator of the set of values described in this request.
  // ///
  // /// [1]: http://www.postgresql.org/docs/current/static/sql-select.html
  // /// [2]: http://dba.stackexchange.com/questions/76973
  // fn read<I>(&collection: Collection,
  //            filter: Option<Filter>,
  //            range: Range,
  //            order: Vec<Ordering>,
  //            returning: Vec<Pointer> /* = all_pointer */)
  //            -> Result<I, Error>
  //            where I: Iterator<Item=Value>;

  /// The update request. Designed after a [SQL `UPDATE` statement][1].
  ///
  /// Must return an array of all the documents that were updated with this
  /// request.
  ///
  /// [1]: http://www.postgresql.org/docs/current/static/sql-update.html
  fn update<I>(&collection: Collection,
               filter: Option<Filter>,
               patches: Vec<Patch>,
               returning: Vec<Pointer> /* = all_pointer */)
               -> Result<I, Error>
               where I: Iterator<Item=Value>;

  /// The delete request. Desinged after a [SQL `DELETE` statement][1].
  ///
  /// Must return an array of all documents that were deleted with this
  /// request.
  ///
  /// [1]: http://www.postgresql.org/docs/current/static/sql-delete.html
  fn delete<I>(&collection: Collection,
               filter: Option<Filter>,
               returning: Vec<Pointer> /* = all_pointer */)
               -> Result<I, Error>
               where I: Iterator<Item=Value>;
}
