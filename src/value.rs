//! Types representing for data which will be retrieved from the driver.
//! Currently this data is expected to look like a JSON object but this may be
//! changed in the future. Driver authors must cast the data they retrieve from
//! the driver to these types.

use std::iter;

use linear_map::LinearMap;
use serde::ser::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer, Error as DeError, Visitor, SeqVisitor, MapVisitor};
use serde::de::impls::VecVisitor;
use serde_json;

use error::Error;

/// The type which represents the key for maps used throughout the Ardite
/// codebase.
///
/// Functions similarly to an object key in JavaScript.
pub type Key = String;

/// Represents a [JSON pointer][1] to a document property. Examples of a
/// pointer in this context include `/hello/world` or `/a/b/c/d`.
///
/// These pointers are represented as a list of keys.
///
/// [1]: https://duckduckgo.com/?q=json+pointer&atb=v1&ia=about
pub type Pointer = Vec<Key>;

/// Ordered representation of a map of key/value pairs, like a JSON object.
/// Backed by a linear map to maintain order and have high performance for
/// small objects.
// TODO: newtype pattern?
pub type Object = LinearMap<Key, Value>;

/// Ordered array of values, like a JSON array.
// TODO: newtype pattern?
pub type Array = Vec<Value>;

/// Various value types. Based on types in the [JSON standard][1] (see section
/// 5).
///
/// [1]: http://ecma-international.org/publications/files/ECMA-ST/ECMA-404.pdf
#[derive(PartialEq, Clone, Debug)]
pub enum Value {
  /// The abscense of any value.
  Null,
  /// True or false.
  Boolean(bool),
  /// An integer numeric value.
  I64(i64),
  /// A floating point numeric value.
  F64(f64),
  /// A list of characters.
  String(String),
  /// A map of key/value pairs.
  Object(Object),
  /// A list of values.
  Array(Array)
}

impl Value {
  /// Gets a value at a specific point. Helpful for retrieving nested values.
  // TODO: Consider removing `Pointer` and using methods like `get`, `get_path`,
  // `set`, `set_path`. This is also good for the `pointer.is_empty()` case.
  pub fn get(&self, mut pointer: Pointer) -> Option<&Value> {
    if pointer.is_empty() {
      Some(self)
    } else {
      match *self {
        Value::Object(ref map) => {
          if let Some(value) = map.get(&pointer.remove(0)) {
            value.get(pointer)
          } else {
            None
          }
        },
        Value::Array(ref vec) => {
          if let Some(value) = pointer.remove(0).parse::<usize>().ok().map_or(None, |i| vec.get(i)) {
            value.get(pointer)
          } else {
            None
          }
        },
        _ => None
      }
    }
  }

  pub fn map_keys<F>(self, transform: F) -> Value where F: Fn(Key) -> Key {
    match self {
      Value::Object(object) => {
        let mut new_object = Object::new();
        for (key, value) in object.into_iter() {
          new_object.insert(transform(key), value);
        }
        Value::Object(new_object)
      },
      value @ _ => value
    }
  }

  pub fn map_values<F>(self, transform: F) -> Value where F: Fn(Value) -> Value {
    match self {
      Value::Object(object) => {
        let mut new_object = Object::new();
        for (key, value) in object.into_iter() {
          new_object.insert(key, transform(value));
        }
        Value::Object(new_object)
      },
      Value::Array(array) => {
        let mut new_array = Array::new();
        for value in array.into_iter() {
          new_array.push(transform(value));
        }
        Value::Array(new_array)
      },
      value @ _ => value
    }
  }

  pub fn map_entries<F>(self, transform: F) -> Value where F: Fn((Key, Value)) -> (Key, Value) {
    match self {
      Value::Object(object) => {
        let mut new_object = Object::new();
        for (key, value) in object.into_iter() {
          let (new_key, new_value) = transform((key, value));
          new_object.insert(new_key, new_value);
        }
        Value::Object(new_object)
      },
      value @ _ => value
    }
  }

  /// Creates a `Value` from a JSON string.
  pub fn from_json(json: &str) -> Result<Value, Error> {
    serde_json::from_str(json).map_err(Error::from)
  }

  /// Converts a `Value` into a JSON string.
  pub fn to_json(&self) -> Result<String, Error> {
    serde_json::to_string(self).map_err(Error::from)
  }

  /// Converts a `Value` into a nice and indented JSON string.
  pub fn to_json_pretty(&self) -> Result<String, Error> {
    serde_json::to_string_pretty(self).map_err(Error::from)
  }
}

impl Serialize for Value {
  #[inline]
  fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
    match *self {
      Value::Null => serializer.serialize_unit(),
      Value::Boolean(value) => serializer.serialize_bool(value),
      Value::I64(value) => serializer.serialize_i64(value),
      Value::F64(value) => serializer.serialize_f64(value),
      Value::String(ref value) => serializer.serialize_str(&value),
      Value::Array(ref value) => value.serialize(serializer),
      Value::Object(ref value) => value.serialize(serializer)
    }
  }
}

impl Deserialize for Value {
  #[inline]
  fn deserialize<D>(deserializer: &mut D) -> Result<Value, D::Error> where D: Deserializer {
    struct ValueVisitor;

    impl Visitor for ValueVisitor {
      type Value = Value;

      #[inline] fn visit_bool<E>(&mut self, value: bool) -> Result<Value, E> { Ok(Value::Boolean(value)) }
      #[inline] fn visit_u64<E>(&mut self, value: u64) -> Result<Value, E> { Ok(Value::I64(value as i64)) }
      #[inline] fn visit_i64<E>(&mut self, value: i64) -> Result<Value, E> { Ok(Value::I64(value)) }
      #[inline] fn visit_f64<E>(&mut self, value: f64) -> Result<Value, E> { Ok(Value::F64(value)) }
      #[inline] fn visit_str<E>(&mut self, value: &str) -> Result<Value, E> where E: DeError { self.visit_string(value.to_owned()) }
      #[inline] fn visit_string<E>(&mut self, value: String) -> Result<Value, E> { Ok(Value::String(value)) }
      #[inline] fn visit_none<E>(&mut self) -> Result<Value, E> { Ok(Value::Null) }
      #[inline] fn visit_some<D>(&mut self, deserializer: &mut D) -> Result<Value, D::Error> where D: Deserializer { Deserialize::deserialize(deserializer) }
      #[inline] fn visit_unit<E>(&mut self) -> Result<Value, E> { Ok(Value::Null) }
      #[inline] fn visit_seq<V>(&mut self, visitor: V) -> Result<Value, V::Error> where V: SeqVisitor { let values = try!(VecVisitor::new().visit_seq(visitor)); Ok(Value::Array(values)) }

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Value, V::Error> where V: MapVisitor {
        let mut object = LinearMap::with_capacity(visitor.size_hint().0);
        while let Some((key, value)) = try!(visitor.visit()) {
          object.insert(key, value);
        }
        try!(visitor.end());
        Ok(Value::Object(object))
      }
    }

    deserializer.deserialize(ValueVisitor)
  }
}

impl<V> From<Option<V>> for Value where V: Into<Value> {
  fn from(option: Option<V>) -> Self {
    match option {
      None => Value::Null,
      Some(value) => value.into()
    }
  }
}

impl From<bool> for Value {
  fn from(boolean: bool) -> Self {
    Value::Boolean(boolean)
  }
}

impl From<i64> for Value {
  fn from(number: i64) -> Self {
    Value::I64(number)
  }
}

impl From<f64> for Value {
  fn from(number: f64) -> Self {
    Value::F64(number)
  }
}

impl From<String> for Value {
  fn from(string: String) -> Self {
    Value::String(string)
  }
}

impl<'a> From<&'a str> for Value {
  fn from(string: &'a str) -> Self {
    Value::from(string.to_owned())
  }
}

/// An iterator of values. Used by drivers to convert their own iterator
/// implementations into a single type.
pub struct Iter {
  iter: Box<Iterator<Item=Value> + 'static>
}

impl Iter {
  /// Create a new value iterator.
  pub fn new<I>(iter: I) -> Self where I: Iterator<Item=Value> + 'static {
    Iter {
      iter: Box::new(iter)
    }
  }

  /// Returns an empty iterator.
  pub fn none() -> Self {
    Iter::new(iter::empty())
  }
}

impl Iterator for Iter {
  type Item = Value;

  #[inline]
  fn next(&mut self) -> Option<Value> {
    self.iter.next()
  }
}

#[cfg(test)]
mod tests {
  use value::Value;

  #[test]
  fn test_get_primitive() {
    assert_eq!(value!().get(point![]).cloned(), Some(value!()));
    assert_eq!(value!().get(point!["hello"]).cloned(), None);
    assert_eq!(value!().get(point!["a", "b", "c", "d", "e"]).cloned(), None);
    assert_eq!(value!(true).get(point![]).cloned(), Some(value!(true)));
    assert_eq!(value!(true).get(point!["hello"]).cloned(), None);
    assert_eq!(value!(36).get(point![]).cloned(), Some(value!(36)));
    assert_eq!(value!(36).get(point!["hello"]).cloned(), None);
    assert_eq!(value!("world").get(point![]).cloned(), Some(value!("world")));
    assert_eq!(value!("world").get(point!["hello"]).cloned(), None);
  }

  #[test]
  fn test_get_object() {
    let object = value!({
      "hello" => true,
      "world" => 8,
      "yolo" => "swag",
      "5" => (),
      "moon" => {
        "hello" => "yoyo"
      }
    });
    assert_eq!(object.get(point![]).cloned(), Some(object.clone()));
    assert_eq!(object.get(point!["hello"]).cloned(), Some(value!(true)));
    assert_eq!(object.get(point!["yolo"]).cloned(), Some(value!("swag")));
    assert_eq!(object.get(point!["5"]).cloned(), Some(value!()));
    assert_eq!(object.get(point!["world", "hello"]).cloned(), None);
    assert_eq!(object.get(point!["moon", "hello"]).cloned(), Some(value!("yoyo")));
    assert_eq!(object.get(point!["moon", "nope"]).cloned(), None);
  }

  #[test]
  fn test_get_array() {
    let array = value!([
      false,
      64,
      {
        "hello" => true,
        "world" => false,
        "moon" => {
          "goodbye" => "yoyo"
        }
      },
      [[1, 2, 3], 4, 5 ]
    ]);
    assert_eq!(array.get(point![]).cloned(), Some(array.clone()));
    assert_eq!(array.get(point!["0"]).cloned(), Some(value!(false)));
    assert_eq!(array.get(point!["1"]).cloned(), Some(value!(64)));
    assert_eq!(array.get(point!["2", "hello"]).cloned(), Some(value!(true)));
    assert_eq!(array.get(point!["2", "moon", "goodbye"]).cloned(), Some(value!("yoyo")));
    assert_eq!(array.get(point!["length"]).cloned(), None);
    assert_eq!(array.get(point!["3", "0", "1"]).cloned(), Some(value!(2)));
  }

  #[test]
  fn test_from_json() {
    assert_eq!(Value::from_json("null").unwrap(), value!());
    assert_eq!(Value::from_json("true").unwrap(), value!(true));
    assert_eq!(Value::from_json("false").unwrap(), value!(false));
    assert_eq!(Value::from_json("7").unwrap(), value!(7));
    assert_eq!(Value::from_json("3.3").unwrap(), value!(3.3));
    assert_eq!(Value::from_json(r#""Hello,\n\"world\"!""#).unwrap(), value!("Hello,\n\"world\"!"));
    assert_eq!(Value::from_json(r#"{"hello":"world","foo":true,"null":null,"goodbye":{"moon":2}}"#).unwrap(), value!({
      "hello" => "world",
      "foo" => true,
      "null" => (),
      "goodbye" => {
        "moon" => 2
      }
    }));
    assert_eq!(
      Value::from_json(r#"["world",3.3,{"hello":"world"},null,null,[1,2,3],null]"#).unwrap(),
      value!(["world", 3.3, { "hello" => "world" }, (), (), [1, 2, 3], ()])
    );
  }

  #[test]
  fn test_to_json() {
    assert_eq!(&value!().to_json().unwrap(), "null");
    assert_eq!(&value!(true).to_json().unwrap(), "true");
    assert_eq!(&value!(false).to_json().unwrap(), "false");
    assert_eq!(&value!(7).to_json().unwrap(), "7");
    assert_eq!(&value!(6.667).to_json().unwrap(), "6.667");
    assert_eq!(&value!("Hello,\n\"world\"!").to_json().unwrap(), r#""Hello,\n\"world\"!""#);
    assert_eq!(&value!({
      "hello" => "world",
      "foo" => true,
      "null" => (),
      "goodbye" => {
        "moon" => 2
      }
    }).to_json().unwrap(), r#"{"hello":"world","foo":true,"null":null,"goodbye":{"moon":2}}"#);
    assert_eq!(
      &value!(["world", 3.333, { "hello" => "world" }, (), (), [1, 2, 3], ()]).to_json().unwrap(),
      r#"["world",3.333,{"hello":"world"},null,null,[1,2,3],null]"#
    );
  }

  #[test]
  fn test_to_json_pretty() {
    assert_eq!(
      &value!(["world", 3.333, { "hello" => "world" }, (), (), [1, 2, 3], ()]).to_json_pretty().unwrap(),
      "[\n  \"world\",\n  3.333,\n  {\n    \"hello\": \"world\"\n  },\n  null,\n  null,\n  [\n    1,\n    2,\n    3\n  ],\n  null\n]"
    );
  }
}
