use schema::Definition;
use driver::Driver;

pub struct Service {
  definition: Definition,
  driver: Box<Driver + 'static>
}
