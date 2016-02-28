//! Types representing for data which will be retrieved from the driver.
//! Currently this data is expected to look like a JSON object but this may be
//! changed in the future. Driver authors must cast the data they retrieve from
//! the driver to these types.

use linear_map::LinearMap;

/// The atomic level of a pointer.
pub type Key = String;

/// Represents a JSON pointer to a document property.
pub type Pointer = Vec<Key>;

/// Various value types. Based on types in the [JSON standard][1] (see section
/// 5).
///
/// [1]: http://ecma-international.org/publications/files/ECMA-ST/ECMA-404.pdf
#[derive(PartialEq, Debug)]
pub enum Value {
  /// The abscense of any value.
  Null,
  /// True or false.
  Boolean(bool),
  /// A numeric value, float, integer, whatever.
  Number(f64),
  /// A list of characters.
  String(String),
  /// A map of key/value pairs. Stored as a vector of tuples for performance
  /// and to maintain key ordering.
  Object(LinearMap<Key, Value>),
  /// A list of values. Just a value, but using *only* integer keys.
  Array(Vec<Value>)
}
