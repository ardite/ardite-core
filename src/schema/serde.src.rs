//! Serializes and deserializes Ardite Schema Definitions from different data
//! formats such as JSON and YAML.

use std::collections::BTreeMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::fs::File;
use linear_map::LinearMap;
use regex::Regex;
use serde_json;
use serde_yaml;
use error::{Error, ErrorCode};
use schema::{Definition, Type, Schema, SchemaType};
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
      Ok(try!(data.into()))
    },
    "yml" => {
      let mut string = String::new();
      try!(file.read_to_string(&mut string));
      let data: SerdeDefinition = try!(serde_yaml::from_str(&string));
      Ok(try!(data.into()))
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

impl Into<Result<Definition, Error>> for SerdeDefinition {
  /// Transforms the intermediary type into the useful type.
  fn into(self) -> Result<Definition, Error> {
    let mut definition = Definition::new();

    for (key, value) in self.types.into_iter() {
      let mut type_ = Type::new(key);
      type_.set_schema(try!(value.into()));
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

impl Into<Result<Schema, Error>> for SerdeSchema {
  /// Transforms the intermediary type into the useful type.
  fn into(self) -> Result<Schema, Error> {
    match self.type_ {
      Some(type_) => match type_.as_ref() {
        "null" => Ok(Schema {
          type_: SchemaType::Null
        }),
        "boolean" => Ok(Schema {
          type_: SchemaType::Boolean
        }),
        "number" | "integer" => Ok(Schema {
          type_: SchemaType::Number {
            multiple_of: self.multiple_of,
            minimum: self.minimum,
            exclusive_minimum: self.exclusive_minimum.unwrap_or(false),
            maximum: self.maximum,
            exclusive_maximum: self.exclusive_maximum.unwrap_or(false)
          }
        }),
        "string" => Ok(Schema {
          type_: SchemaType::String {
            min_length: self.min_length,
            max_length: self.max_length,
            pattern: self.pattern.map_or(None, |pattern| Regex::new(&pattern).ok())
          }
        }),
        "array" => {
          if let Some(items) = self.items {
            Ok(Schema {
              type_: SchemaType::Array {
                items: Box::new(try!((*items).into()))
              }
            })
          } else {
            Err(Error::validation("Missing `items` property for type 'array'.", "Add a schema at `items`."))
          }
        },
        "object" => Ok(Schema {
          type_: SchemaType::Object {
            required: self.required.unwrap_or_else(|| vec![]),
            additional_properties: self.additional_properties.unwrap_or(false),
            properties: {
              let mut map = LinearMap::new();
              for (key, definition) in self.properties.unwrap_or_default() {
                map.insert(key, try!(definition.into()));
              }
              map
            }
          }
        }),
        _ => Err(Error::validation(
          format!("Invalid type '{}'.", type_),
          format!("Use a permitted type like 'string' and not '{}'.", type_)
        ))
      },
      None => {
        if let Some(enum_) = self.enum_ {
          Ok(Schema {
            type_: SchemaType::Enum(enum_.into_iter().map(Value::String).collect())
          })
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
