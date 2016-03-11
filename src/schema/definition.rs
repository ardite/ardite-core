//! Contains the full definition of a data system which Ardite will use.

use linear_map::LinearMap;
use schema::Schema;
use value::Key;

/// The definition object which contains all necessary information to
/// understand an Ardite Schema Definition.
#[derive(PartialEq, Debug)]
pub struct Definition {
  /// Types defined in the database.
  pub types: LinearMap<Key, Type>
}

/// Represents a high-level database type.
#[derive(PartialEq, Debug)]
pub struct Type {
  /// The schema used to validate data which claims to be of this type.
  pub schema: Schema
}

/// A function to be used by tests which creates a complete basic definition.
/// Should define the same schema which exists in the
/// `tests/fixtures/definitions/basic.json` file.
#[cfg(test)]
pub fn create_basic() -> Definition {
  use regex::Regex;
  use schema::*;

  Definition {
    types: linear_map! {
      S!("person") => Type {
        schema: Schema {
          type_: SchemaType::Object {
            required: vec![S!("email")],
            additional_properties: false,
            properties: linear_map! {
              S!("email") => Schema {
                type_: SchemaType::String {
                  min_length: Some(4),
                  max_length: Some(256),
                  pattern: Some(Regex::new(r".+@.+\..+").unwrap())
                }
              },
              S!("name") => Schema {
                type_: SchemaType::String {
                  min_length: Some(2),
                  max_length: Some(64),
                  pattern: None
                }
              }
            }
          }
        }
      },
      S!("post") => Type {
        schema: Schema {
          type_: SchemaType::Object {
            required: vec![S!("headline")],
            additional_properties: false,
            properties: linear_map! {
              S!("headline") => Schema {
                type_: SchemaType::String {
                  min_length: Some(4),
                  max_length: Some(1024),
                  pattern: None
                }
              },
              S!("text") => Schema {
                type_: SchemaType::String {
                  min_length: None,
                  max_length: Some(65536),
                  pattern: None
                }
              },
              S!("topic") => Schema {
                type_: SchemaType::Enum(vec![vstring!("showcase"), vstring!("help"), vstring!("ama")])
              }
            }
          }
        }
      }
    }
  }
}
