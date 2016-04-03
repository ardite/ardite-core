extern crate ardite;
#[macro_use]
extern crate ardite_driver_tests as tests;
extern crate linear_map;

use ardite::Value;
use ardite::driver::{Driver, Memory};

test_driver!(Tests);

pub struct Tests;

impl tests::Tests for Tests {
  fn test_driver(name: &str, mut values: Vec<Value>) -> Box<Driver> {
    let memory = Memory::new();
    memory.append_to_type(name, &mut values);
    Box::new(memory)
  }
}
