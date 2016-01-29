//! Models the static structure of the database which Ardite is serving. All
//! Ardite services need some static representation of the database which
//! contains just enough information to get started. The types in this module
//! are that bootstrap static structure.

/// Represents a collection of documents. May contain whatever necessary data
/// to *locate* the collection. These collections must be simple enough to do
/// easy equality checks.
pub trait Collection {
  /// Get the public name of the collection. In a PostgreSQL database, this
  /// would be the table name without the schema name.
  fn get_name() -> String;
}
