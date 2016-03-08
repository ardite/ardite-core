//! Contains the full definition of a data system which Ardite will use.

use schema::Schema;

/// The definition object which contains all necessary information to
/// understand an Ardite Schema Definition.
#[derive(PartialEq, Debug)]
pub struct Definition {
  /// The shape of the defined data.
  pub data: Schema
}
