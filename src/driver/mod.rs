//! This module contains the common driver code. Specific implementations for
//! different drivers exist elsewhere.

mod driver;
mod discover;
mod memory;
#[cfg(feature = "driver_mongodb")]
pub mod mongodb;

pub use driver::driver::Driver;
pub use driver::discover::discover_driver;
pub use driver::memory::MemoryDriver;
