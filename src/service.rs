use std::collections::BTreeMap;
use std::ops::Deref;
use std::path::PathBuf;

use linear_map::LinearMap;

use error::Error;
use schema;
use schema::{Definition, Collection};
use driver::{discover_driver, Driver, Memory};
use query::{Condition, Sort, Range};
use value::{Value, Iter};

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
    if let Some(default) = self.definition.driver() {
      driver_configs.push(default);
    }

    // Add the driver configs for the types.
    for (_, collection) in self.definition.collections() {
      if let Some(driver) = collection.driver() {
        driver_configs.push(driver);
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

  #[inline] pub fn get_collection(&self, name: &str) -> Option<&Collection> { self.definition.get_collection(name) }
  #[inline] pub fn collections(&self) -> &BTreeMap<String, Collection> { self.definition.collections() }

  #[inline]
  pub fn read(
    &self,
    name: &str,
    condition: Condition,
    sorts: Vec<Sort>,
    range: Range
  ) -> Result<Iter, Error> {
    let collection = try!(
      self.get_collection(name)
      .ok_or(Error::not_found(format!("Can’t read for type '{}', because it does not exist in the schema.", name)))
    );
    let driver: &Driver = collection.driver().and_then(|config| self.drivers.get(config)).map_or(&self.memory, Deref::deref);
    driver.read(name, condition, sorts, range)
  }

  #[inline]
  pub fn read_one(
    &self,
    name: &str,
    condition: Condition
  ) -> Result<Value, Error> {
    let collection = try!(
      self.get_collection(name)
      .ok_or(Error::not_found(format!("Can’t read for type '{}', because it does not exist in the schema.", name)))
    );
    let driver: &Driver = collection.driver().and_then(|config| self.drivers.get(config)).map_or(&self.memory, Deref::deref);
    driver.read_one(name, condition)
  }
}
