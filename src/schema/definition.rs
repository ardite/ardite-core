//! Contains the full definition of a data system which Ardite will use.
// TODO: Remove `pub` fields and add a DSL for building definitions.

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
#[derive(PartialEq, Clone, Debug)]
pub struct Type {
  /// The name of the custom type.
  pub name: String,
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
      str!("person") => Type {
        name: str!("person"),
        schema: Schema {
          type_: SchemaType::Object {
            required: vec![str!("email")],
            additional_properties: false,
            properties: linear_map! {
              str!("email") => Schema {
                type_: SchemaType::String {
                  min_length: Some(4),
                  max_length: Some(256),
                  pattern: Some(Regex::new(r".+@.+\..+").unwrap())
                }
              },
              str!("name") => Schema {
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
      str!("post") => Type {
        name: str!("post"),
        schema: Schema {
          type_: SchemaType::Object {
            required: vec![str!("headline")],
            additional_properties: false,
            properties: linear_map! {
              str!("headline") => Schema {
                type_: SchemaType::String {
                  min_length: Some(4),
                  max_length: Some(1024),
                  pattern: None
                }
              },
              str!("text") => Schema {
                type_: SchemaType::String {
                  min_length: None,
                  max_length: Some(65536),
                  pattern: None
                }
              },
              str!("topic") => Schema {
                type_: SchemaType::Enum(vec![vstring!("showcase"), vstring!("help"), vstring!("ama")])
              }
            }
          }
        }
      }
    }
  }
}
