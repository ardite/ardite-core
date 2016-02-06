//! This module contains the common driver code. Specific implementations for
//! different databases exist elsewhere.

use error::Error;
use value::*;

pub trait Driver {
  /// The driver specific collection type.
  type Collection: Collection;

  /// Connects to a database and returns a driver instance. After calling this
  /// the driver is ready to roll!
  fn connect(connection_url: &str) -> Self;

  /// Get all collections in the database.
  fn get_collections(&self) -> Result<Vec<Self::Collection>, Error>;
}

pub trait Collection {
  /// Gets the name of the collection.
  fn get_name(&self) -> String;

  /// Creates a single record in the database. Returns the new record with its
  /// *assigned* identifier.
  fn create_one(&self, value: Value) -> Result<Record, Error>;

  /// Creates many records in the database. Returns a stream of the new
  /// records with their *assigned* identifiers.
  fn create<V, I>(&self, values: V) -> Result<I, Error> where V: Iterator<Item=Value>, I: Iterator<Item=Record> {
    Err(Error::unimplemented("Cannot perform a batch insert."))
  }

  /// Reads a single record from the database. Returns the record that was
  /// read.
  fn read_one(&self, id: Identifier) -> Result<Record, Error>;

  /// Reads many records from the database. Returns a stream of the read
  /// records.
  fn read<I>(&self, filter: Vec<Condition>) -> Result<I, Error> where I: Iterator<Item=Record>;

  /// Updates a single record in the database. Returns the updated record.
  fn update_one(&self, id: Identifier, patches: Vec<(Pointer, Value)>) -> Result<Record, Error>;

  /// Updates many records in the database. Returns a stream of the updated
  /// records.
  fn update<I>(&self, filter: Vec<Condition>, patches: Vec<(Pointer, Value)>) -> Result<I, Error> where I: Iterator<Item=Record> {
    Err(Error::unimplemented("Cannot perform a batch update."))
  }

  /// Removes a single record from the database. Returns the deleted record
  /// with its identifier.
  fn delete_one(&self, id: Identifier) -> Result<Record, Error>;

  /// Removes many records from the database. Returns a stream of the deleted
  /// records.
  fn delete<I>(&self, filter: Vec<Condition>) -> Result<I, Error> where I: Iterator<Item=Record> {
    Err(Error::unimplemented("Cannot perform a batch deletion."))
  }

  /// Gets the schema for the collection. By default no schema is returned.
  fn get_schema(&self) -> Result<Schema, Error> {
    Ok(Schema::None)
  }
}
