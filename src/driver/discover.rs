use driver::Driver;
use error::Error;
use schema;

/// Takes a driver config value and finds and connects the associated driver.
/// Errors if not driver could be found for the given config.
// TODO: @svmnotn this is your jam!
pub fn discover_driver(config: &schema::Driver) -> Result<Box<Driver>, Error> {
  match config.url().scheme.as_str() {
    "mongodb" => connect_mongodb_driver(config),
    _ => Err(Error::not_found(format!("Driver for URL '{}' not found.", config.url())))
  }
}

#[cfg(feature = "driver_mongodb")]
fn connect_mongodb_driver(_: &schema::Driver) -> Result<Box<Driver>, Error> {
  use driver::mongodb::MongoDB;
  MongoDB::connect(config.url()).map(Box::new)
}

#[cfg(not(feature = "driver_mongodb"))]
fn connect_mongodb_driver(_: &schema::Driver) -> Result<Box<Driver>, Error> {
  Err(Error::invalid("Can not use MongoDB driver.", "Try compiling Ardite with the `driver_mongodb` feature enabled."))
}
