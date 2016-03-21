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

impl Deserialize for Definition {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    deserializable_fields! {
      "driver" => Driver,
      "types" => Types
    }

    struct DefinitionVisitor;

    impl Visitor for DefinitionVisitor {
      type Value = Definition;

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapVisitor {
        let mut driver_config: Option<DriverConfig> = None;
        let mut types: Option<BTreeMap<Key, Type>> = None;

        while let Some(key) = try!(visitor.visit_key()) {
          match key {
            Field::Driver => { driver_config = try!(visitor.visit_value()) },
            Field::Types => { types = try!(visitor.visit_value()); },
            Field::Ignore => { try!(visitor.visit_value::<IgnoredAny>()); }
          }
        }

        try!(visitor.end());

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
    deserializable_fields! {
      "schema" => Schema
    }

    struct TypeVisitor;

    impl Visitor for TypeVisitor {
      type Value = Type;

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapVisitor {
        let mut type_ = Type::new();

        while let Some(key) = try!(visitor.visit_key()) {
          match key {
            Field::Schema => {
              let schema: BoxedSchema = try!(visitor.visit_value());
              type_.set_boxed_schema(schema);
            },
            Field::Ignore => { try!(visitor.visit_value::<IgnoredAny>()); }
          }
        }

        try!(visitor.end());
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
    deserializable_fields! {
      "type" => Type,
      "multipleOf" => MultipleOf,
      "minimum" => Minimum,
      "exclusiveMinimum" => ExclusiveMinimum,
      "maximum" => Maximum,
      "exclusiveMaximum" => ExclusiveMaximum,
      "minLength" => MinLength,
      "maxLength" => MaxLength,
      "pattern" => Pattern,
      "items" => Items,
      "properties" => Properties,
      "required" => Required,
      "additionalProperties" => AdditionalProperties,
      "enum" => Enum
    }

    #[derive(Default)]
    struct TempSchema {
      type_: Option<String>,
      multiple_of: Option<f32>,
      minimum: Option<f64>,
      exclusive_minimum: Option<bool>,
      maximum: Option<f64>,
      exclusive_maximum: Option<bool>,
      min_length: Option<u64>,
      max_length: Option<u64>,
      pattern: Option<String>,
      items: Option<BoxedSchema>,
      properties: Option<BTreeMap<String, BoxedSchema>>,
      required: Option<Vec<String>>,
      additional_properties: Option<bool>
    }

    struct SchemaVisitor;

    impl Visitor for SchemaVisitor {
      type Value = BoxedSchema;

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapVisitor {
        let mut tmp_schema: TempSchema = Default::default();

        while let Some(key) = try!(visitor.visit_key()) {
          match key {
            Field::Type => { tmp_schema.type_ = Some(try!(visitor.visit_value())); },
            Field::MultipleOf => { tmp_schema.multiple_of = Some(try!(visitor.visit_value())); },
            Field::Minimum => { tmp_schema.minimum = Some(try!(visitor.visit_value())); },
            Field::ExclusiveMinimum => { tmp_schema.exclusive_minimum = Some(try!(visitor.visit_value())); },
            Field::Maximum => { tmp_schema.maximum = Some(try!(visitor.visit_value())); },
            Field::ExclusiveMaximum => { tmp_schema.exclusive_maximum = Some(try!(visitor.visit_value())); },
            Field::MinLength => { tmp_schema.min_length = Some(try!(visitor.visit_value())); },
            Field::MaxLength => { tmp_schema.max_length = Some(try!(visitor.visit_value())); },
            Field::Pattern => { tmp_schema.pattern = Some(try!(visitor.visit_value())); },
            Field::Items => { tmp_schema.items = Some(try!(visitor.visit_value())); },
            Field::Properties => { tmp_schema.properties = Some(try!(visitor.visit_value())); },
            Field::Required => { tmp_schema.required = Some(try!(visitor.visit_value())); },
            Field::AdditionalProperties => { tmp_schema.additional_properties = Some(try!(visitor.visit_value())); },
            Field::Enum => {
              let values: Vec<Value> = try!(visitor.visit_value());
              try!(visitor.end());
              return Ok(Box::new(Schema::enum_(values)));
            },
            Field::Ignore => { try!(visitor.visit_value::<IgnoredAny>()); }
          }
        }

        try!(visitor.end());

        if let Some(type_) = tmp_schema.type_ {
          match type_.as_str() {
            "null" => Ok(Box::new(Schema::null())),
            "boolean" => Ok(Box::new(Schema::boolean())),
            "number" | "integer" => {
              let mut schema = Schema::number();
              if type_ == "integer" { schema.set_multiple_of(1.0); }
              else if let Some(multiple_of) = tmp_schema.multiple_of { schema.set_multiple_of(multiple_of); }
              if let Some(minimum) = tmp_schema.minimum { schema.set_minimum(minimum); }
              if let Some(maximum) = tmp_schema.maximum { schema.set_maximum(maximum); }
              if tmp_schema.exclusive_minimum.unwrap_or(false) { schema.enable_exclusive_minimum(); }
              if tmp_schema.exclusive_maximum.unwrap_or(false) { schema.enable_exclusive_maximum(); }
              Ok(Box::new(schema))
            },
            "string" => {
              let mut schema = Schema::string();
              if let Some(min_length) = tmp_schema.min_length { schema.set_min_length(min_length); }
              if let Some(max_length) = tmp_schema.max_length { schema.set_max_length(max_length); }
              if let Some(pattern) = tmp_schema.pattern.and_then(|p| Regex::new(&p).ok()) { schema.set_pattern(pattern); }
              Ok(Box::new(schema))
            },
            "array" => {
              let mut schema = Schema::array();
              if let Some(items) = tmp_schema.items { schema.set_boxed_items(items); }
              Ok(Box::new(schema))
            },
            "object" => {
              let mut schema = Schema::object();
              schema.set_required(tmp_schema.required.unwrap_or_default());
              if tmp_schema.additional_properties.unwrap_or(false) { schema.enable_additional_properties(); }
              for (key, sub_schema) in tmp_schema.properties.unwrap_or_default() {
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

  use schema::{Definition, Type, Schema};

  #[test]
  fn test_json_deserialize_definition() {
    let from_str = serde_json::from_str::<Definition>;
    assert_eq!(from_str("{}").unwrap(), Definition::new());
    assert_eq!(from_str(r#"{"hello":"world"}"#).unwrap(), Definition::new());
    assert!(from_str(r#"{"types":2}"#).is_err());
    assert!(from_str(r#"{"types":"yo"}"#).is_err());
    assert!(from_str(r#"{"types":[]}"#).is_err());
    assert_eq!(from_str(r#"{"types":{}}"#).unwrap(), Definition::new());
  }

  #[test]
  fn test_json_deserialize_type() {
    let from_str = serde_json::from_str::<Type>;
    assert_eq!(from_str("{}").unwrap(), Type::new());
    assert_eq!(from_str(r#"{"hello":"world"}"#).unwrap(), Type::new());
    assert!(from_str(r#"{"schema":2}"#).is_err());
    assert!(from_str(r#"{"schema":"yo"}"#).is_err());
    assert!(from_str(r#"{"schema":[]}"#).is_err());
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
