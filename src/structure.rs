//! Models the static structure of the database which Ardite is serving. All
//! Ardite services need some static representation of the database which
//! contains just enough information to get started. The types in this module
//! are that bootstrap static structure.

use document::Property;

/// Represents a collection of documents.
pub struct Collection {
  /// The outward facing name of the collection. In a relational database
  /// with schemas (for example), this should be the table name, not including
  /// the schema name.
  name: String,

  /// The primary key property required for all documents in a collection.
  /// For each document in the collection the value for this property should be
  /// able to uniquely identify the document within the collection. Ardite does
  /// not support composite primary keys ([more information][1]).
  ///
  /// [1]: http://stackoverflow.com/questions/1383062/composite-primary-key
  primary_key: Property,

  /// Keys of properties which are optional in all documents in this collection
  /// linked to the primary key of an external collection. This allows for
  /// relationship detection.
  foreign_keys: Vec<(Property, Collection)>
}

/// Represents the entire database structure.
pub struct Structure {
  /// All of the collections in the database which are accessible in some way
  /// via Ardite services.
  collections: Vec<Collection>
}
