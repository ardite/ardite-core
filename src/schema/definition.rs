//! The full definition of a data system which Ardite will use to provide
//! powerful services.

use std::collections::BTreeMap;
use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;

use inflections::case::to_snake_case;
use linear_map::LinearMap;
use serde_json;
use serde_yaml;
use url::Url;

use error::{Error, NotAcceptable};
use query::Query;
use schema::{Schema, SchemaObject};

/// The definition object which contains all necessary information to
/// understand an Ardite Schema Definition.
#[derive(PartialEq, Debug)]
pub struct Definition {
  /// The default driver when one is not specified for a specific type.
  driver: Option<Driver>,
  /// Collections defined in the database.
  collections: BTreeMap<String, Collection>
}

impl Definition {
  /// Creates a new empty instance of `Definition`.
  pub fn new() -> Self {
    Definition {
      driver: None,
      collections: BTreeMap::new()
    }
  }

  /// Set the definition’s driver config.
  pub fn set_driver(&mut self, driver: Driver) {
    self.driver = Some(driver);
  }

  /// Get the definition’s driver config.
  pub fn driver(&self) -> Option<&Driver> {
    self.driver.as_ref()
  }

  /// Add a new collection to the `Definition`. Whatever name gets passed in
  /// will automatically get converted to snake case. Keys get converted because
  /// we want to guarantee that all type keys inserted are in the snake case
  /// style. By making this guarantee we allow our services to be flexible with
  /// the names they display.
  ///
  /// If the case conversions were in the deserialization code this guarantee
  /// could not be made.
  ///
  /// # Example
  /// ```rust
  /// use ardite::schema::{Definition, Collection};
  ///
  /// let mut definition = Definition::new();
  ///
  /// definition.add_collection("helloWorld", Collection::new());
  ///
  /// assert!(definition.get_collection("helloWorld").is_none());
  /// assert_eq!(definition.get_collection("hello_world").unwrap(), &Collection::new());
  /// ```
  pub fn add_collection<N>(&mut self, name: N, collection: Collection) where N: Into<String> {
    self.collections.insert(to_snake_case(&name.into()), collection);
  }

  /// Gets the collection of a certain name.
  pub fn get_collection(&self, name: &str) -> Option<&Collection> {
    self.collections.get(name)
  }

  /// Gets all of the definition’s types.
  pub fn collections(&self) -> &BTreeMap<String, Collection> {
    &self.collections
  }

  /// Gets an Ardite Schema Definition from a file. Aims to support mainly the
  /// JSON and YAML formats.
  // TODO: validate file against JSON schema.
  pub fn from_file(path: PathBuf) -> Result<Definition, Error> {
    if !path.exists() {
      return Err(Error::not_found(format!("Schema definition file not found at '{}'.", path.display())))
    }
    let extension = path.extension().map_or("", |s| s.to_str().unwrap());
    let file = try!(File::open(&path));
    let reader = BufReader::new(file);
    Ok(match extension {
      "json" => try!(serde_json::from_reader(reader)),
      "yml" => try!(serde_yaml::from_reader(reader)),
      _ => {
        return Err(
          Error
          ::new(NotAcceptable, format!("File extension '{}' cannot be deserialized in '{}'.", extension, path.display()))
          .set_hint("Use a readable file extension like '.json' or '.yml'.")
        )
      }
    })
  }
}

/// Represents a high-level database type.
#[derive(PartialEq, Debug)]
pub struct Collection {
  /// A type may optionally have its own driver.
  driver: Option<Driver>,
  /// The schema used to validate data which claims to be of this type.
  schema: SchemaObject
}

impl Collection {
  /// Create a new instance of `Type`.
  pub fn new() -> Self {
    Collection {
      driver: None,
      schema: SchemaObject::new()
    }
  }

  /// Set the type’s driver config.
  pub fn set_driver(&mut self, driver: Driver) {
    self.driver = Some(driver);
  }

  /// Get the type’s driver config.
  pub fn driver(&self) -> Option<&Driver> {
    self.driver.as_ref()
  }

  #[inline] pub fn get<'a>(&'a self, key: &str) -> Option<&'a Schema> { self.schema.get(key) }
  #[inline] pub fn get_path<'a>(&'a self, path: &[&str]) -> Option<&'a Schema> { self.schema.get_path(path) }
  #[inline] pub fn validate_query(&self, query: &Query) -> Result<(), Error> { self.schema.validate_query(query) }

  /// Inserts a property into the underlying object schema. See the docs for
  /// `SchemaObject::insert_property` for more information.
  #[inline] pub fn insert_property<K, S>(&mut self, key: K, schema: S) where K: Into<String>, S: Schema { self.schema.insert_property(key, schema); }

  /// Inserts a boxed property into the underlying object schema. See the docs
  /// for `SchemaObject::insert_boxed_property` for more information.
  #[inline] pub fn insert_boxed_property<K>(&mut self, key: K, schema: Box<Schema>) where K: Into<String> { self.schema.insert_boxed_property(key, schema); }

  /// Gets the properties from the underlying object schema. See the docs for
  /// `SchemaObject::properties` fro more information.
  #[inline] pub fn properties(&self) -> &LinearMap<String, Box<Schema>> { self.schema.properties() }

  /// Sets the required property keys in the underlying object schema. See the
  /// docs for `SchemaObject::set_required` for more information.
  #[inline] pub fn set_required<K>(&mut self, required: Vec<K>) where K: Into<String> { self.schema.set_required(required) }

  /// Gets the required properties in the underlying object schema. See the
  /// docs for `SchemaObject::required` for more information.
  #[inline] pub fn required(&self) -> &Vec<String> { self.schema.required() }

  /// Enable additional properties in the underlying object schema. See the
  /// docs for `SchemaObject::enable_additional_properties` for more
  /// information.
  #[inline] pub fn enable_additional_properties(&mut self) { self.schema.enable_additional_properties() }

  /// Question if whether additional properties are enabled in the underlying
  /// object schema. See the docs for `SchemaObject::additional_properties` for
  /// more information.
  #[inline] pub fn additional_properties(&self) -> bool { self.schema.additional_properties() }
}

/// Configuration for what driver to use and what URL to use to connect that
/// driver.
// TODO: can't finalize this until dynamic loading of drivers is implemented.
#[derive(Eq, PartialEq, Debug)]
pub struct Driver {
  /// The URL to pass into the driver when connecting.
  url: Url
}

impl Driver {
  /// Create a new driver config. Is only passed a URL and the scheme of the
  /// URL will be used for the name.
  pub fn new(url: Url) -> Self {
    Driver {
      url: url
    }
  }

  /// Returns the URL to the driver.
  pub fn url(&self) -> &Url {
    &self.url
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert_type_will_snake_case() {
    let mut definition = Definition::new();
    definition.add_collection("helloWorld", Collection::new());
    definition.add_collection("yo yo", Collection::new());
    definition.add_collection("COOL_COOL", Collection::new());
    assert!(definition.get_collection("helloWorld").is_none());
    assert!(definition.get_collection("yo yo").is_none());
    assert!(definition.get_collection("COOL_COOL").is_none());
    assert!(definition.get_collection("hello_world").is_some());
    assert!(definition.get_collection("yo_yo").is_some());
    assert!(definition.get_collection("cool_cool").is_some());
  }
}
