use std::collections::BTreeMap;
use std::path::PathBuf;

use error::Error;
use schema::{Definition, DriverConfig};
use driver::{Driver, MemoryDriver};

pub struct Service {
  definition: Definition,
  memory_driver: MemoryDriver,
  drivers: BTreeMap<DriverConfig, Box<Driver>>
}

impl Service {
  pub fn new(definition: Definition) -> Self {
    Service {
      definition: definition,
      memory_driver: MemoryDriver::new(),
      drivers: BTreeMap::new()
    }
  }

  pub fn from_file(path: PathBuf) -> Result<Self, Error> {
    Service::new(try!(Definition::from_file(path)))
  }

  /// Iterates through the `DriverConfig`s in the definition, connecting them,
  /// and storing them internally. After running this method, all drivers
  /// outside of memory will be connected.
  pub fn connect_drivers(&mut self) -> Result<(), Error> {
    unimplemented!();
  }
}
