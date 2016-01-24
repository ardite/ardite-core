//! Types representing for data which will be retrieved from the database.
//! Currently this data is expected to look like a JSON object but this may be
//! changed in the future. Driver authors must cast the data they retrieve from
//! the database to these types.

use std::collections::BTreeMap;
use structure::Collection;

/// Represents a JSON pointer to a document property.
pub type Pointer = Vec<String>;

/// Various value types. Based on types in the [JSON standard][1] (see section
/// 5).
///
/// [1]: http://ecma-international.org/publications/files/ECMA-ST/ECMA-404.pdf
pub enum Value {
  /// The abscense of any value.
  Null,
  /// True or false.
  Boolean(bool),
  /// A numeric value, float, integer, whatever.
  Number(f64),
  /// A list of characters.
  String(String),
  /// A list of values.
  Array(Vec<Value>),
  /// A map of key/value pairs.
  Object(BTreeMap<String, Value>),
  /// A reference to a document in another collection.
  Reference(Collection, Box<Value>)
}

/// Different database collection property updates.
pub enum Patch {
  /// Set a property to a new value.
  Set(Pointer, Value),
  /// Reset a property to its default value.
  Reset(Pointer),
  /// Remove a property from the database entirely.
  Remove(Pointer)
}

/// A recursive filter condition for a `Value`.
pub enum Filter {
  /// Combine multiple filters with an “and” operator.
  And(Vec<Box<Filter>>),
  /// Combine multiple filters with an “or” operator.
  Or(Vec<Box<Filter>>),
  /// Inverts the filter.
  Not(Box<Filter>),
  /// The basic condition of a filter.
  Condition(Pointer, FilterCondition)
}

pub enum FilterCondition {
  Equal(Value),
  OneOf(Vec<Value>),
  GreaterThan(Value),
  LessThan(Value)
}

/// A single way in which to order a collection of documents.
pub struct Ordering(Pointer, OrderDirection);

pub enum OrderDirection {
  Ascending,
  Descending
}
