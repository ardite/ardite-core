use std::collections::BTreeMap;
use std::ops::Deref;
use std::path::PathBuf;

use linear_map::LinearMap;

use error::Error;
use schema;
use schema::{Definition, Type};
use driver::{discover_driver, Driver, Memory};
use query::{Condition, SortRule, Range, Query};
use value::{Key, Value, Iter};

pub struct Service<'a> {
  definition: Definition,
  memory: Memory,
  /// A map of driver configs to their respective drivers. We use a `LinearMap`
  /// because it does not require the `DriverConfig` to implement anything
  /// crazy like `Hash` or `Ord`. We also don’t ever suspect having a large
  /// number of drivers.
  drivers: LinearMap<&'a schema::Driver, Box<Driver>>
}

impl<'a> Service<'a> {
  pub fn new(definition: Definition) -> Self {
    Service {
      definition: definition,
      memory: Memory::new(),
      drivers: LinearMap::new()
    }
  }

  pub fn from_file(path: PathBuf) -> Result<Self, Error> {
    Ok(Service::new(try!(Definition::from_file(path))))
  }

  /// Iterates through the `DriverConfig`s in the definition, connecting them,
  /// and storing them internally. After running this method, all drivers
  /// outside of memory will be connected.
  pub fn connect_drivers(&'a mut self) -> Result<(), Error> {
    let mut driver_configs = Vec::new();

    // Add the driver config for the definition.
    if let Some(default_driver) = self.definition.driver() {
      driver_configs.push(default_driver);
    }

    // Add the driver configs for the types.
    for (_, type_) in self.definition.types() {
      if let Some(type_driver) = type_.driver() {
        driver_configs.push(type_driver);
      }
    }

    // Discover and connect all of the drivers specified in the driver configs.
    for driver_config in driver_configs.into_iter() {
      self.drivers.insert(driver_config, try!(discover_driver(driver_config)));
    }

    Ok(())
  }

  pub fn definition(&self) -> &Definition {
    &self.definition
  }

  #[inline] pub fn get_type(&self, name: &str) -> Option<&Type> { self.definition.get_type(name) }
  #[inline] pub fn types(&self) -> &BTreeMap<Key, Type> { self.definition.types() }

  #[inline]
  pub fn read(
    &self,
    type_name: &Key,
    condition: Condition,
    sort: Vec<SortRule>,
    range: Range,
    query: Query
  ) -> Result<Iter, Error> {
    let type_ = try!(
      self.get_type(type_name)
      .ok_or(Error::not_found(format!("Can’t read for type '{}', because it does not exist in the schema.", type_name)))
    );
    try!(type_.validate_query(&query));
    let driver: &Driver = type_.driver().and_then(|config| self.drivers.get(config)).map_or(&self.memory, Deref::deref);
    driver.read(type_name, condition, sort, range, query)
  }

  #[inline]
  pub fn read_one(
    &self,
    type_name: &Key,
    condition: Condition,
    query: Query
  ) -> Result<Value, Error> {
    let type_ = try!(
      self.get_type(type_name)
      .ok_or(Error::not_found(format!("Can’t read for type '{}', because it does not exist in the schema.", type_name)))
    );
    try!(type_.validate_query(&query));
    let driver: &Driver = type_.driver().and_then(|config| self.drivers.get(config)).map_or(&self.memory, Deref::deref);
    driver.read_one(type_name, condition, query)
  }
}
