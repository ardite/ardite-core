use std::collections::BTreeMap;
use std::io::BufReader;
use std::path::PathBuf;
use std::fs::File;
use linear_map::LinearMap;
use serde_json;
use error::{Error, ErrorCode};
use definition::Definition;
use definition::schema::Schema;
use value::Value;

pub fn from_file(path: PathBuf) -> Result<Definition, Error> {
  let extension = path.extension().map_or("", |s| s.to_str().unwrap());
  let file = try!(File::open(&path));
  let reader = BufReader::new(file);
  match extension {
    "json" => {
      let data: SerdeDefinition = try!(serde_json::from_reader(reader));
      Ok(try!(data.to_definition()))
    },
    "yml" => Err(Error::unimplemented("YAML file parsing has not yet been implemented.")),
    _ => Err(Error {
      code: ErrorCode::NotAcceptable,
      message: String::from(format!("File extension '{}' cannot be deserialized in '{}'.", extension, path.display())),
      hint: Some(String::from(format!("Use a recognizable file extension like '.json' or '.yml'.")))
    })
  }
}

/// Type used to deserialize data files into a usable definition type.
#[derive(Deserialize)]
struct SerdeDefinition {
  data: SerdeSchema
}

impl SerdeDefinition {
  /// Transforms the intermediary type into the useful type.
  fn to_definition(self) -> Result<Definition, Error> {
    Ok(Definition {
      data: try!(self.data.to_schema())
    })
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
  // TODO: `enum` should not just accept strings.
  #[serde(rename="enum")]
  enum_: Option<Vec<String>>
}

impl SerdeSchema {
  /// Transforms the intermediary type into the useful type.
  fn to_schema(self) -> Result<Schema, Error> {
    match self.type_ {
      Some(type_) => match type_.as_ref() {
        "null" => Ok(Schema::Null),
        "boolean" => Ok(Schema::Boolean),
        "number" | "integer" => Ok(Schema::Number {
          multiple_of: self.multiple_of,
          minimum: self.minimum,
          exclusive_minimum: self.exclusive_minimum.unwrap_or(false),
          maximum: self.maximum,
          exclusive_maximum: self.exclusive_maximum.unwrap_or(false)
        }),
        "string" => Ok(Schema::String {
          min_length: self.min_length,
          max_length: self.max_length,
          pattern: self.pattern
        }),
        "array" => {
          if let Some(items) = self.items {
            Ok(Schema::Array {
              items: Box::new(try!(items.to_schema()))
            })
          } else {
            Err(Error::validation("Missing `items` property for type 'array'.", "Add a schema at `items`."))
          }
        },
        "object" => Ok(Schema::Object {
          required: self.required.unwrap_or(vec![]),
          additional_properties: self.additional_properties.unwrap_or(false),
          properties: {
            let mut map = LinearMap::new();
            for (key, definition) in self.properties.unwrap_or(BTreeMap::new()) {
              map.insert(key, try!(definition.to_schema()));
            }
            map
          }
        }),
        _ => Err(Error::validation(format!("Invalid type '{}'.", type_), format!("Use a permitted type like 'string' and not '{}'.", type_)))
      },
      None => {
        if let Some(enum_) = self.enum_ {
          Ok(Schema::Enum(enum_.into_iter().map(|s| Value::String(s)).collect()))
        } else {
          Err(Error::validation("No schema type specified.", "Set a `type` property or an `enum` property."))
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use definition::Definition;
  use definition::schema::Schema;
  use definition::serde::from_file;
  
  lazy_static! {
    static ref BASIC_DEFINITION: Definition = Definition {
      data: Schema::Object {
        required: vec![],
        additional_properties: false,
        properties: linear_map! {
          String::from("people") => Schema::Array {
            items: Box::new(Schema::Object {
              required: vec![String::from("email")],
              additional_properties: false,
              properties: linear_map! {
                String::from("email") => Schema::String {
                  min_length: Some(4),
                  max_length: Some(256),
                  pattern: Some(String::from(r".+@.+\..+"))
                },
                String::from("name") => Schema::String {
                  min_length: Some(2),
                  max_length: Some(64),
                  pattern: None
                }
              }
            })
          },
          String::from("posts") => Schema::Array {
            items: Box::new(Schema::Object {
              required: vec![String::from("headline")],
              additional_properties: false,
              properties: linear_map! {
                String::from("headline") => Schema::String {
                  min_length: Some(4),
                  max_length: Some(1024),
                  pattern: None
                },
                String::from("text") => Schema::String {
                  min_length: None,
                  max_length: Some(65536),
                  pattern: None
                },
                String::from("topic") => Schema::Enum(vec![vstring!("showcase"), vstring!("help"), vstring!("ama")])
              }
            })
          }
        }
      }
    };
  }
  
  #[test]
  fn test_basic_json() {
    assert_eq!(from_file(PathBuf::from("tests/fixtures/definitions/basic.json")).unwrap(), *BASIC_DEFINITION);
  }
  
  #[test]
  #[ignore]
  fn test_basic_yaml() {
    assert_eq!(from_file(PathBuf::from("tests/fixtures/definitions/basic.yml")).unwrap(), *BASIC_DEFINITION);
  }
}
