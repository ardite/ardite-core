//! Contains the full definition of a data system which Ardite will use.

use schema::Schema;
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
  pub fn new<I>(name: I, schema: Schema) -> Self where I: Into<Key> {
    Type {
      name: name.into(),
      schema: schema
    }
  }
}

/// A function to be used by tests which creates a complete basic definition.
/// Should define the same schema which exists in the
/// `tests/fixtures/definitions/basic.json` file.
#[cfg(test)]
pub fn create_basic() -> Definition {
  use regex::Regex;
  use schema::Schema;

  let mut definition = Definition::new();

  definition.add_type(Type::new("person", {
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
    person
  }));

  definition.add_type(Type::new("post", {
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
    post.add_property("topic", Schema::enum_(vec!["showcase", "help", "ama"]));
    post
  }));

  definition
}
