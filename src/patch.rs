// TODO: Is a `patch` module really the best place to put this?

use document::{Property, Value};

/// Different database collection property updates.
pub enum Patch {
  /// Set a property to a new value.
  Set(Property, Value),

  /// Reset a property to its default value.
  Reset(Property),

  /// Remove a property from the database entirely.
  Remove(Property)
}
