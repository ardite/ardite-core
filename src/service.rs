use std::path::PathBuf;

use linear_map::LinearMap;

use error::Error;
use schema::{Definition, DriverConfig};
use driver::{Driver, MemoryDriver};

pub struct Service {
  definition: Definition,
  memory_driver: MemoryDriver,
  /// A map of driver configs to their respective drivers. We use a `LinearMap`
  /// because it does not require the `DriverConfig` to implement anything
  /// crazy like `Hash` or `Ord`. We also don’t ever suspect having a large
  /// number of drivers.
  drivers: LinearMap<DriverConfig, Box<Driver>>
}

impl Service {
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
  pub fn connect_drivers(&mut self) -> Result<(), Error> {
    unimplemented!();
  }
}
