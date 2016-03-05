//! This module contains all the operations related to the Ardite Schema
//! Definition, a format which is used to imperitevly define the data interface
//! used with Ardite services. 

pub mod schema;
pub mod serde;

use definition::schema::Schema;

/// The definition object which contains all necessary information to
/// understand an Ardite Schema Definition.
#[derive(PartialEq, Debug)]
pub struct Definition {
  /// The shape of the defined data.
  pub data: Schema
}
