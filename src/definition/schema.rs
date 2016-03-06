//! Format for defining the shape of data in an Ardite Schema Definition.

use linear_map::LinearMap;
use regex::Regex;
use error::Error;
use query::{Query, Selection};
use value::{Key, Pointer, Value};

lazy_static! {
  static ref INTEGER_RE: Regex = Regex::new(r"^\d+$").unwrap();
}

/// A schema detailing what the data received from the driver (or inserted
/// into the driver) should be. A subset of [JSON Schema][1]. A reference
/// on JSON Schema type-specific validations used in this enum may be found
/// [here][2].
///
/// The schema is a subset of JSON Schema for three reasons:
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
/// [2]: http://spacetelescope.github.io/understanding-json-schema/reference/type.html
#[derive(PartialEq, Clone, Debug)]
pub enum Schema {
  /// There is no schema. No validations should occur. Does not represent the
  /// abscense of any value, only represents that a schema does not define the
  /// data structure at this point.
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

impl Schema {
  /// Gets a nested schema at a certain point.
  pub fn get(&self, mut pointer: Pointer) -> Self {
    if pointer.len() == 0 {
      self.clone()
    } else {
      match self {
        &Schema::None => Schema::None,
        &Schema::Null => Schema::None,
        &Schema::Boolean => Schema::None,
        &Schema::Number{..} => Schema::None,
        &Schema::String{..} => Schema::None,
        &Schema::Array{ref items} => {
          if INTEGER_RE.is_match(&pointer.remove(0)) {
            items.get(pointer)
          } else {
            Schema::None
          }
        },
        &Schema::Object{ref properties,..} => {
          if let Some(schema) = properties.get(&pointer.remove(0)) {
            schema.get(pointer)
          } else {
            Schema::None
          }
        },
        &Schema::Enum(_) => Schema::None
      }
    }
  }
  
  /// Validates a query that a user would like to make on the database by
  /// comparing it to the schema. Mostly checks that all properties described
  /// in the query are accessible according to the schema.
  pub fn validate_query(&self, query: &Query) -> Result<(), Error> {
    static NO_PRIMITIVE_HINT: &'static str = "Try not querying specific properties of a primitive like `null` or `boolean`.";
    match (self, query) {
      // No schema describes these values, its the wild west. Go crazy query.
      // `Schema::None` does not represent the absence of value, just the
      // absence of validation.
      (&Schema::None, _) => Ok(()),
      (&Schema::Null, &Query::Value) => Ok(()),
      (&Schema::Null, &Query::Object(_)) => Err(Error::validation("Cannot deeply query null.", NO_PRIMITIVE_HINT)),
      (&Schema::Boolean, &Query::Value) => Ok(()),
      (&Schema::Boolean, &Query::Object(_)) => Err(Error::validation("Cannot deeply query a boolean.", NO_PRIMITIVE_HINT)),
      (&Schema::Number{..}, &Query::Value) => Ok(()),
      (&Schema::Number{..}, &Query::Object(_)) => Err(Error::validation("Cannot deeply query a number.", NO_PRIMITIVE_HINT)),
      (&Schema::String{..}, &Query::Value) => Ok(()),
      (&Schema::String{..}, &Query::Object(_)) => Err(Error::validation("Cannot deeply query a string.", NO_PRIMITIVE_HINT)),
      (&Schema::Array{..}, &Query::Value) => Ok(()),
      (&Schema::Array{ref items}, &Query::Object(ref query_properties)) => {
        match query_properties.keys().map(|selection| match selection {
          &Selection::Key(ref key) => {
            if !INTEGER_RE.is_match(key) {
              Err(Error::validation(format!("Cannot query non-integer \"{}\" array property.", key), "Only query integer array keys like 1, 2, and 3."))
            } else {
              items.validate_query(&query_properties.get(selection).unwrap())
            }
          }
        }).find(|r| r.is_err()) {
          None => Ok(()),
          Some(error) => error
        }
      },
      (&Schema::Object{..}, &Query::Value) => Ok(()),
      (&Schema::Object{ref properties, ref additional_properties,..}, &Query::Object(ref query_properties)) => {
        match query_properties.keys().map(|selection| match selection {
          &Selection::Key(ref key) => {
            if let Some(property_schema) = properties.get(key) {
              property_schema.validate_query(&query_properties.get(selection).unwrap())
            } else if !additional_properties {
              Err(Error::validation(format!("Cannot query object property \"{}\".", key), "Query an object property that is defined in the schema."))
            } else {
              Ok(())
            }
          }
        }).find(|r| r.is_err()) {
          None => Ok(()),
          Some(error) => error
        }
      },
      (&Schema::Enum(_), &Query::Value) => Ok(()),
      (&Schema::Enum(_), &Query::Object(_)) => Err(Error::validation("Cannot deeply query an enum.", NO_PRIMITIVE_HINT))
    }
  }
}

#[cfg(test)]
mod tests {
  use definition::schema::Schema;
  use query::{Query, Selection};
  
  #[test]
  fn test_get_primitive() {
    assert_eq!(Schema::None.get(point![]), Schema::None);
    assert_eq!(Schema::None.get(point!["hello"]), Schema::None);
    assert_eq!(Schema::Boolean.get(point![]), Schema::Boolean);
    assert_eq!(Schema::Boolean.get(point!["hello"]), Schema::None);
    assert_eq!(Schema::Number {
      multiple_of: None,
      minimum: None,
      exclusive_minimum: false,
      maximum: None,
      exclusive_maximum: false
    }.get(point!["hello"]), Schema::None);
    assert_eq!(Schema::String {
      min_length: None,
      max_length: None,
      pattern: None
    }.get(point!["hello"]), Schema::None);
  }
  
  #[test]
  fn test_get_array() {
    let array_none = Schema::Array {
      items: Box::new(Schema::None)
    };
    let array_bool = Schema::Array {
      items: Box::new(Schema::Boolean)
    };
    assert_eq!(array_none.get(point!["1"]), Schema::None);
    assert_eq!(array_none.get(point!["asd"]), Schema::None);
    assert_eq!(array_bool.get(point!["1"]), Schema::Boolean);
    assert_eq!(array_bool.get(point!["9999999"]), Schema::Boolean);
    assert_eq!(array_bool.get(point!["asd"]), Schema::None);
  }
  
  #[test]
  fn test_get_object() {
    let object = Schema::Object {
      required: vec![],
      additional_properties: false,
      properties: linear_map! {
        String::from("hello") => Schema::Boolean,
        String::from("world") => Schema::Boolean,
        String::from("5") => Schema::Boolean,
        String::from("goodbye") => Schema::Object {
          required: vec![],
          additional_properties: false,
          properties: linear_map! {
            String::from("hello") => Schema::Boolean,
            String::from("world") => Schema::Boolean
          }
        }
      }
    }; 
    assert_eq!(object.get(point!["yo"]), Schema::None);
    assert_eq!(object.get(point!["hello"]), Schema::Boolean);
    assert_eq!(object.get(point!["goodbye", "world"]), Schema::Boolean);
    assert_eq!(object.get(point!["goodbye", "yo"]), Schema::None);
  }

  #[test]
  fn test_query_none() {
    assert_eq!(Schema::None.validate_query(&Query::Value).is_ok(), true);
    assert_eq!(Schema::None.validate_query(&Query::Object(linear_map! {
      Selection::Key("s@#f&/Ij)82h(;pa0]".to_string()) => Query::Value,
      Selection::Key("123".to_string()) => Query::Value,
      Selection::Key("hello".to_string()) => Query::Value,
      Selection::Key("nested".to_string()) => Query::Object(linear_map! {
        Selection::Key("yo".to_string()) => Query::Value
      })
    })).is_ok(), true);
  }

  #[test]
  fn test_query_primitive() {
    assert!(Schema::Null.validate_query(&Query::Value).is_ok());
    let obj_query = Query::Object(linear_map! {});
    Schema::Null.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema::Boolean.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema::Number {
      multiple_of: None,
      minimum: None,
      exclusive_minimum: false,
      maximum: None,
      exclusive_maximum: false
    }.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema::String {
      min_length: None,
      max_length: None,
      pattern: None
    }.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema::Enum(vec![
      vbool!(true),
      vbool!(false)
    ]).validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
  }

  #[test]
  fn test_query_array() {
    let array_none = Schema::Array {
      items: Box::new(Schema::None)
    };
    let array_bool = Schema::Array {
      items: Box::new(Schema::Boolean)
    };
    assert!(array_none.validate_query(&Query::Value).is_ok());
    assert!(array_none.validate_query(&Query::Object(linear_map! {
      Selection::Key("1".to_string()) => Query::Value
    })).is_ok());
    assert!(array_none.validate_query(&Query::Object(linear_map! {
      Selection::Key("1".to_string()) => Query::Object(linear_map! {})
    })).is_ok());
    assert!(array_bool.validate_query(&Query::Object(linear_map! {
      Selection::Key("1".to_string()) => Query::Value,
      Selection::Key("2".to_string()) => Query::Value,
      Selection::Key("3".to_string()) => Query::Value,
      Selection::Key("50".to_string()) => Query::Value,
      Selection::Key("999999999999999".to_string()) => Query::Value
    })).is_ok());
    array_none.validate_query(&Query::Object(linear_map! {
      Selection::Key("hello".to_string()) => Query::Value
    })).unwrap_err().assert_message("non-integer \"hello\"");
    array_bool.validate_query(&Query::Object(linear_map! {
      Selection::Key("1".to_string()) => Query::Object(linear_map! {})
    })).unwrap_err().assert_message(r"Cannot deeply query a boolean\.");
  }

  #[test]
  fn test_query_object() {
    let object = Schema::Object {
      required: vec![],
      additional_properties: false,
      properties: linear_map! {
        String::from("hello") => Schema::Boolean,
        String::from("world") => Schema::Boolean,
        String::from("5") => Schema::Boolean,
        String::from("goodbye") => Schema::Object {
          required: vec![],
          additional_properties: false,
          properties: linear_map! {
            String::from("hello") => Schema::Boolean,
            String::from("world") => Schema::Boolean
          }
        }
      }
    }; 
    let object_additional = Schema::Object {
      required: vec![],
      additional_properties: true,
      properties: linear_map! {
        String::from("hello") => Schema::Boolean,
        String::from("world") => Schema::Boolean
      }
    };
    assert!(object.validate_query(&Query::Object(linear_map! {
      Selection::Key("world".to_string()) => Query::Value,
      Selection::Key("5".to_string()) => Query::Value,
      Selection::Key("goodbye".to_string()) => Query::Value
    })).is_ok());
    object.validate_query(&Query::Object(linear_map! {
      Selection::Key("hello".to_string()) => Query::Value,
      Selection::Key("moon".to_string()) => Query::Value
    })).unwrap_err().assert_message("Cannot query object property \"moon\".");
    object.validate_query(&Query::Object(linear_map! {
      Selection::Key("hello".to_string()) => Query::Object(linear_map! {})
    })).unwrap_err().assert_message(r"Cannot deeply query a boolean\.");
    assert!(object.validate_query(&Query::Object(linear_map! {
      Selection::Key("goodbye".to_string()) => Query::Object(linear_map! {
        Selection::Key("hello".to_string()) => Query::Value
      })
    })).is_ok());
    object.validate_query(&Query::Object(linear_map! {
      Selection::Key("goodbye".to_string()) => Query::Object(linear_map! {
        Selection::Key("hello".to_string()) => Query::Object(linear_map! {})
      })
    })).unwrap_err().assert_message(r"Cannot deeply query a boolean\.");
    assert!(object_additional.validate_query(&Query::Object(linear_map! {
      Selection::Key("world".to_string()) => Query::Value,
      Selection::Key("5".to_string()) => Query::Value,
      Selection::Key("goodvye".to_string()) => Query::Value,
      Selection::Key("moon".to_string()) => Query::Value
    })).is_ok());
  }
}
