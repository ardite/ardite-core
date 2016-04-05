//! Types representing for data which will be retrieved from the driver.
//! Currently this data is expected to look like a JSON object but this may be
//! changed in the future. Driver authors must cast the data they retrieve from
//! the driver to these types.

use std::cmp::Ordering;

use linear_map;
use linear_map::LinearMap;
use serde::ser::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer, Error as DeError, Visitor, SeqVisitor, MapVisitor};
use serde::de::impls::VecVisitor;
use serde_json;

use error::Error;

/// Ordered representation of a map of key/value pairs, like a JSON object.
/// Backed by a linear map to maintain order and have high performance for
/// small objects.
#[derive(PartialEq, Clone, Debug)]
pub struct Object(LinearMap<String, Value>);

impl Object {
  #[inline] pub fn new() -> Self { Object(LinearMap::new()) }
  #[inline] pub fn get(&self, key: &str) -> Option<&Value> { self.0.get(key) }
  #[inline] pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<Value> where K: Into<String>, V: Into<Value> { self.0.insert(key.into(), value.into()) }

  pub fn map_keys<F>(self, transform: F) -> Object where F: Fn(String) -> String {
    let mut object = Object::new();
    for (key, value) in self.into_iter() {
      object.insert(transform(key), value);
    }
    object
  }

  pub fn map_values<F>(self, transform: F) -> Object where F: Fn(Value) -> Value {
    let mut object = Object::new();
    for (key, value) in self.into_iter() {
      object.insert(key, transform(value));
    }
    object
  }

  pub fn map_entries<F>(self, transform: F) -> Object where F: Fn((String, Value)) -> (String, Value) {
    let mut object = Object::new();
    for (key, value) in self.into_iter() {
      let (new_key, new_value) = transform((key, value));
      object.insert(new_key, new_value);
    }
    object
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

impl IntoIterator for Object {
  type Item = (String, Value);
  type IntoIter = linear_map::IntoIter<String, Value>;
  #[inline] fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl Serialize for Object {
  #[inline] fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer { self.0.serialize(serializer) }
}

impl Deserialize for Object {
  #[inline] fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer { LinearMap::deserialize(deserializer).map(Object) }
}

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
  Null(()),
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
  /// Gets the value of an object or array variant for a key.
  ///
  /// # Example
  /// ```rust
  /// # #[macro_use(value)]
  /// # extern crate ardite;
  /// # fn main() {
  /// assert_eq!(value!(2).get("hello"), None);
  /// assert_eq!(value!({ "hello" => "world" }).get("hello"), Some(&value!("world")));
  /// assert_eq!(value!({ "hello" => "world" }).get("goodbye"), None);
  /// assert_eq!(value!([false, "a", "b", 42]).get("0"), Some(&value!(false)));
  /// assert_eq!(value!([false, "a", "b", 42]).get("20"), None);
  /// # }
  /// ```
  pub fn get<'a>(&'a self, key: &str) -> Option<&'a Value> {
    match *self {
      Value::Object(ref object) => object.get(key),
      Value::Array(ref array) => key.parse::<usize>().ok().map_or(None, |i| array.get(i)),
      _ => None
    }
  }

  /// Gets the value of an object or array variant recursively.
  ///
  /// # Example
  /// ```rust
  /// # #[macro_use(value)]
  /// # extern crate ardite;
  /// # fn main() {
  /// assert_eq!(value!(2).get_path(&["hello", "world"]), None);
  /// assert_eq!(value!({ "hello" => "world" }).get_path(&["hello", "world"]), None);
  /// assert_eq!(value!({ "hello" => { "world" => true } }).get_path(&["hello", "world"]), Some(&value!(true)));
  /// assert_eq!(value!({
  ///   "a" => {
  ///     "b" => {
  ///       "c" => [0, 1, 2, { "4" => 42 }, 4, 5]
  ///     }
  ///   }
  /// }).get_path(&["a", "b", "c", "3", "4"]), Some(&value!(42)));
  /// # }
  /// ```
  pub fn get_path<'a>(&'a self, path: &[&str]) -> Option<&'a Value> {
    path.iter().fold(Some(self), |value, key| value.and_then(|value| value.get(key)))
  }

  /// Sets the value of a certain key on an object or array.
  ///
  /// # Example
  /// ```rust
  /// # #[macro_use(value)]
  /// # extern crate ardite;
  /// # fn main() {
  /// assert!(value!(false).set("hello", value!(true)).is_err());
  /// assert!(value!([1, 2, 3]).set("200", value!(true)).is_err());
  /// assert_eq!(value!([1, 2, 3]).set("1", value!(true)).unwrap(), value!([1, true, 3]));
  /// assert_eq!(value!({}).set("hello", value!("world")).unwrap(), value!({ "hello" => "world" }));
  /// assert_eq!(value!({ "hello" => "moon" }).set("hello", value!("world")).unwrap(), value!({ "hello" => "world" }));
  /// # }
  /// ```
  pub fn set(self, key: &str, new: Value) -> Result<Value, Error> {
    match self {
      Value::Object(mut object) => {
        object.insert(key.to_owned(), new);
        Ok(Value::Object(object))
      },
      Value::Array(mut array) => {
        if let Some(index) = key.parse::<usize>().ok() {
          if index < array.len() {
            array[index] = new;
            Ok(Value::Array(array))
          } else {
            Err(Error::invalid(
              format!("Can’t set index {} because it is out of range for array of length {}.", index, array.len()),
              "Try setting an index inside the array’s bounds."
            ))
          }
        } else {
          Err(Error::invalid(
            format!("Key '{}' is not a positive integer and can’t be used to set a value for an array.", key),
            "Try using a positive integer like 0 as the key."
          ))
        }
      },
      _ => Err(Error::invalid(
        format!("Cannot set key '{}' for primitive value {}.", key, self.debug_name()),
        "Try using setting a value on an object or an array instead."
      ))
    }
  }

  pub fn map_keys<F>(self, transform: F) -> Value where F: Fn(String) -> String {
    match self {
      Value::Object(object) => Value::Object(object.map_keys(transform)),
      value @ _ => value
    }
  }

  pub fn map_values<F>(self, transform: F) -> Value where F: Fn(Value) -> Value {
    match self {
      Value::Object(object) => Value::Object(object.map_values(transform)),
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

  pub fn map_entries<F>(self, transform: F) -> Value where F: Fn((String, Value)) -> (String, Value) {
    match self {
      Value::Object(object) => Value::Object(object.map_entries(transform)),
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

  fn debug_name(&self) -> &'static str {
    match *self {
      Value::Null(_) => "null",
      Value::Boolean(_) => "boolean",
      Value::I64(_) => "i64",
      Value::F64(_) => "f64",
      Value::String(_) => "string",
      Value::Object(_) => "object",
      Value::Array(_) => "array"
    }
  }
}

impl PartialOrd<Value> for Value {
  /// Only orders some variants with obvious orderings. Such variants being:
  ///
  /// - `Value::Null`
  /// - `Value::Boolean`
  /// - `Value::I64`
  /// - `Value::F64`
  /// - `Value::String`
  fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
    use self::Value::*;
    match (self, other) {
      (&Null(ref a), &Null(ref b)) => a.partial_cmp(b),
      (&Boolean(ref a), &Boolean(ref b)) => a.partial_cmp(b),
      (&I64(ref a), &I64(ref b)) => a.partial_cmp(b),
      (&F64(ref a), &F64(ref b)) => a.partial_cmp(b),
      (&String(ref a), &String(ref b)) => a.partial_cmp(b),
      _ => None
    }
  }
}

impl Serialize for Value {
  #[inline]
  fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
    match *self {
      Value::Null(_) => serializer.serialize_unit(),
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
      #[inline] fn visit_none<E>(&mut self) -> Result<Value, E> { Ok(Value::Null(())) }
      #[inline] fn visit_some<D>(&mut self, deserializer: &mut D) -> Result<Value, D::Error> where D: Deserializer { Deserialize::deserialize(deserializer) }
      #[inline] fn visit_unit<E>(&mut self) -> Result<Value, E> { Ok(Value::Null(())) }
      #[inline] fn visit_seq<V>(&mut self, visitor: V) -> Result<Value, V::Error> where V: SeqVisitor { let values = try!(VecVisitor::new().visit_seq(visitor)); Ok(Value::Array(values)) }

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Value, V::Error> where V: MapVisitor {
        let mut object = LinearMap::with_capacity(visitor.size_hint().0);
        while let Some((key, value)) = try!(visitor.visit()) {
          object.insert(key, value);
        }
        try!(visitor.end());
        Ok(Value::Object(Object(object)))
      }
    }

    deserializer.deserialize(ValueVisitor)
  }
}

impl<V> From<Option<V>> for Value where V: Into<Value> {
  fn from(option: Option<V>) -> Self {
    match option {
      None => Value::Null(()),
      Some(value) => value.into()
    }
  }
}

impl From<()> for Value {
  fn from(unit: ()) -> Self {
    Value::Null(unit)
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

impl From<Object> for Value {
  fn from(object: Object) -> Self {
    Value::Object(object)
  }
}

#[cfg(test)]
mod tests {
  use value::Value;

  #[test]
  fn test_get_primitive() {
    assert_eq!(value!().get_path(&[]).cloned(), Some(value!()));
    assert_eq!(value!().get_path(&["hello"]).cloned(), None);
    assert_eq!(value!().get_path(&["a", "b", "c", "d", "e"]).cloned(), None);
    assert_eq!(value!(true).get_path(&[]).cloned(), Some(value!(true)));
    assert_eq!(value!(true).get_path(&["hello"]).cloned(), None);
    assert_eq!(value!(36).get_path(&[]).cloned(), Some(value!(36)));
    assert_eq!(value!(36).get_path(&["hello"]).cloned(), None);
    assert_eq!(value!("world").get_path(&[]).cloned(), Some(value!("world")));
    assert_eq!(value!("world").get_path(&["hello"]).cloned(), None);
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
    assert_eq!(object.get_path(&[]).cloned(), Some(object.clone()));
    assert_eq!(object.get_path(&["hello"]).cloned(), Some(value!(true)));
    assert_eq!(object.get_path(&["yolo"]).cloned(), Some(value!("swag")));
    assert_eq!(object.get_path(&["5"]).cloned(), Some(value!()));
    assert_eq!(object.get_path(&["world", "hello"]).cloned(), None);
    assert_eq!(object.get_path(&["moon", "hello"]).cloned(), Some(value!("yoyo")));
    assert_eq!(object.get_path(&["moon", "nope"]).cloned(), None);
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
      [[1, 2, 3], 4, 5]
    ]);
    assert_eq!(array.get_path(&[]).cloned(), Some(array.clone()));
    assert_eq!(array.get_path(&["0"]).cloned(), Some(value!(false)));
    assert_eq!(array.get_path(&["1"]).cloned(), Some(value!(64)));
    assert_eq!(array.get_path(&["2", "hello"]).cloned(), Some(value!(true)));
    assert_eq!(array.get_path(&["2", "moon", "goodbye"]).cloned(), Some(value!("yoyo")));
    assert_eq!(array.get_path(&["length"]).cloned(), None);
    assert_eq!(array.get_path(&["3", "0", "1"]).cloned(), Some(value!(2)));
  }

  #[test]
  fn test_set_primitive() {
    assert!(value!().set("hello", value!(true)).is_err());
    assert!(value!(false).set("hello", value!(true)).is_err());
    assert!(value!(32).set("hello", value!(true)).is_err());
    assert!(value!("hello").set("hello", value!(true)).is_err());
  }

  #[test]
  fn test_set_array() {
    assert!(value!([1, 2, 3]).set("yo", value!(true)).is_err());
    assert!(value!([1, 2, 3]).set("3", value!(true)).is_err());
    assert_eq!(value!([1, 2, 3]).set("1", value!(true)).unwrap(), value!([1, true, 3]));
  }

  #[test]
  fn test_set_object() {
    assert_eq!(value!({}).set("hello", value!("world")).unwrap(), value!({ "hello" => "world" }));
    assert_eq!(value!({ "hello" => "moon" }).set("hello", value!("world")).unwrap(), value!({ "hello" => "world" }));
    assert_eq!(value!({ "yo" => 42 }).set("hello", value!("world")).unwrap(), value!({ "yo" => 42, "hello" => "world" }));
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
