//! The driver trait which all drivers will implement.

use error::Error;

pub trait Driver {
  /// Connects to a driver and returns a driver instance. After calling this
  /// the driver is ready to roll!
  fn connect(uri: &str) -> Result<Self, Error> where Self: Sized;
}
