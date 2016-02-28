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
#[derive(PartialEq, Clone, Debug)]
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

impl Value {
  /// Gets a value at a specific point. Helpful for retrieving nested values.
  pub fn get(&self, mut pointer: Pointer) -> Option<Value> {
    match self {
      &Value::Null => if pointer.len() == 0 { Some(Value::Null) } else { None },
      &Value::Boolean(_) => if pointer.len() == 0 { Some(self.clone()) } else { None },
      &Value::Number(_) => if pointer.len() == 0 { Some(self.clone()) } else { None },
      &Value::String(_) => if pointer.len() == 0 { Some(self.clone()) } else { None },
      &Value::Object(ref map) => {
        if pointer.len() == 0 {
          Some(self.clone())
        } else {
          let key = pointer.remove(0);
          if let Some(value) = map.get(&key) {
            value.get(pointer)
          } else {
            None
          }
        }
      },
      &Value::Array(ref vec) => {
        if pointer.len() == 0 {
          Some(self.clone())
        } else {
          let key = pointer.remove(0);
          if let Some(index) = key.parse::<usize>().ok() {
            if let Some(value) = vec.get(index) {
              value.get(pointer)
            } else {
              None
            }
          } else {
            None
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_get_primitive() {
    assert_eq!(vnull!().get(point![]), Some(vnull!()));
    assert_eq!(vnull!().get(point!["hello"]), None);
    assert_eq!(vnull!().get(point!["a", "b", "c", "d", "e"]), None);
    assert_eq!(vbool!(true).get(point![]), Some(vbool!(true)));
    assert_eq!(vbool!(true).get(point!["hello"]), None);
    assert_eq!(vnum!(36f64).get(point![]), Some(vnum!(36f64)));
    assert_eq!(vnum!(36f64).get(point!["hello"]), None);
    assert_eq!(vstring!("world").get(point![]), Some(vstring!("world")));
    assert_eq!(vstring!("world").get(point!["hello"]), None);
  }

  #[test]
  fn test_get_object() {
    let object = vobject!{
      "hello" => vbool!(true),
      "world" => vnum!(8f64),
      "5" => vnull!(),
      "moon" => vobject!{
        "hello" => vstring!("yoyo")
      }
    };
    assert_eq!(object.get(point![]), Some(object.clone()));
    assert_eq!(object.get(point!["hello"]), Some(vbool!(true)));
    assert_eq!(object.get(point!["5"]), Some(vnull!()));
    assert_eq!(object.get(point!["world", "hello"]), None);
    assert_eq!(object.get(point!["moon", "hello"]), Some(vstring!("yoyo")));
    assert_eq!(object.get(point!["moon", "nope"]), None);
  }

  #[test]
  fn test_get_array() {
    let array = varray![
      vbool!(false),
      vnum!(64f64),
      vobject!{
        "hello" => vbool!(true),
        "world" => vbool!(false),
        "moon" => vobject!{
          "goodbye" => vstring!("yoyo")
        }
      },
      varray![
        varray![
          vnum!(1f64),
          vnum!(2f64),
          vnum!(3f64)
        ],
        vnum!(4f64),
        vnum!(5f64)
      ]
    ];
    assert_eq!(array.get(point![]), Some(array.clone()));
    assert_eq!(array.get(point!["0"]), Some(vbool!(false)));
    assert_eq!(array.get(point!["1"]), Some(vnum!(64f64)));
    assert_eq!(array.get(point!["2", "hello"]), Some(vbool!(true)));
    assert_eq!(array.get(point!["2", "moon", "goodbye"]), Some(vstring!("yoyo")));
    assert_eq!(array.get(point!["length"]), None);
    assert_eq!(array.get(point!["3", "0", "1"]), Some(vnum!(2f64)));
  }
}
