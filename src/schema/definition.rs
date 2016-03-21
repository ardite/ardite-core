//! Contains the full definition of a data system which Ardite will use.

use std::collections::BTreeMap;
use std::io::BufReader;
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;

use serde_json;
use serde_yaml;
use url::Url;

use error::{Error, NotAcceptable};
use schema::{Schema, BoxedSchema};
use value::Key;

/// The definition object which contains all necessary information to
/// understand an Ardite Schema Definition.
#[derive(PartialEq, Debug)]
pub struct Definition {
  /// The default driver when one is not specified for a specific type.
  driver: Option<DriverConfig>,
  /// Types defined in the database.
  types: BTreeMap<Key, Type>
}

impl Definition {
  /// Creates a new empty instance of `Definition`.
  pub fn new() -> Self {
    Definition {
      driver: None,
      types: BTreeMap::new()
    }
  }

  /// Set the driver config.
  pub fn set_driver(&mut self, driver: DriverConfig) {
    self.driver = Some(driver);
  }

  /// Get the driver config.
  pub fn driver(&self) -> Option<&DriverConfig> {
    self.driver.as_ref()
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
        return Err(
          Error
          ::new(NotAcceptable, format!("File extension '{}' cannot be deserialized in '{}'.", extension, path.display()))
          .set_hint("Use a recognizable file extension like '.json' or '.yml'.")
        )
      }
    })
  }
}

/// Represents a high-level database type.
#[derive(PartialEq, Debug)]
pub struct Type {
  /// A type may optionally have its own driver.
  driver: Option<DriverConfig>,
  /// The schema used to validate data which claims to be of this type.
  // TODO: only allow object schemas with `SchemaObject`
  schema: Option<BoxedSchema>
}

impl Type {
  /// Create a new instance of `Type`.
  pub fn new() -> Self {
    Type {
      driver: None,
      schema: None
    }
  }

  /// Set the driver config.
  pub fn set_driver(&mut self, driver: DriverConfig) {
    self.driver = Some(driver);
  }

  /// Get the driver config.
  pub fn driver(&self) -> Option<&DriverConfig> {
    self.driver.as_ref()
  }

  /// Set the schema for the type. Polymorphic so it accepts any type which
  /// implements schema which gets boxed into a trait object. If you have a
  /// schema trait object, see `set_boxed_schema`.
  pub fn set_schema<S>(&mut self, schema: S) where S: Schema + 'static {
    self.schema = Some(Box::new(schema));
  }

  pub fn set_boxed_schema(&mut self, schema: BoxedSchema) {
    self.schema = Some(schema);
  }

  /// Gets the schema of the type.
  pub fn schema(&self) -> Option<&Schema> {
    self.schema.as_ref().map(|schema| schema.deref())
  }
}

/// Configuration for what driver to use and what URL to use to connect that
/// driver.
// TODO: can't finalize this until dynamic loading of drivers is implemented.
#[derive(PartialEq, Debug)]
pub struct DriverConfig {
  /// The URL to pass into the driver when connecting.
  url: Url
}

impl DriverConfig {
  /// Create a new driver config. Is only passed a URL and the scheme of the
  /// URL will be used for the name.
  pub fn new(url: Url) -> Self {
    DriverConfig {
      url: url
    }
  }

  /// Returns the URL to the driver.
  pub fn url(&self) -> &Url {
    &self.url
  }
}
