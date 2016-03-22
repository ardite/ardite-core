//! This module contains all the operations related to the Ardite Schema
//! Definition, a format which is used to imperitevly define the data interface
//! used with Ardite services.

mod definition;
mod schema;
mod de;

pub use schema::schema::*;
pub use schema::definition::{Definition, Type, DriverConfig};
