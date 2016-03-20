//! Types representing for data which will be retrieved from the driver.
//! Currently this data is expected to look like a JSON object but this may be
//! changed in the future. Driver authors must cast the data they retrieve from
//! the driver to these types.

use linear_map::LinearMap;

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
  pub fn get(&self, mut pointer: Pointer) -> Option<&Value> {
    match *self {
      Value::Object(ref map) => {
        if pointer.is_empty() {
          Some(self)
        } else if let Some(value) = map.get(&pointer.remove(0)) {
          value.get(pointer)
        } else {
          None
        }
      },
      Value::Array(ref vec) => {
        if pointer.is_empty() {
          Some(self)
        } else if let Some(value) = pointer.remove(0).parse::<usize>().ok().map_or(None, |i| vec.get(i)) {
          value.get(pointer)
        } else {
          None
        }
      },
      _ => if pointer.is_empty() { Some(self) } else { None }
    }
  }

  /// Converts a value into a JSON string for distribution.
  pub fn to_json(&self) -> String {
    match *self {
      Value::Null => "null".to_owned(),
      Value::Boolean(value) => if value { "true".to_owned() } else { "false".to_owned() },
      Value::I64(value) => value.to_string(),
      Value::F64(value) => value.to_string(),
      Value::String(ref value) => "\"".to_owned() + &escape_string_for_json(value) + "\"",
      Value::Object(ref map) => {
        let mut json = "{".to_owned();
        for (key, value) in map {
          json.push_str("\"");
          json.push_str(&escape_string_for_json(key));
          json.push_str("\":");
          json.push_str(&value.to_json());
          json.push_str(",");
        }
        // Remove the last comma.
        json.pop();
        json.push_str("}");
        json
      },
      Value::Array(ref vec) => {
        let mut json = "[".to_owned();
        for item in vec {
          json.push_str(&item.to_json());
          json.push_str(",");
        }
        // Remove the last comma.
        json.pop();
        json.push_str("]");
        json
      }
    }
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

/// Takes a string and escapes it for use within a JSON encoded object. Read,
/// inside quotes.
fn escape_string_for_json(string: &str) -> String {
  string.replace("\"", "\\\"").replace("\n", "\\n")
}

/// An iterator of values. Used by drivers to convert their own iterator
/// implementations into a single type.
pub struct ValueIter<'a> {
  iter: Box<Iterator<Item=Value> + 'a>
}

impl<'a> ValueIter<'a> {
  /// Create a new value iterator.
  pub fn new<I>(iter: I) -> Self where I: Iterator<Item=Value> + 'a {
    ValueIter {
      iter: Box::new(iter)
    }
  }
}

impl<'a> Iterator for ValueIter<'a> {
  type Item = Value;

  #[inline]
  fn next(&mut self) -> Option<Value> {
    self.iter.next()
  }
}

#[cfg(test)]
mod tests {
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
  fn test_to_json() {
    assert_eq!(&value!().to_json(), "null");
    assert_eq!(&value!(true).to_json(), "true");
    assert_eq!(&value!(false).to_json(), "false");
    assert_eq!(&value!(7).to_json(), "7");
    assert_eq!(&value!(6.667).to_json(), "6.667");
    assert_eq!(&value!("Hello,\n\"world\"!").to_json(), r#""Hello,\n\"world\"!""#);
    assert_eq!(&value!({
      "hello" => "world",
      "foo" => true,
      "null" => (),
      "goodbye" => {
        "moon" => 2
      }
    }).to_json(), r#"{"hello":"world","foo":true,"null":null,"goodbye":{"moon":2}}"#);
    assert_eq!(
      &value!(["world", 3.333, { "hello" => "world" }, (), (), [1, 2, 3], ()]).to_json(),
      r#"["world",3.333,{"hello":"world"},null,null,[1,2,3],null]"#
    );
  }
}
