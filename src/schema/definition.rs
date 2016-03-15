//! Contains the full definition of a data system which Ardite will use.

use schema::{Schema, SchemaType};
use value::Key;

/// The definition object which contains all necessary information to
/// understand an Ardite Schema Definition.
#[derive(PartialEq, Debug)]
pub struct Definition {
  /// Types defined in the database.
  types: Vec<Type>
}

impl Definition {
  /// Creates a new empty instance of `Definition`.
  pub fn new() -> Self {
    Definition {
      types: Vec::new()
    }
  }

  /// Add a new type to the `Definition`.
  pub fn add_type(&mut self, type_: Type) {
    self.types.push(type_);
  }
}

/// Represents a high-level database type.
#[derive(PartialEq, Clone, Debug)]
pub struct Type {
  /// The name of the custom type.
  name: Key,
  /// The schema used to validate data which claims to be of this type.
  schema: Schema
}

impl Type {
  /// Create a new instance of `Type`.
  pub fn new<I>(name: I) -> Self where I: Into<Key> {
    Type {
      name: name.into(),
      schema: Schema {
        type_: SchemaType::None
      }
    }
  }

  /// Set the schema for `Type`.
  pub fn set_schema(&mut self, schema: Schema) {
    self.schema = schema;
  }
}

/// A function to be used by tests which creates a complete basic definition.
/// Should define the same schema which exists in the
/// `tests/fixtures/definitions/basic.json` file.
#[cfg(test)]
pub fn create_basic() -> Definition {
  use regex::Regex;
  use schema::{Schema, SchemaType};

  let mut definition = Definition::new();
  let mut person_type = Type::new("person");
  let mut post_type = Type::new("post");

  person_type.set_schema(Schema {
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
  });

  post_type.set_schema(Schema {
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
  });

  definition.add_type(person_type);
  definition.add_type(post_type);

  definition
}
