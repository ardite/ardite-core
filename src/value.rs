//! Types representing for data which will be retrieved from the database.
//! Currently this data is expected to look like a JSON object but this may be
//! changed in the future. Driver authors must cast the data they retrieve from
//! the database to these types.

use std::collections::BTreeMap;
use structure::Collection;

/// The atomic level of a pointer.
pub type Key = String;

/// Represents a JSON pointer to a document property.
pub type Pointer = Vec<Key>;

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
  Object(BTreeMap<Key, Value>)
}

/// A schema detailing what the data received from the database (or inserted
/// into the database) should be. Inspired after [JSON Schema][1]. A JSON
/// Schema general keyword reference may be found [here][2].
///
/// [1]: http://json-schema.org
/// [2]: http://spacetelescope.github.io/understanding-json-schema/reference/generic.html
pub struct Schema {
  /// The types an object may be. Contains some custom validation information
  /// for each type.
  types: Vec<SchemaType>,
  /// The default value of this part of the schema.
  default: Option<Value>,
  /// Equivelent to JSON Schema‘s `enum`. If not `None`, the value must be
  /// exactly equal to one of these.
  one_of: Option<Vec<Value>>
}

/// Type specific schema validations. A reference on JSON Schema type-specific
/// validations may be found [here][1].
///
/// [1]: http://spacetelescope.github.io/understanding-json-schema/reference/type.html
pub enum SchemaType {
  /// The absence of any value is also represented as the absence of any type.
  Null,
  /// Represents a binary true/false value.
  Boolean,
  /// Represents a numeric type.
  Number {
    /// Forces the number to be a multiple of another. This helps specifying
    /// integers if this value is `Some(1)`.
    multiple_of: Option<f32>,
    /// The minimum value the number can be.
    minimum: Option<f64>,
    /// The maximum value the number can be.
    maximum: Option<f64>
  },
  /// Represents a set of characters type.
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
    item_schema: Option<Schema>
  },
  /// Represents a set of key/value pairs.
  Object {
    /// Schemas associated to the object keys.
    key_schemas: BTreeMap<Key, Schema>,
    /// Keys that are required to be present in the object.
    required_keys: Vec<Key>,
    /// Whether or not extra keys may be present in the object.
    additional_keys: bool
  }
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
  And(Vec<Filter>),
  /// Combine multiple filters with an “or” operator.
  Or(Vec<Filter>),
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

// TODO: Find a more Rust idiomatic solution for ranges.
pub struct Range(Option<u32>, Option<u32>);
