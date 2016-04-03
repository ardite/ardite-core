use std::collections::BTreeMap;

use regex::Regex;
use serde::de;
use serde::de::{Deserialize, Deserializer, Visitor, MapVisitor};
use serde::de::impls::IgnoredAny;
use url::Url;

use schema::{Definition, Collection, Driver, Schema};
use value::{Key, Value};

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

          fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: de::Error {
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
        let mut driver_config: Option<Driver> = None;
        let mut collections: Option<BTreeMap<Key, Collection>> = None;

        visit_map_fields!(visitor, {
          "driver" => driver_config,
          "collections" => collections
        });

        let mut definition = Definition::new();

        if let Some(driver_config) = driver_config { definition.set_driver(driver_config); }

        if let Some(collections) = collections {
          for (key, collection) in collections {
            definition.add_collection(key, collection);
          }
        }

        Ok(definition)
      }
    }

    deserializer.deserialize_map(DefinitionVisitor)
  }
}

impl Deserialize for Collection {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    struct CollectionVisitor;

    impl Visitor for CollectionVisitor {
      type Value = Collection;

      #[inline]
      fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapVisitor {
        let mut driver_config: Option<Driver> = None;
        let mut type_string: Option<String> = None;
        let mut properties: Option<BTreeMap<String, Box<Schema>>> = None;
        let mut required: Option<Vec<String>> = None;
        let mut additional_properties: Option<bool> = None;

        visit_map_fields!(visitor, {
          "driver" => driver_config,
          "type" => type_string,
          "properties" => properties,
          "required" => required,
          "additionalProperties" => additional_properties
        });

        if let Some(type_string) = type_string {
          if type_string != "object" {
            return Err(de::Error::custom(format!("Schema type must be 'object', not '{}'.", type_string)));
          }
        } else {
          return Err(de::Error::custom("Schema type property must be defined."));
        }

        let mut collection = Collection::new();

        collection.set_required(required.unwrap_or_default());
        if additional_properties.unwrap_or(false) { collection.enable_additional_properties(); }
        for (key, schema) in properties.unwrap_or_default() {
          collection.insert_boxed_property(key, schema);
        }

        if let Some(driver_config) = driver_config { collection.set_driver(driver_config); }

        Ok(collection)
      }
    }

    deserializer.deserialize_map(CollectionVisitor)
  }
}

impl Deserialize for Driver {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    struct DriverVisitor;

    impl Visitor for DriverVisitor {
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

    deserializer.deserialize(DriverVisitor)
  }
}

impl Deserialize for Box<Schema> {
  fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
    struct SchemaVisitor;

    impl Visitor for SchemaVisitor {
      type Value = Box<Schema>;

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
        let mut items: Option<Box<Schema>> = None;
        let mut properties: Option<BTreeMap<String, Box<Schema>>> = None;
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
                schema.insert_boxed_property(key, sub_schema);
              }
              Ok(Box::new(schema))
            },
            _ => Err(de::Error::custom(format!("Cannot use '{}' for a schema type property.", type_)))
          }
        } else {
          Err(de::Error::custom("No type property for schema was specified."))
        }
      }
    }

    deserializer.deserialize_map(SchemaVisitor)
  }
}

#[cfg(test)]
mod tests {
  use serde_json;
  use url::Url;

  use schema::{Definition, Collection, Driver};

  #[test]
  fn test_json_definition() {
    let from_str = serde_json::from_str::<Definition>;
    assert_eq!(from_str("{}").unwrap(), Definition::new());
    assert_eq!(from_str(r#"{"hello":"world"}"#).unwrap(), Definition::new());
    assert!(from_str(r#"{"collections":2}"#).is_err());
    assert!(from_str(r#"{"collections":"yo"}"#).is_err());
    assert!(from_str(r#"{"collections":[]}"#).is_err());
    assert_eq!(from_str(r#"{"collections":{}}"#).unwrap(), Definition::new());
  }

  #[test]
  fn test_json_collection() {
    let from_str = serde_json::from_str::<Collection>;
    assert_eq!(from_str(r#"{"type":"object"}"#).unwrap(), Collection::new());
    assert!(from_str("{}").is_err());
    assert!(from_str(r#"{"hello":"world"}"#).is_err());
    assert_eq!(from_str(r#"{"type":"object","hello":"world"}"#).unwrap(), Collection::new());
    assert!(from_str(r#"{"type":2}"#).is_err());
    assert!(from_str(r#"{"type":"yo"}"#).is_err());
    assert!(from_str(r#"{"type":[]}"#).is_err());
  }

  #[test]
  fn test_json_driver() {
    let from_str = serde_json::from_str::<Driver>;
    assert_eq!(from_str(r#""mongodb://localhost:27017""#).unwrap(), Driver::new(Url::parse("mongodb://localhost:27017").unwrap()));
    assert!(from_str(r#""not a url or a name""#).is_err());
  }
}
