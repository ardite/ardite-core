//! Interfaces to update values in the driver.

use value::{Pointer, Value};

/// A single atomic patch on the driver.
pub enum Patch {
  /// Sets a value at the exact point in the driver.
  Set(Pointer, Value),
  /// Removes a value at the exact point in the driver.
  Remove(Pointer)
}
