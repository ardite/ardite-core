//! Provides a default and reference driver implementation which stores all of
//! its information in memory.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Mutex;

use itertools::Itertools;
use url::Url;

use driver::{Driver, Iter};
use error::Error;
use query::{Range, Sort, Condition};
use value::Object;

/// The default driver to be used by a service when no other driver is
/// specified. This driver, unlike the others, stores all of its data in
/// memory. The best usecase for this driver is in testing and development.
///
/// A warning, whenever data from the `Memory` driver is accessed, it prevents
/// any other thread from using the data stored in memory. This basically means
/// only one request can be made to this driver at once. Because of this, it
/// would be a very bad idea to use this driver in production.
pub struct Memory {
  /// The actual internal `HashMap` store. Wrapped in a `Mutex` so that we can
  /// mutate the value *without* requiring a mutable reference to `Memory`.
  store: Mutex<HashMap<String, Vec<Object>>>
}

impl Memory {
  /// Creates a new instance of the memory driver.
  pub fn new() -> Self {
    Memory {
      store: Mutex::new(HashMap::new())
    }
  }

  /// Look ma, no mutable! Yes, you read the type signature correctly. You get
  /// a mutable reference to the collection *without* requiring a mutable
  /// reference to `self`. It is a requirement of the `Driver` trait that we
  /// never use mutable references to `self` because `Driver`s will often be
  /// shared across multiple different threads.
  pub fn append_to_collection(&self, name: &str, objects: &mut Vec<Object>) {
    let mut store = self.store.lock().unwrap();

    if !store.contains_key(name) {
      store.insert(name.to_owned(), Vec::new());
    }

    // We can safely unwrap here because we guarantee the collection exists in
    // the if statement above.
    store.get_mut(name).unwrap().append(objects);
  }
}

impl Driver for Memory {
  /// Connecting the memory driver in this way will *always* be an error. This
  /// is because the memory driver doesn’t depend on any `url` (as its data
  /// *is* stored locally in memory). Instead use the `new` function provided
  /// on the struct.
  ///
  /// This method may eventually not return an error if a valid use case is
  /// shown for the memory driver in production, so do not depend on this
  /// functionality.
  fn connect(_: &Url) -> Result<Self, Error> {
    Err(Error::invalid("You can’t connect to memory silly.", "Use the `new` method instead for the memory driver."))
  }

  // TODO: Test that condition and sort is applied before range to the results.
  fn read(
    &self,
    name: &str,
    cond: Condition,
    sorts: Vec<Sort>,
    range: Range
  ) -> Result<Iter, Error> {
    if let Some(objects) = self.store.lock().unwrap().get(name) {
      Ok(Iter::new(
        objects
        .into_iter()
        .filter(|object| cond.is_object_true(object))
        .slice(range)
        .cloned()
        .sorted_by(|a, b| {
          sorts
          .iter()
          .fold(None, |ord, sort| ord.or_else(|| sort.partial_cmp(a, b)))
          .unwrap_or(Ordering::Equal)
        })
        .into_iter()
      ))
    } else {
      Ok(Iter::none())
    }
  }
}
