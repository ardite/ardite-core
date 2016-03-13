#![allow(unknown_lints)]

#[macro_use(lazy_static)]
extern crate lazy_static;
extern crate linear_map;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

#[cfg(feature = "driver_mongodb")]
#[macro_use(bson, doc)]
extern crate bson;
#[cfg(feature = "driver_mongodb")]
extern crate mongodb;

#[cfg(test)]
#[macro_use]
mod tests;

pub mod driver;
pub mod error;
pub mod patch;
pub mod query;
pub mod schema;
pub mod value;
