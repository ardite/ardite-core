//! Serializes and deserializes Ardite Schema Definitions from different data
//! formats such as JSON and YAML.

mod types;

use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;

use regex::Regex;
use serde_json;
use serde_yaml;

use error::{Error, ErrorCode};
use schema::{Definition, Type, Schema};
use schema::serde::types::*;

/// Gets an Ardite Schema Definition from a file. Aims to support mainly the
/// JSON and YAML formats.
// TODO: validate file against JSON schema.
pub fn from_file(path: PathBuf) -> Result<Definition, Error> {
  let extension = path.extension().map_or("", |s| s.to_str().unwrap());
  let file = try!(File::open(&path));
  let reader = BufReader::new(file);
  let definition: SerdeDefinition = match extension {
    "json" => try!(serde_json::from_reader(reader)),
    "yml" => try!(serde_yaml::from_reader(reader)),
    _ => {
      return Err(Error::new(
        ErrorCode::NotAcceptable,
        format!("File extension '{}' cannot be deserialized in '{}'.", extension, path.display()),
        Some("Use a recognizable file extension like '.json' or '.yml'.".to_owned())
      ))
    }
  };
  serde_definition_into_definition(definition)
}

/// Transforms the intermediary type into the useful type.
fn serde_definition_into_definition(serde_definition: SerdeDefinition) -> Result<Definition, Error> {
  let mut definition = Definition::new();

  for (key, schema) in serde_definition.types.into_iter() {
    let mut type_ = Type::new();
    type_.set_boxed_schema(try!(serde_schema_into_schema(schema)));
    definition.add_type(key, type_);
  }

  Ok(definition)
}

/// Transforms the intermediary type into the useful type.
fn serde_schema_into_schema(serde_schema: SerdeSchema) -> Result<Box<Schema + 'static>, Error> {
  match serde_schema.type_ {
    Some(type_) => match type_.as_str() {
      "null" => Ok(Box::new(Schema::null())),
      "boolean" => Ok(Box::new(Schema::boolean())),
      type_ @ "number" | type_ @ "integer" => {
        let mut schema = Schema::number();
        if type_ == "integer" { schema.set_multiple_of(1.0); }
        else if let Some(multiple_of) = serde_schema.multiple_of { schema.set_multiple_of(multiple_of); }
        if let Some(minimum) = serde_schema.minimum { schema.set_minimum(minimum); }
        if let Some(maximum) = serde_schema.maximum { schema.set_maximum(maximum); }
        if serde_schema.exclusive_minimum.unwrap_or(false) { schema.enable_exclusive_minimum(); }
        if serde_schema.exclusive_maximum.unwrap_or(false) { schema.enable_exclusive_maximum(); }
        Ok(Box::new(schema))
      },
      "string" => {
        let mut schema = Schema::string();
        if let Some(min_length) = serde_schema.min_length { schema.set_min_length(min_length); }
        if let Some(max_length) = serde_schema.max_length { schema.set_max_length(max_length); }
        if let Some(pattern) = serde_schema.pattern.and_then(|p| Regex::new(&p).ok()) { schema.set_pattern(pattern); }
        Ok(Box::new(schema))
      },
      "array" => {
        let mut schema = Schema::array();
        if let Some(items) = serde_schema.items { schema.set_boxed_items(try!(serde_schema_into_schema(*items))); }
        Ok(Box::new(schema))
      },
      "object" => {
        let mut schema = Schema::object();
        schema.set_required(serde_schema.required.unwrap_or_default());
        if serde_schema.additional_properties.unwrap_or(false) { schema.enable_additional_properties(); }
        for (key, value) in serde_schema.properties.unwrap_or_default() {
          schema.add_boxed_property(key, try!(serde_schema_into_schema(value)));
        }
        Ok(Box::new(schema))
      },
      _ => Err(Error::invalid(
        format!("Invalid type '{}'.", type_),
        format!("Use a permitted type like 'string' and not '{}'.", type_)
      ))
    },
    None => {
      if let Some(enum_) = serde_schema.enum_ {
        Ok(Box::new(Schema::enum_(enum_)))
      } else {
        Err(Error::invalid("No schema type specified.", "Set a `type` property or an `enum` property."))
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::from_file;

  use std::path::PathBuf;

  use schema::definition::create_basic;

  #[test]
  fn test_basic_json() {
    assert_eq!(from_file(PathBuf::from("tests/fixtures/definitions/basic.json")).unwrap(), create_basic());
  }

  #[test]
  fn test_basic_yaml() {
    assert_eq!(from_file(PathBuf::from("tests/fixtures/definitions/basic.yml")).unwrap(), create_basic());
  }
}
