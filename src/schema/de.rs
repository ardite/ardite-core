use std::collections::BTreeMap;
use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;

use serde::de;
use serde::de::{Deserialize, Deserializer, MapVisitor};
use serde::de::impls::IgnoredAny;
use serde_json;
use serde_yaml;
use url::Url;

use error::{Error, NotAcceptable};
use schema::{Schema, Type, Driver};
use value::Value;

pub fn from_file(path: PathBuf) -> Result<Schema, Error> {
  if !path.exists() {
    return Err(Error::not_found(format!("Schema definition file not found at '{}'.", path.display())))
  }
  let extension = path.extension().map_or("", |s| s.to_str().unwrap());
  let file = try!(File::open(&path));
  let reader = BufReader::new(file);
  Ok(match extension {
    "json" => try!(serde_json::from_reader(reader)),
    "yml" => try!(serde_yaml::from_reader(reader)),
    _ => {
      return Err(
        Error::new(NotAcceptable, format!("File extension '{}' cannot be deserialized in '{}'.", extension, path.display()))
        .set_hint("Use a readable file extension like '.json' or '.yml'.")
      )
    }
  })
}

macro_rules! visit_map_fields {
  ($visitor:expr, { $($field_name:expr => $var_name:ident),* }) => {{
    #[allow(non_camel_case_types)]
    enum Field { $($var_name,)* Ignore }

    impl Deserialize for Field {
      #[inline]
      fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        struct Visitor;

        impl de::Visitor for Visitor {
          type Value = Field;

          fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: de::Error {
            match value {
              $($field_name => Ok(Field::$var_name),)*
              _ => Ok(Field::Ignore)
            }
          }
        }

        deserializer.deserialize_struct_field(Visitor)
      }
    }

    while let Some(key) = try!($visitor.visit_key()) {
      match key {
        $(Field::$var_name => { $var_name = try!($visitor.visit_value()); },)*
        Field::Ignore => { try!($visitor.visit_value::<IgnoredAny>()); }
      }
    }

    try!($visitor.end());
  }}
}

impl Deserialize for Schema {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    struct Visitor;

    impl de::Visitor for Visitor {
      type Value = Schema;

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapVisitor {
        let mut driver: Option<Driver> = None;
        let mut types: Option<BTreeMap<String, Type>> = None;

        visit_map_fields!(visitor, {
          "driver" => driver,
          "types" => types
        });

        let mut schema = Schema::new();

        if let Some(driver) = driver { schema.set_driver(driver); }

        if let Some(types) = types {
          for (key, type_) in types {
            schema.insert_type(key, type_);
          }
        }

        Ok(schema)
      }
    }

    deserializer.deserialize_map(Visitor)
  }
}

impl Deserialize for Type {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    struct Visitor;

    impl de::Visitor for Visitor {
      type Value = Type;

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapVisitor {
        try!(visitor.end());
        Ok(Type::new())
      }
    }

    deserializer.deserialize_map(Visitor)
  }
}

impl Deserialize for Driver {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    struct Visitor;

    impl de::Visitor for Visitor {
      type Value = Driver;

      #[inline]
      fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: de::Error {
        match Url::parse(value) {
          Ok(url) => Ok(Driver::new(url)),
          Err(error) => Err(de::Error::custom(format!("{}", error)))
        }
      }

      #[inline]
      fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: de::Error {
        self.visit_str(&value)
      }
    }

    deserializer.deserialize(Visitor)
  }
}

#[cfg(test)]
mod tests {
  use serde_json;
  use url::Url;

  use schema::{Schema, Type, Driver};

  #[test]
  fn test_json_schema() {
    let from_str = serde_json::from_str::<Schema>;
    assert_eq!(from_str("{}").unwrap(), Schema::new());
    assert_eq!(from_str(r#"{"hello":"world"}"#).unwrap(), Schema::new());
    assert!(from_str(r#"{"types":2}"#).is_err());
    assert!(from_str(r#"{"types":"yo"}"#).is_err());
    assert!(from_str(r#"{"types":[]}"#).is_err());
    assert_eq!(from_str(r#"{"types":{}}"#).unwrap(), Schema::new());
  }

  #[test]
  fn test_json_type() {
    let from_str = serde_json::from_str::<Type>;
    assert_eq!(from_str("{}").unwrap(), Type::new());
  }

  #[test]
  fn test_json_driver() {
    let from_str = serde_json::from_str::<Driver>;
    assert_eq!(from_str(r#""mongodb://localhost:27017""#).unwrap(), Driver::new(Url::parse("mongodb://localhost:27017").unwrap()));
    assert!(from_str(r#""not a url or a name""#).is_err());
  }
}
