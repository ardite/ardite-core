//! Format for defining the shape of data in an Ardite Schema Definition.
// TODO: Remove `pub` fields and add a DSL for building schemas.

use linear_map::LinearMap;
use regex::Regex;
use error::Error;
use query::Query;
use value::{Key, Pointer, Value};

lazy_static! {
  static ref INTEGER_RE: Regex = Regex::new(r"^\d+$").unwrap();
}

/// A schema detailing what the data received from the driver (or inserted
/// into the driver) should be. To describe this data we use a subset of
/// [JSON Schema][1]. The schema is a subset of JSON Schema for three reasons:
///
/// 1. Searchability throughout the schema. It must be possible to do
///    `schema.get("/hello/world")` which finds an object schema, for example
///    with the `hello` property and then another nested `world` property.
///    Nested schemas must be retrievable and this goal is not possible with
///    JSON Schema constructs like `oneOf`, `allOf`, `noneOf`, or `not` make it
///    difficult (if not impossible) to find a single schema for a pointer.
///
/// 2. Schema extension. In some areas, adding new properties to the schema
///    which don’t have strict validation purposes is useful. For example
///    `$type`, `$gen`, or `key`.
///
/// 3. Easy interoperability with a Rust enum. In Rust-land the best way to
///    represent a schema like this is with an enum. The official JSON
///    meta-schema and specification do not provide a format to easily
///    transform to a Rust enum format, therefore a custom definition is
///    required.
///
/// [1]: http://json-schema.org
#[derive(PartialEq, Clone, Debug)]
pub struct Schema {
  pub type_: SchemaType
}

impl Schema {
  /// Gets a nested schema at a certain point.
  pub fn get(&self, mut pointer: Pointer) -> Option<Self> {
    if pointer.is_empty() {
      Some(self.clone())
    } else {
      match self.type_ {
        SchemaType::Array{ref items} => {
          if INTEGER_RE.is_match(&pointer.remove(0)) {
            items.get(pointer)
          } else {
            None
          }
        },
        SchemaType::Object{ref properties,..} => {
          if let Some(schema) = properties.get(&pointer.remove(0)) {
            schema.get(pointer)
          } else {
            None
          }
        },
        SchemaType::Null |
        SchemaType::Boolean |
        SchemaType::Number{..} |
        SchemaType::String{..} |
        SchemaType::Enum(_) => None
      }
    }
  }

  /// Validates a query that a user would like to make on the database by
  /// comparing it to the schema. Mostly checks that all properties described
  /// in the query are accessible according to the schema.
  pub fn validate_query(&self, query: &Query) -> Result<(), Error> {
    static NO_PRIMITIVE_HINT: &'static str = "Try not querying specific properties of a primitive like `null` or `boolean`.";
    match (&self.type_, query) {
      (&SchemaType::Null, &Query::Keys(_)) => Err(Error::validation("Cannot deeply query null.", NO_PRIMITIVE_HINT)),
      (&SchemaType::Boolean, &Query::Keys(_)) => Err(Error::validation("Cannot deeply query a boolean.", NO_PRIMITIVE_HINT)),
      (&SchemaType::Number{..}, &Query::Keys(_)) => Err(Error::validation("Cannot deeply query a number.", NO_PRIMITIVE_HINT)),
      (&SchemaType::String{..}, &Query::Keys(_)) => Err(Error::validation("Cannot deeply query a string.", NO_PRIMITIVE_HINT)),
      (&SchemaType::Array{ref items}, &Query::Keys(ref query_properties)) => {
        match query_properties.keys().map(|key| {
          if INTEGER_RE.is_match(key) {
            items.validate_query(&query_properties.get(key).unwrap())
          } else {
            Err(Error::validation(format!("Cannot query non-integer \"{}\" array property.", key), "Only query integer array keys like 1, 2, and 3."))
          }
        }).find(|r| r.is_err()) {
          None => Ok(()),
          Some(error) => error
        }
      },
      (&SchemaType::Object{ref properties, ref additional_properties,..}, &Query::Keys(ref query_properties)) => {
        match query_properties.keys().map(|key| {
          if let Some(property_schema) = properties.get(key) {
            property_schema.validate_query(&query_properties.get(key).unwrap())
          } else if *additional_properties {
            Ok(())
          } else {
            Err(Error::validation(format!("Cannot query object property \"{}\".", key), "Query an object property that is defined in the schema."))
          }
        }).find(|r| r.is_err()) {
          None => Ok(()),
          Some(error) => error
        }
      },
      (&SchemaType::Enum(_), &Query::Keys(_)) => Err(Error::validation("Cannot deeply query an enum.", NO_PRIMITIVE_HINT)),
      (_, &Query::All) => Ok(())
    }
  }
}

/// SchemaType specific validations for values. A reference
/// on JSON Schema type-specific validations used in this enum may be found
/// [here][1].
///
/// [1]: http://spacetelescope.github.io/understanding-json-schema/reference/type.html
#[derive(PartialEq, Clone, Debug)]
pub enum SchemaType {
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
    /// Whether or not the minimum value should be included when validating.
    /// Default is `false`.
    exclusive_minimum: bool,
    /// The maximum value the number can be.
    maximum: Option<f64>,
    /// Whether or not the maximum value should be included when validating.
    /// Default is `false`.
    exclusive_maximum: bool
  },
  String {
    /// The mimimum length of characters in the string.
    min_length: Option<u64>,
    /// The maximum length of characters in the string.
    max_length: Option<u64>,
    /// A regular expression pattern to validate the string against.
    pattern: Option<Regex>
  },
  /// Represents a set of any type.
  Array {
    /// A schema which all items in the array must match.
    items: Box<Schema>
  },
  /// Represents a set of key/value pairs.
  Object {
    /// Schemas associated to the object properties.
    properties: LinearMap<Key, Schema>,
    /// Properties that are required to be in the object.
    required: Vec<Key>,
    /// Whether or not there may be extra properties outside of the ones
    /// defined by the properties map.
    additional_properties: bool
  },
  /// Represents a value which *must* be one of the defined values. An enum is
  /// considered a primitive type as if it is a single value is a higher order
  /// type, no variation is allowed.
  Enum(Vec<Value>)
}

#[cfg(test)]
mod tests {
  use schema::{Schema, SchemaType};
  use query::Query;

  #[test]
  fn test_get_primitive() {
    assert_eq!(Schema { type_: SchemaType::Null }.get(point![]).unwrap(), Schema { type_: SchemaType::Null });
    assert!(Schema { type_: SchemaType::Null }.get(point!["hello"]).is_none());
    assert_eq!(Schema { type_: SchemaType::Boolean }.get(point![]).unwrap(), Schema { type_: SchemaType::Boolean });
    assert!(Schema { type_: SchemaType::Boolean }.get(point!["hello"]).is_none());
    assert!(Schema {
      type_: SchemaType::Number {
        multiple_of: None,
        minimum: None,
        exclusive_minimum: false,
        maximum: None,
        exclusive_maximum: false
      }
    }.get(point!["hello"]).is_none());
    assert!(Schema {
      type_: SchemaType::String {
        min_length: None,
        max_length: None,
        pattern: None
      }
    }.get(point!["hello"]).is_none());
  }

  #[test]
  fn test_get_array() {
    let array_none = Schema {
      type_: SchemaType::Array {
        items: Box::new(Schema { type_: SchemaType::Null })
      }
    };
    let array_bool = Schema {
      type_: SchemaType::Array {
        items: Box::new(Schema { type_: SchemaType::Boolean })
      }
    };
    assert_eq!(array_none.get(point!["1"]).unwrap(), Schema { type_: SchemaType::Null });
    assert!(array_none.get(point!["asd"]).is_none());
    assert_eq!(array_bool.get(point!["1"]).unwrap(), Schema { type_: SchemaType::Boolean });
    assert_eq!(array_bool.get(point!["9999999"]).unwrap(), Schema { type_: SchemaType::Boolean });
    assert!(array_bool.get(point!["asd"]).is_none());
  }

  #[test]
  fn test_get_object() {
    let object = Schema {
      type_: SchemaType::Object {
        required: vec![],
        additional_properties: false,
        properties: linear_map! {
          str!("hello") => Schema { type_: SchemaType::Boolean },
          str!("world") => Schema { type_: SchemaType::Boolean },
          str!("5") => Schema { type_: SchemaType::Boolean },
          str!("goodbye") => Schema {
            type_: SchemaType::Object {
              required: vec![],
              additional_properties: false,
              properties: linear_map! {
                str!("hello") => Schema { type_: SchemaType::Boolean },
                str!("world") => Schema { type_: SchemaType::Boolean }
              }
            }
          }
        }
      }
    };
    assert!(object.get(point!["yo"]).is_none());
    assert_eq!(object.get(point!["hello"]).unwrap(), Schema { type_: SchemaType::Boolean });
    assert_eq!(object.get(point!["goodbye", "world"]).unwrap(), Schema { type_: SchemaType::Boolean });
    assert!(object.get(point!["goodbye", "yo"]).is_none());
  }

  #[test]
  fn test_query_primitive() {
    assert!(Schema { type_: SchemaType::Null }.validate_query(&Query::All).is_ok());
    let obj_query = Query::Keys(linear_map! {});
    Schema { type_: SchemaType::Null }.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema { type_: SchemaType::Boolean }.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema {
      type_: SchemaType::Number {
        multiple_of: None,
        minimum: None,
        exclusive_minimum: false,
        maximum: None,
        exclusive_maximum: false
      }
    }.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema {
      type_: SchemaType::String {
        min_length: None,
        max_length: None,
        pattern: None
      }
    }.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema { type_: SchemaType::Enum(vec![
      vbool!(true),
      vbool!(false)
    ]) }.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
  }

  #[test]
  fn test_query_array() {
    let array_null = Schema {
      type_: SchemaType::Array {
        items: Box::new(Schema { type_: SchemaType::Null })
      }
    };
    let array_bool = Schema {
      type_: SchemaType::Array {
        items: Box::new(Schema { type_: SchemaType::Boolean })
      }
    };
    assert!(array_null.validate_query(&Query::All).is_ok());
    assert!(array_null.validate_query(&Query::Keys(linear_map! {
      str!("1") => Query::All
    })).is_ok());
    array_null.validate_query(&Query::Keys(linear_map! {
      str!("1") => Query::Keys(linear_map! {})
    })).unwrap_err().assert_message(r"Cannot deeply query null\.");
    assert!(array_bool.validate_query(&Query::Keys(linear_map! {
      str!("1") => Query::All,
      str!("2") => Query::All,
      str!("3") => Query::All,
      str!("50") => Query::All,
      str!("9999999999999") => Query::All
    })).is_ok());
    array_null.validate_query(&Query::Keys(linear_map! {
      str!("hello") => Query::All
    })).unwrap_err().assert_message("non-integer \"hello\"");
    array_bool.validate_query(&Query::Keys(linear_map! {
      str!("1") => Query::Keys(linear_map! {})
    })).unwrap_err().assert_message(r"Cannot deeply query a boolean\.");
  }

  #[test]
  fn test_query_object() {
    let object = Schema {
      type_: SchemaType::Object {
        required: vec![],
        additional_properties: false,
        properties: linear_map! {
          str!("hello") => Schema { type_: SchemaType::Boolean },
          str!("world") => Schema { type_: SchemaType::Boolean },
          str!("5") => Schema { type_: SchemaType::Boolean },
          str!("goodbye") => Schema {
            type_: SchemaType::Object {
              required: vec![],
              additional_properties: false,
              properties: linear_map! {
                str!("hello") => Schema { type_: SchemaType::Boolean },
                str!("world") => Schema { type_: SchemaType::Boolean }
              }
            }
          }
        }
      }
    };
    let object_additional = Schema {
      type_: SchemaType::Object {
        required: vec![],
        additional_properties: true,
        properties: linear_map! {
          str!("hello") => Schema { type_: SchemaType::Boolean },
          str!("world") => Schema { type_: SchemaType::Boolean }
        }
      }
    };
    assert!(object.validate_query(&Query::Keys(linear_map! {
      str!("world") => Query::All,
      str!("5") => Query::All,
      str!("goodbye") => Query::All
    })).is_ok());
    object.validate_query(&Query::Keys(linear_map! {
      str!("hello") => Query::All,
      str!("moon") => Query::All
    })).unwrap_err().assert_message("Cannot query object property \"moon\".");
    object.validate_query(&Query::Keys(linear_map! {
      str!("hello") => Query::Keys(linear_map! {})
    })).unwrap_err().assert_message(r"Cannot deeply query a boolean\.");
    assert!(object.validate_query(&Query::Keys(linear_map! {
      str!("goodbye") => Query::Keys(linear_map! {
        str!("hello") => Query::All
      })
    })).is_ok());
    object.validate_query(&Query::Keys(linear_map! {
      str!("goodbye") => Query::Keys(linear_map! {
        str!("hello") => Query::Keys(linear_map! {})
      })
    })).unwrap_err().assert_message(r"Cannot deeply query a boolean\.");
    assert!(object_additional.validate_query(&Query::Keys(linear_map! {
      str!("world") => Query::All,
      str!("5") => Query::All,
      str!("goodbye") => Query::All,
      str!("moon") => Query::All
    })).is_ok());
  }
}
