use std::path::PathBuf;

use error::Error;
use schema;
use schema::{Schema, Type};
use driver::{discover_driver, Driver, Memory};
use query::{Condition, Sort, Range};
use value::{Value, Iter};

pub fn from_file(path: PathBuf) -> Result<Service, Error> {
  let schema = try!(schema::from_file(path));
  let service = try!(Service::from_schema(schema));
  Ok(service)
}

pub struct Service {
  pub schema: Schema,
  driver: Box<Driver>
}

impl Service {
  pub fn from_schema(schema: Schema) -> Result<Self, Error> {
    let driver = if let Some(driver) = schema.driver() {
      try!(discover_driver(driver))
    } else {
      Box::new(Memory::new()) as Box<Driver>
    };

    Ok(Service {
      schema: schema,
      driver: driver
    })
  }

  #[inline]
  fn get_type_or_else(&self, name: &str) -> Result<&Type, Error> {
    self.schema
    .get_type(name)
    .ok_or_else(|| Error::not_found(format!("Can’t use type '{}' because it does not exist in the schema.", name)))
  }

  #[inline]
  pub fn read(
    &self,
    name: &str,
    condition: Condition,
    sorts: Vec<Sort>,
    range: Range
  ) -> Result<Iter, Error> {
    let _ = try!(self.get_type_or_else(name));
    self.driver.read(name, condition, sorts, range)
  }

  #[inline]
  pub fn read_one(
    &self,
    name: &str,
    condition: Condition
  ) -> Result<Value, Error> {
    let _ = try!(self.get_type_or_else(name));
    self.driver.read_one(name, condition)
  }
}
