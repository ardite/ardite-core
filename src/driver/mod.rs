//! This module contains the common driver code. Specific implementations for
//! different drivers exist elsewhere.

mod driver;
#[cfg(feature = "driver_mongodb")]
pub mod mongodb;

pub use driver::driver::Driver;
