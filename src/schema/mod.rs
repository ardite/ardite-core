//! This module contains all the operations related to the Ardite Schema
//! Definition, a format which is used to imperitevly define the data interface
//! used with Ardite services. 

pub mod definition;
pub mod schema;
pub mod serde;

pub use schema::definition::Definition;
pub use schema::schema::Schema;
pub use schema::serde::from_file;
