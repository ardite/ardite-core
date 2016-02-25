//! Types representing for data which will be retrieved from the driver.
//! Currently this data is expected to look like a JSON object but this may be
//! changed in the future. Driver authors must cast the data they retrieve from
//! the driver to these types.

use std::collections::BTreeMap;

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
  /// A list of values.
  Array(Vec<Value>),
  /// A map of key/value pairs.
  Object(BTreeMap<Key, Value>),
  /// References another value somewhere else in the object tree.
  Ref(Pointer)
}

/// A schema detailing what the data received from the driver (or inserted
/// into the driver) should be. Inspired after [JSON Schema][1]. A reference
/// on JSON Schema type-specific validations used in this enum may be found
/// [here][2].
///
/// [1]: http://json-schema.org
/// [2]: http://spacetelescope.github.io/understanding-json-schema/reference/type.html
pub enum Schema {
  /// There is no schema. No validations should occur.
  None,
  /// Represents the absence of any value.
  Null,
  /// Represents a binary true/false value.
  Boolean,
  /// Represents a numeric type.
  Number {
    /// Forces the number to be a multiple of another. This helps in specifying
    /// integers if this value is `Some(1)` for example.
    multiple_of: Option<f32>,
    /// The minimum value the number can be.
    minimum: Option<f64>,
    /// The maximum value the number can be.
    maximum: Option<f64>
  },
  String {
    /// The mimimum length of characters in the string.
    min_length: Option<u64>,
    /// The maximum length of characters in the string.
    max_length: Option<u64>,
    /// A regular expression pattern to validate the string against.
    // TODO: Use a regex crate.
    pattern: Option<String>
  },
  /// Represents a set of any type.
  Array {
    /// A schema which all items in the array must match.
    items: Box<Schema>
  },
  /// Represents any tuple of values.
  Tuple {
    /// Schemas which each tuple value (in the same place) must comply with.
    items: Vec<Schema>,
    /// Whether or not there can be more items in the tuple.
    additional_items: bool
  },
  /// Represents a set of key/value pairs.
  Object {
    /// Schemas associated to the object properties.
    properties: BTreeMap<Key, Schema>,
    /// Properties that are required to be in the object.
    required: Vec<Key>,
    /// Whether or not there may be extra properties outside of the ones
    /// defined by the properties map.
    additional_properties: bool
  },
  /// Represents a value which *must* be one of the defined values.
  Enum(Vec<Value>)
}
