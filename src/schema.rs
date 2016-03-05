use linear_map::LinearMap;
use regex::Regex;
use error::Error;
use query::Query;
use value::{Key, Value};

/// A schema detailing what the data received from the driver (or inserted
/// into the driver) should be. Inspired after [JSON Schema][1]. A reference
/// on JSON Schema type-specific validations used in this enum may be found
/// [here][2].
///
/// [1]: http://json-schema.org
/// [2]: http://spacetelescope.github.io/understanding-json-schema/reference/type.html
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
    // TODO: Use a regex crate.
    pattern: Option<String>
  },
  /// Represents a set of any type.
  Array {
    /// A schema which all items in the array must match.
    items: Box<Schema>,
    /// The minimum number of items in the array.
    min_items: Option<u64>,
    /// The maximum number of items in the array.
    max_items: Option<u64>
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
  /// Validates a query that a user would like to make on the database by
  /// comparing it to the schema. Mostly checks that all properties described
  /// in the query are accessible according to the schema.
  pub fn validate_query(&self, query: &Query) -> Result<(), Error> {
    static NO_PRIMITIVE_HINT: &'static str = "Try not querying specific properties of a primitive like `null` or `boolean`.";
    lazy_static! { static ref INTEGER_RE: Regex = Regex::new(r"^\d+$").unwrap(); }

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
      (&Schema::Array{ref items,..}, &Query::Object(ref query_properties)) => {
        match query_properties.keys().map(|key| {
          if !INTEGER_RE.is_match(key) {
            Err(Error::validation(format!("Cannot query non-integer \"{}\" array property.", key), "Only query integer array keys like 1, 2, and 3."))
          } else {
            items.validate_query(&query_properties[key])
          }
        }).find(|r| r.is_err()) {
          None => Ok(()),
          Some(error) => error
        }
      },
      (&Schema::Tuple{..}, &Query::Value) => Ok(()),
      (&Schema::Tuple{ref items, ref additional_items}, &Query::Object(ref query_properties)) => {
        match query_properties.keys().map(|key| {
          if !INTEGER_RE.is_match(key) {
            Err(Error::validation(format!("Cannot query non-integer \"{}\" array property.", key), "Only query integer array keys like 1, 2, and 3."))
          } else if let Some(item_schema) = items.get(key.parse::<usize>().unwrap()) {
            item_schema.validate_query(&query_properties[key])
          } else if !additional_items {
            Err(Error::validation(format!("Tuple has only {} values. Can’t query the index {}.", items.len(), key), format!("Query a key less than or equal to {}.", items.len() - 1)))
          } else {
            Ok(())
          }
        }).find(|r| r.is_err()) {
          None => Ok(()),
          Some(error) => error
        }
      },
      (&Schema::Object{..}, &Query::Value) => Ok(()),
      (&Schema::Object{ref properties, ref additional_properties,..}, &Query::Object(ref query_properties)) => {
        match query_properties.keys().map(|key| {
          if let Some(property_schema) = properties.get(key) {
            property_schema.validate_query(&query_properties[key])
          } else if !additional_properties {
            Err(Error::validation(format!("Cannot query object property \"{}\".", key), "Query an object property that is defined in the schema."))
          } else {
            Ok(())
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
  use linear_map::LinearMap;
  use schema::Schema;

  #[test]
  fn test_query_none() {
    assert_eq!(Schema::None.validate_query(&qvalue!()).is_ok(), true);
    assert_eq!(Schema::None.validate_query(&qobject!{
      "s@#f&/Ij)82h(;pa0]" => qvalue!(),
      "123" => qvalue!(),
      "hello" => qvalue!(),
      "nested" => qobject!{
        "yo" => qvalue!()
      }
    }).is_ok(), true);
  }

  #[test]
  fn test_query_primitive() {
    assert!(Schema::Null.validate_query(&qvalue!()).is_ok());
    let obj_query = qobject!{};
    Schema::Null.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema::Boolean.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema::Number{
      multiple_of: None,
      minimum: None,
      exclusive_minimum: false,
      maximum: None,
      exclusive_maximum: false
    }.validate_query(&obj_query).unwrap_err().assert_message(r"deeply query");
    Schema::String{
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
      items: Box::new(Schema::None),
      min_items: None,
      max_items: None
    };
    let array_bool = Schema::Array {
      items: Box::new(Schema::Boolean),
      min_items: None,
      max_items: None
    };
    assert!(array_none.validate_query(&qvalue!()).is_ok());
    assert!(array_none.validate_query(&qobject!{
      "1" => qvalue!()
    }).is_ok());
    assert!(array_none.validate_query(&qobject!{
      "1" => qobject!{}
    }).is_ok());
    assert!(array_bool.validate_query(&qobject!{
      "1" => qvalue!(),
      "2" => qvalue!(),
      "3" => qvalue!(),
      "50" => qvalue!(),
      "999999999999999" => qvalue!()
    }).is_ok());
    array_none.validate_query(&qobject!{
      "hello" => qvalue!()
    }).unwrap_err().assert_message("non-integer \"hello\"");
    array_bool.validate_query(&qobject!{
      "1" => qobject!{}
    }).unwrap_err().assert_message(r"Cannot deeply query a boolean\.");
  }

  #[test]
  fn test_query_tuple() {
    let nums = Schema::Tuple{
      items: vec![Schema::Boolean, Schema::Boolean, Schema::Boolean],
      additional_items: false
    };
    let nums_additional = Schema::Tuple{
      items: vec![Schema::Boolean, Schema::Boolean, Schema::Boolean],
      additional_items: true
    };
    let nums_and_object = Schema::Tuple{
      items: vec![
        Schema::Boolean,
        Schema::Object{
          properties: {
            let mut map = LinearMap::new();
            map.insert(String::from("hello"), Schema::Boolean);
            map
          },
          required: vec![],
          additional_properties: false
        },
        Schema::Boolean
      ],
      additional_items: false
    };
    assert!(nums.validate_query(&qobject!{
      "0" => qvalue!(),
      "2" => qvalue!()
    }).is_ok());
    nums.validate_query(&qobject!{
      "0" => qvalue!(),
      "1" => qvalue!(),
      "2" => qvalue!(),
      "3" => qvalue!()
    }).unwrap_err().assert_message(r"Tuple has only 3 values\. Can’t query the index 3\.");
    nums.validate_query(&qobject!{
      "asd" => qvalue!(),
      "1" => qvalue!(),
      "2" => qvalue!(),
      "3" => qvalue!()
    }).unwrap_err().assert_message("non-integer \"asd\"");
    nums.validate_query(&qobject!{
      "1" => qvalue!(),
      "2" => qobject!{},
      "3" => qvalue!()
    }).unwrap_err().assert_message(r"Cannot deeply query a boolean\.");
    assert!(nums_additional.validate_query(&qobject!{
      "0" => qvalue!(),
      "1" => qvalue!(),
      "3" => qvalue!(),
      "99999999" => qvalue!()
    }).is_ok());
    assert!(nums_and_object.validate_query(&qobject!{
      "0" => qvalue!(),
      "1" => qobject!{"hello" => qvalue!()},
      "2" => qvalue!()
    }).is_ok());
  }

  #[test]
  fn test_query_object() {
    let object = Schema::Object{
      properties: {
        let mut map1 = LinearMap::new();
        map1.insert(String::from("hello"), Schema::Boolean);
        map1.insert(String::from("world"), Schema::Boolean);
        map1.insert(String::from("5"), Schema::Boolean);
        map1.insert(String::from("goodbye"), Schema::Object{
          properties: {
            let mut map2 = LinearMap::new();
            map2.insert(String::from("hello"), Schema::Boolean);
            map2.insert(String::from("world"), Schema::Boolean);
            map2
          },
          required: vec![],
          additional_properties: false
        });
        map1
      },
      required: vec![],
      additional_properties: false
    };
    let object_additional = Schema::Object{
      properties: {
        let mut map = LinearMap::new();
        map.insert(String::from("hello"), Schema::Boolean);
        map.insert(String::from("world"), Schema::Boolean);
        map
      },
      required: vec![],
      additional_properties: true
    };
    assert!(object.validate_query(&qobject!{
      "world" => qvalue!(),
      "5" => qvalue!(),
      "goodbye" => qvalue!()
    }).is_ok());
    object.validate_query(&qobject!{
      "hello" => qvalue!(),
      "moon" => qvalue!()
    }).unwrap_err().assert_message("Cannot query object property \"moon\".");
    object.validate_query(&qobject!{
      "hello" => qobject!{}
    }).unwrap_err().assert_message(r"Cannot deeply query a boolean\.");
    assert!(object.validate_query(&qobject!{
      "goodbye" => qobject!{
        "hello" => qvalue!()
      }
    }).is_ok());
    object.validate_query(&qobject!{
      "goodbye" => qobject!{
        "hello" => qobject!{}
      }
    }).unwrap_err().assert_message(r"Cannot deeply query a boolean\.");
    assert!(object_additional.validate_query(&qobject!{
      "world" => qvalue!(),
      "5" => qvalue!(),
      "goodbye" => qvalue!(),
      "moon" => qvalue!()
    }).is_ok());
  }
}
