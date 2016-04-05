use std::collections::BTreeMap;

use inflections::Inflect;
use url::Url;

#[derive(PartialEq, Debug)]
pub struct Schema {
  driver: Option<Driver>,
  types: BTreeMap<String, Type>
}

impl Schema {
  pub fn new() -> Self {
    Schema {
      driver: None,
      types: BTreeMap::new()
    }
  }

  pub fn set_driver(&mut self, driver: Driver) {
    self.driver = Some(driver);
  }

  pub fn driver(&self) -> Option<&Driver> {
    self.driver.as_ref()
  }

  pub fn insert_type<N>(&mut self, name: N, type_: Type) where N: Inflect {
    self.types.insert(name.to_snake_case(), type_);
  }

  pub fn get_type(&self, name: &str) -> Option<&Type> {
    self.types.get(name)
  }

  pub fn types(&self) -> &BTreeMap<String, Type> {
    &self.types
  }
}

#[derive(PartialEq, Debug)]
pub struct Type {
  properties: Vec<String>
}

impl Type {
  pub fn new() -> Self {
    Type {
      properties: Vec::new()
    }
  }
}

/// Configuration for what driver to use and what URL to use to connect that
/// driver.
// TODO: can't finalize this until dynamic loading of drivers is implemented.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
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
    let mut schema = Schema::new();
    schema.insert_type("helloWorld", Type::new());
    schema.insert_type("yo yo", Type::new());
    schema.insert_type("COOL_COOL", Type::new());
    assert!(schema.get_type("helloWorld").is_none());
    assert!(schema.get_type("yo yo").is_none());
    assert!(schema.get_type("COOL_COOL").is_none());
    assert!(schema.get_type("hello_world").is_some());
    assert!(schema.get_type("yo_yo").is_some());
    assert!(schema.get_type("cool_cool").is_some());
  }
}
