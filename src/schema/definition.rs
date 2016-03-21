//! Contains the full definition of a data system which Ardite will use.

use std::collections::BTreeMap;
use std::io::BufReader;
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;

use serde_json;
use serde_yaml;

use error::{Error, NotAcceptable};
use schema::Schema;
use value::Key;

/// The definition object which contains all necessary information to
/// understand an Ardite Schema Definition.
#[derive(Debug)]
pub struct Definition {
  /// Types defined in the database.
  types: BTreeMap<Key, Type>
}

impl Definition {
  /// Creates a new empty instance of `Definition`.
  pub fn new() -> Self {
    Definition {
      types: BTreeMap::new()
    }
  }

  /// Add a new type to the `Definition`.
  pub fn add_type<K>(&mut self, name: K, type_: Type) where K: Into<Key> {
    self.types.insert(name.into(), type_);
  }

  /// Gets type of a certain name.
  pub fn get_type<'a, K>(&self, name: K) -> Option<&Type> where K: Into<&'a Key> {
    self.types.get(name.into())
  }

  /// Gets an Ardite Schema Definition from a file. Aims to support mainly the
  /// JSON and YAML formats.
  // TODO: validate file against JSON schema.
  pub fn from_file(path: PathBuf) -> Result<Definition, Error> {
    let extension = path.extension().map_or("", |s| s.to_str().unwrap());
    let file = try!(File::open(&path));
    let reader = BufReader::new(file);
    Ok(match extension {
      "json" => try!(serde_json::from_reader(reader)),
      "yml" => try!(serde_yaml::from_reader(reader)),
      _ => {
        return Err(Error::new(
          NotAcceptable,
          format!("File extension '{}' cannot be deserialized in '{}'.", extension, path.display()),
          Some("Use a recognizable file extension like '.json' or '.yml'.".to_owned())
        ))
      }
    })
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
  /// The schema used to validate data which claims to be of this type.
  schema: Option<Box<Schema + 'static>>
}

impl Type {
  /// Create a new instance of `Type`.
  pub fn new() -> Self {
    Type {
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

  /// Gets the schema of the type.
  pub fn schema(&self) -> Option<&Schema> {
    self.schema.as_ref().map(|schema| schema.deref())
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use regex::Regex;

  use schema::{Definition, Type, Schema};

  fn create_basic_definition() -> Definition {
    // TODO: use order in file, not serde’s `BTreeMap` order.
    let mut definition = Definition::new();

    definition.add_type("person", {
      let mut type_ = Type::new();
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

    definition.add_type("post", {
      let mut type_ = Type::new();
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

  #[test]
  fn test_basic_json() {
    assert_eq!(
      Definition::from_file(PathBuf::from("tests/fixtures/definitions/basic.json")).unwrap(),
      create_basic_definition()
    );
  }

  #[test]
  fn test_basic_yaml() {
    assert_eq!(
      Definition::from_file(PathBuf::from("tests/fixtures/definitions/basic.yml")).unwrap(),
      create_basic_definition()
    );
  }
}
