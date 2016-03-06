// TODO: Replace all `String::from` with `.to_string`

#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(serde_macros)]

#[macro_use]
extern crate lazy_static;
extern crate linear_map;
extern crate regex;
extern crate serde;
extern crate serde_json;

#[cfg(test)]
#[macro_use]
mod tests;

pub mod definition;
pub mod driver;
pub mod error;
pub mod patch;
pub mod query;
pub mod value;
