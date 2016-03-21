use std::collections::BTreeMap;

use regex::Regex;
use serde::de::{Deserialize, Deserializer, Error as DeError, Visitor, MapVisitor};
use serde::de::impls::IgnoredAny;
use url::Url;

use schema::{Definition, Type, DriverConfig, Schema, BoxedSchema};
use value::{Key, Value};

macro_rules! deserializable_fields {
  ($($name:expr => $variant:ident),*) => {
    // Create an enum from our variants.
    enum Field {
      $(
        $variant,
      )*
      Ignore
    }

    // Implement deserialize for our enum.
    impl Deserialize for Field {
      #[inline]
      fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        // Create a visitor for our fields.
        struct FieldVisitor;

        // Implement visitor for said visitor.
        impl Visitor for FieldVisitor {
          type Value = Field;

          // Visit `str`.
          fn visit_str<E>(&mut self, value: &str) -> Result<Field, E> where E: DeError {
            match value {
              // If the value matches one of our variants return it.
              $(
                $name => Ok(Field::$variant),
              )*
              // Otherwise ignore it.
              _ => Ok(Field::Ignore)
            }
          }
        }

        // Finally, deserialize the struct fields using our visitor.
        deserializer.deserialize_struct_field(FieldVisitor)
      }
    }
  }
}

macro_rules! visit_map_fields {
  ($visitor:expr, { $($field_name:expr => $var_name:ident),* }) => {{
    #[allow(non_camel_case_types)]
    enum __Field { $($var_name,)* __Ignore }

    impl Deserialize for __Field {
      #[inline]
      fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        struct __Visitor;

        impl Visitor for __Visitor {
          type Value = __Field;

          fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: DeError {
            match value {
              $($field_name => Ok(__Field::$var_name),)*
              _ => Ok(__Field::__Ignore)
            }
          }
        }

        deserializer.deserialize_struct_field(__Visitor)
      }
    }

    while let Some(key) = try!($visitor.visit_key()) {
      match key {
        $(__Field::$var_name => { $var_name = try!($visitor.visit_value()); },)*
        __Field::__Ignore => { try!($visitor.visit_value::<IgnoredAny>()); }
      }
    }

    try!($visitor.end());
  }}
}

impl Deserialize for Definition {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    struct DefinitionVisitor;

    impl Visitor for DefinitionVisitor {
      type Value = Definition;

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapVisitor {
        let mut driver_config: Option<DriverConfig> = None;
        let mut types: Option<BTreeMap<Key, Type>> = None;

        visit_map_fields!(visitor, {
          "driver" => driver_config,
          "types" => types
        });

        let mut definition = Definition::new();

        if let Some(driver_config) = driver_config { definition.set_driver(driver_config); }

        if let Some(types) = types {
          for (key, type_) in types {
            definition.add_type(key, type_);
          }
        }

        Ok(definition)
      }
    }

    deserializer.deserialize_map(DefinitionVisitor)
  }
}

impl Deserialize for Type {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    struct TypeVisitor;

    impl Visitor for TypeVisitor {
      type Value = Type;

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapVisitor {
        let mut schema: Option<BoxedSchema> = None;

        visit_map_fields!(visitor, {
          "schema" => schema
        });

        let mut type_ = Type::new();

        if let Some(schema) = schema { type_.set_boxed_schema(schema); }

        Ok(type_)
      }
    }

    deserializer.deserialize_map(TypeVisitor)
  }
}

impl Deserialize for DriverConfig {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    struct DriverConfigVisitor;

    impl Visitor for DriverConfigVisitor {
      type Value = DriverConfig;

      #[inline]
      fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: DeError {
        match Url::parse(value) {
          Ok(url) => Ok(DriverConfig::new(url)),
          Err(error) => Err(DeError::custom(format!("{}", error)))
        }
      }

      #[inline]
      fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: DeError {
        self.visit_str(&value)
      }
    }

    deserializer.deserialize(DriverConfigVisitor)
  }
}

impl Deserialize for BoxedSchema {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    struct SchemaVisitor;

    impl Visitor for SchemaVisitor {
      type Value = BoxedSchema;

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapVisitor {
        let mut type_: Option<String> = None;
        let mut multiple_of: Option<f32> = None;
        let mut minimum: Option<f64> = None;
        let mut exclusive_minimum: Option<bool> = None;
        let mut maximum: Option<f64> = None;
        let mut exclusive_maximum: Option<bool> = None;
        let mut min_length: Option<u64> = None;
        let mut max_length: Option<u64> = None;
        let mut pattern: Option<String> = None;
        let mut items: Option<BoxedSchema> = None;
        let mut properties: Option<BTreeMap<String, BoxedSchema>> = None;
        let mut required: Option<Vec<String>> = None;
        let mut additional_properties: Option<bool> = None;
        let mut enum_: Option<Vec<Value>> = None;

        visit_map_fields!(visitor, {
          "type" => type_,
          "multipleOf" => multiple_of,
          "minimum" => minimum,
          "exclusiveMinimum" => exclusive_minimum,
          "maximum" => maximum,
          "exclusiveMaximum" => exclusive_maximum,
          "minLength" => min_length,
          "maxLength" => max_length,
          "pattern" => pattern,
          "items" => items,
          "properties" => properties,
          "required" => required,
          "additionalProperties" => additional_properties,
          "enum" => enum_
        });

        if let Some(enum_) = enum_ {
          return Ok(Box::new(Schema::enum_(enum_)));
        }

        if let Some(type_) = type_ {
          match type_.as_str() {
            "null" => Ok(Box::new(Schema::null())),
            "boolean" => Ok(Box::new(Schema::boolean())),
            "number" | "integer" => {
              let mut schema = Schema::number();
              if type_ == "integer" { schema.set_multiple_of(1.0); }
              else if let Some(multiple_of) = multiple_of { schema.set_multiple_of(multiple_of); }
              if let Some(minimum) = minimum { schema.set_minimum(minimum); }
              if let Some(maximum) = maximum { schema.set_maximum(maximum); }
              if exclusive_minimum.unwrap_or(false) { schema.enable_exclusive_minimum(); }
              if exclusive_maximum.unwrap_or(false) { schema.enable_exclusive_maximum(); }
              Ok(Box::new(schema))
            },
            "string" => {
              let mut schema = Schema::string();
              if let Some(min_length) = min_length { schema.set_min_length(min_length); }
              if let Some(max_length) = max_length { schema.set_max_length(max_length); }
              if let Some(pattern) = pattern.and_then(|p| Regex::new(&p).ok()) { schema.set_pattern(pattern); }
              Ok(Box::new(schema))
            },
            "array" => {
              let mut schema = Schema::array();
              if let Some(items) = items { schema.set_boxed_items(items); }
              Ok(Box::new(schema))
            },
            "object" => {
              let mut schema = Schema::object();
              schema.set_required(required.unwrap_or_default());
              if additional_properties.unwrap_or(false) { schema.enable_additional_properties(); }
              for (key, sub_schema) in properties.unwrap_or_default() {
                schema.add_boxed_property(key, sub_schema);
              }
              Ok(Box::new(schema))
            },
            _ => Err(DeError::custom(format!("Cannot use '{}' for a schema type property.", type_)))
          }
        } else {
          Err(DeError::custom("No type property for schema was specified."))
        }
      }
    }

    deserializer.deserialize_map(SchemaVisitor)
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use regex::Regex;
  use serde_json;
  use url::Url;

  use schema::{Definition, Type, DriverConfig, Schema};

  #[test]
  fn test_json_definition() {
    let from_str = serde_json::from_str::<Definition>;
    assert_eq!(from_str("{}").unwrap(), Definition::new());
    assert_eq!(from_str(r#"{"hello":"world"}"#).unwrap(), Definition::new());
    assert!(from_str(r#"{"types":2}"#).is_err());
    assert!(from_str(r#"{"types":"yo"}"#).is_err());
    assert!(from_str(r#"{"types":[]}"#).is_err());
    assert_eq!(from_str(r#"{"types":{}}"#).unwrap(), Definition::new());
  }

  #[test]
  fn test_json_type() {
    let from_str = serde_json::from_str::<Type>;
    assert_eq!(from_str("{}").unwrap(), Type::new());
    assert_eq!(from_str(r#"{"hello":"world"}"#).unwrap(), Type::new());
    assert!(from_str(r#"{"schema":2}"#).is_err());
    assert!(from_str(r#"{"schema":"yo"}"#).is_err());
    assert!(from_str(r#"{"schema":[]}"#).is_err());
  }

  #[test]
  fn test_json_driver_config() {
    let from_str = serde_json::from_str::<DriverConfig>;
    assert_eq!(from_str(r#""mongodb://localhost:27017""#).unwrap(), DriverConfig::new(Url::parse("mongodb://localhost:27017").unwrap()));
    assert!(from_str(r#""not a url or a name""#).is_err());
  }

  fn create_basic_definition() -> Definition {
    // TODO: use order in file, not serde’s `BTreeMap` order.
    let mut definition = Definition::new();

    definition.add_type("person", {
      let mut type_ = Type::new();
      let mut person = Schema::object();
      person.set_required(vec!["email"]);
      person.add_property("email", {
        let mut email = Schema::string();
        email.set_min_length(4);
        email.set_max_length(256);
        email.set_pattern(Regex::new(r".+@.+\..+").unwrap());
        email
      });
      person.add_property("name", {
        let mut name = Schema::string();
        name.set_min_length(2);
        name.set_max_length(64);
        name
      });
      type_.set_schema(person);
      type_
    });

    definition.add_type("post", {
      let mut type_ = Type::new();
      let mut post = Schema::object();
      post.set_required(vec!["headline"]);
      post.add_property("headline", {
        let mut headline = Schema::string();
        headline.set_min_length(4);
        headline.set_max_length(1024);
        headline
      });
      post.add_property("text", {
        let mut text = Schema::string();
        text.set_max_length(65536);
        text
      });
      post.add_property("topic", {
        Schema::enum_(vec!["showcase", "help", "ama"])
      });
      type_.set_schema(post);
      type_
    });

    definition
  }

  #[test]
  fn test_basic_json() {
    assert_eq!(
      Definition::from_file(PathBuf::from("tests/fixtures/definitions/basic.json")).unwrap(),
      create_basic_definition()
    );
  }

  #[test]
  fn test_basic_yaml() {
    assert_eq!(
      Definition::from_file(PathBuf::from("tests/fixtures/definitions/basic.yml")).unwrap(),
      create_basic_definition()
    );
  }
}
