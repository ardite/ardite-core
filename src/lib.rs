#[macro_use]
extern crate lazy_static;
extern crate linear_map;
extern crate regex;
extern crate serde;
extern crate serde_json;

#[cfg(test)]
#[macro_use]
mod tests;

pub mod driver;
pub mod error;
pub mod patch;
pub mod query;
pub mod schema;
pub mod value;
