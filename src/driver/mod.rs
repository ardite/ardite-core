//! This module contains the common driver code. Specific implementations for
//! different drivers exist elsewhere.

mod driver;
pub mod memory;
#[cfg(feature = "driver_mongodb")]
pub mod mongodb;

pub use driver::driver::Driver;
