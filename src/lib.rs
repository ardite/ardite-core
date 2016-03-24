//! The core library for all Ardite services. It provides the connective
//! tissue between the user defined schema—a driver—and the service which the
//! user wants to use.
//!
//! These docs are the documentation for the core rust API. If you are
//! interested in *using* Ardite this is probably not the documentation for
//! you. This documentation is for people developing *for* Ardite. As in the
//! people developing services and/or drivers. If you are not developing a
//! service or a driver, go to the Ardite [`README`][1] to find appropriate
//! documentation for your usage.
//!
//! [1]: https://github.com/ardite/ardite-core

#![allow(unknown_lints)]
// TODO: #![deny(missing_docs)]

extern crate inflector;
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

pub mod case;
pub mod driver;
pub mod error;
pub mod query;
pub mod schema;
    mod service;
pub mod value;

pub use error::Error;
pub use service::*;
pub use value::Value;
