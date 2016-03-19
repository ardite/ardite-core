//! Contains the full definition of a data system which Ardite will use.

use std::ops::Deref;
use schema::Schema;
use value::Key;

/// The definition object which contains all necessary information to
/// understand an Ardite Schema Definition.
#[derive(Debug)]
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

  /// Gets type of a certain name.
  pub fn find_type<K>(&self, tmp_name: K) -> Option<&Type> where K: Into<Key> {
    let name = tmp_name.into();
    self.types.iter().find(|type_| type_.name() == name)
  }
}

#[cfg(test)]
impl PartialEq<Definition> for Definition {
  fn eq(&self, other: &Self) -> bool {
    format!("{:?}", self) == format!("{:?}", other)
  }
}

/// Represents a high-level database type.
#[derive(Debug)]
pub struct Type {
  /// The name of the custom type.
  name: Key,
  /// The schema used to validate data which claims to be of this type.
  schema: Option<Box<Schema + 'static>>
}

impl Type {
  /// Create a new instance of `Type`.
  pub fn new<K>(name: K) -> Self where K: Into<Key> {
    Type {
      name: name.into(),
      schema: None
    }
  }

  /// Set the schema for the type. Polymorphic so it accepts any type which
  /// implements schema which gets boxed into a trait object. If you have a
  /// schema trait object, see `set_boxed_schema`.
  pub fn set_schema<S>(&mut self, schema: S) where S: Schema + 'static {
    self.schema = Some(Box::new(schema));
  }

  pub fn set_boxed_schema(&mut self, schema: Box<Schema>) {
    self.schema = Some(schema);
  }

  /// Gets the name of the type.
  pub fn name(&self) -> Key {
    self.name.to_owned()
  }

  /// Gets the schema of the type.
  pub fn schema(&self) -> Option<&Schema> {
    self.schema.as_ref().map(|schema| schema.deref())
  }
}

/// A function to be used by tests which creates a complete basic definition.
/// Should define the same schema which exists in the
/// `tests/fixtures/definitions/basic.json` file.
#[cfg(test)]
pub fn create_basic() -> Definition {
  use regex::Regex;
  use schema::Schema;

  // TODO: use order in file, not serde’s `BTreeMap` order.
  let mut definition = Definition::new();

  definition.add_type({
    let mut type_ = Type::new("person");
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

  definition.add_type({
    let mut type_ = Type::new("post");
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
