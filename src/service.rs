use std::path::PathBuf;

use linear_map::LinearMap;

use error::Error;
use schema::{Definition, DriverConfig};
use driver::{discover_driver, Driver, MemoryDriver};

pub struct Service<'a> {
  definition: Definition,
  memory_driver: MemoryDriver,
  /// A map of driver configs to their respective drivers. We use a `LinearMap`
  /// because it does not require the `DriverConfig` to implement anything
  /// crazy like `Hash` or `Ord`. We also don’t ever suspect having a large
  /// number of drivers.
  drivers: LinearMap<&'a DriverConfig, Box<Driver>>
}

impl<'a> Service<'a> {
  pub fn new(definition: Definition) -> Self {
    Service {
      definition: definition,
      memory_driver: MemoryDriver::new(),
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
}
