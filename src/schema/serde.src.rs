//! Serializes and deserializes Ardite Schema Definitions from different data
//! formats such as JSON and YAML.

use std::collections::BTreeMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::fs::File;
use regex::Regex;
use serde_json;
use serde_yaml;
use error::{Error, ErrorCode};
use schema::{Definition, Type, Schema};
use value::Value;

/// Gets an Ardite Schema Definition from a file. Aims to support mainly the
/// JSON and YAML formats.
pub fn from_file(path: PathBuf) -> Result<Definition, Error> {
  let extension = path.extension().map_or("", |s| s.to_str().unwrap());
  let mut file = try!(File::open(&path));
  match extension {
    "json" => {
      let reader = BufReader::new(file);
      let data: SerdeDefinition = try!(serde_json::from_reader(reader));
      Ok(try!(data.into_definition()))
    },
    "yml" => {
      let mut string = String::new();
      try!(file.read_to_string(&mut string));
      let data: SerdeDefinition = try!(serde_yaml::from_str(&string));
      Ok(try!(data.into_definition()))
    },
    _ => Err(Error::new(
      ErrorCode::NotAcceptable,
      format!("File extension '{}' cannot be deserialized in '{}'.", extension, path.display()),
      Some("Use a recognizable file extension like '.json' or '.yml'.".to_owned())
    ))
  }
}

/// SchemaType used to deserialize data files into a usable definition type.
#[derive(Deserialize)]
struct SerdeDefinition {
  types: BTreeMap<String, SerdeSchema>
}

impl SerdeDefinition {
  /// Transforms the intermediary type into the useful type.
  fn into_definition(self) -> Result<Definition, Error> {
    let mut definition = Definition::new();

    for (key, value) in self.types.into_iter() {
      let mut type_ = Type::new(key);
      type_.set_boxed_schema(try!(value.into_schema()));
      definition.add_type(type_);
    }

    Ok(definition)
  }
}

/// Intermediary type used to deserialized data files into a usable schema
/// enum.
#[derive(Deserialize)]
struct SerdeSchema {
  #[serde(rename="type")]
  type_: Option<String>,
  #[serde(rename="multipleOf")]
  multiple_of: Option<f32>,
  minimum: Option<f64>,
  #[serde(rename="exclusiveMinimum")]
  exclusive_minimum: Option<bool>,
  maximum: Option<f64>,
  #[serde(rename="exclusiveMaximum")]
  exclusive_maximum: Option<bool>,
  #[serde(rename="minLength")]
  min_length: Option<u64>,
  #[serde(rename="maxLength")]
  max_length: Option<u64>,
  pattern: Option<String>,
  items: Option<Box<SerdeSchema>>,
  properties: Option<BTreeMap<String, SerdeSchema>>,
  required: Option<Vec<String>>,
  #[serde(rename="additionalProperties")]
  additional_properties: Option<bool>,
  #[serde(rename="enum")]
  enum_: Option<Vec<String>>
}

impl SerdeSchema {
  /// Transforms the intermediary type into the useful type.
  fn into_schema(self) -> Result<Box<Schema>, Error> {
    match self.type_ {
      Some(type_) => match type_.as_ref() {
        "null" => Ok(Box::new(Schema::null())),
        "boolean" => Ok(Box::new(Schema::boolean())),
        type_ @ "number" | type_ @ "integer" => {
          let mut schema = Schema::number();
          if type_ == "integer" { schema.set_multiple_of(1.0); }
          else if let Some(multiple_of) = self.multiple_of { schema.set_multiple_of(multiple_of); }
          if let Some(minimum) = self.minimum { schema.set_minimum(minimum); }
          if let Some(maximum) = self.maximum { schema.set_maximum(maximum); }
          if self.exclusive_minimum.unwrap_or(false) { schema.enable_exclusive_minimum(); }
          if self.exclusive_maximum.unwrap_or(false) { schema.enable_exclusive_maximum(); }
          Ok(Box::new(schema))
        },
        "string" => {
          let mut schema = Schema::string();
          if let Some(min_length) = self.min_length { schema.set_min_length(min_length); }
          if let Some(max_length) = self.max_length { schema.set_min_length(max_length); }
          if let Some(pattern) = self.pattern.and_then(|p| Regex::new(&p).ok()) { schema.set_pattern(pattern); }
          Ok(Box::new(schema))
        },
        "array" => {
          let mut schema = Schema::array();
          if let Some(items) = self.items { schema.set_boxed_items(try!((*items).into_schema())); }
          Ok(Box::new(schema))
        },
        "object" => {
          let mut schema = Schema::object();
          schema.set_required(self.required.unwrap_or_else(|| vec![]));
          if self.additional_properties.unwrap_or(false) { schema.enable_additional_properties(); }
          for (key, serde) in self.properties.unwrap_or_default() {
            schema.add_boxed_property(key, try!(serde.into_schema()));
          }
          Ok(Box::new(schema))
        },
        _ => Err(Error::validation(
          format!("Invalid type '{}'.", type_),
          format!("Use a permitted type like 'string' and not '{}'.", type_)
        ))
      },
      None => {
        if let Some(enum_) = self.enum_ {
          Ok(Box::new(Schema::enum_(enum_.into_iter().map(Value::String).collect())))
        } else {
          Err(Error::validation("No schema type specified.", "Set a `type` property or an `enum` property."))
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::from_file;
  use std::path::PathBuf;

  #[test]
  fn test_basic_json() {
    // TODO: use `assert_eq` when `Schema` implements `PartialEq`.
    assert!(from_file(PathBuf::from("tests/fixtures/definitions/basic.json")).is_ok());
  }

  #[test]
  fn test_basic_yaml() {
    // TODO: use `assert_eq` when `Schema` implements `PartialEq`.
    assert!(from_file(PathBuf::from("tests/fixtures/definitions/basic.yml")).is_ok());
  }
}
