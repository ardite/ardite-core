//! Defines complex queries over Ardite driver data structures.

use std::convert::From;
use linear_map::LinearMap;
use value::{Key, Pointer, Value};

/// Specifies a positive numeric range of data.
pub struct Range(Bound, Bound);

/// Taken from unstable [Rust][1]. When stable we should use it.
///
/// [1]: http://doc.rust-lang.org/std/collections/enum.Bound.html
pub enum Bound {
  Included(u64),
  Excluded(u64),
  Unbounded
}

/// A condition which will resolve to a boolean value after comparing a certain
/// value with a set rule.
// TODO: Add standard conditions.
pub enum Condition {
  /// The condition always passes.
  True,
  /// The condition always fails.
  False,
  /// Inverts a condition.
  Not(Box<Condition>),
  /// Composes many conditions. They all must be true for the condition to be
  /// true.
  And(Vec<Condition>),
  /// Composes many conditions. Only one must be true for the condition to be
  /// true.
  Or(Vec<Condition>),
  /// If the compared value is exactly equal to this one, the condition passes.
  Equal(Value)
}

/// Specifies a complex driver query. The query is structured like a tree
/// except each node is unaware of its name (or if it even has a name). It
/// cannot be expected that a `Query` tree will map 1 to 1 with a `Value` tree.
#[derive(PartialEq, Debug)]
pub enum Query {
  /// Queries a single value.
  All,
  /// Queries some partial properties of an object.
  Keys(LinearMap<Key, Query>)
}

impl From<Pointer> for Query {
  fn from(pointer: Pointer) -> Self {
    // Reverse loop through the pointer to construct the query.
    pointer.iter().rev().fold(Query::All, |acc, key| {
      let mut map = LinearMap::new();
      map.insert(key.to_owned(), acc);
      Query::Keys(map)
    })
  }
}

#[cfg(test)]
mod tests {
  use query::Query;

  #[test]
  fn test_from_pointer() {
    assert_eq!(Query::from(point!["hello", "good", "world"]), Query::Keys(linear_map! {
      S!("hello") => Query::Keys(linear_map! {
        S!("good") => Query::Keys(linear_map! {
          S!("world") => Query::All
        })
      })
    }));
    assert_eq!(Query::from(point!["good"]), Query::Keys(linear_map! {
      S!("good") => Query::All
    }));
    assert_eq!(Query::from(point![]), Query::All);
  }
}
