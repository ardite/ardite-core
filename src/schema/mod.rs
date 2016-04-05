//! This module contains all the operations related to the Ardite Schema
//! Definition, a format which is used to imperitevly define the data interface
//! used with Ardite services.

mod de;
mod schema;

pub use self::de::from_file;
pub use self::schema::*;
