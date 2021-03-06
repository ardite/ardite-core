#![allow(unknown_lints)]
// TODO: #![deny(missing_docs)]

#[macro_use(lazy_static)]
extern crate lazy_static;
#[macro_use(linear_map)]
extern crate linear_map;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate url;

#[cfg(feature = "driver_mongodb")]
#[macro_use(bson, doc)]
extern crate bson;
#[cfg(feature = "driver_mongodb")]
extern crate mongodb;

#[macro_use]
mod macros;

pub mod driver;
pub mod error;
pub mod query;
pub mod schema;
pub mod value;

pub use driver::Driver;
pub use error::Error;
pub use schema::{Definition, Type, DriverConfig, Schema};
pub use value::{Key, Pointer, Object, Array, Value};
