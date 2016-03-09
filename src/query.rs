//! Defines complex queries over Ardite driver data structures.

use std::convert::From;
use linear_map::LinearMap;
use value::*;

/// Specifies a complex driver query. The query is structured like a tree
/// except each node is unaware of its name (or if it even has a name). It
/// cannot be expected that a `Query` tree will map 1 to 1 with a `Value` tree.
#[derive(PartialEq, Debug)]
pub enum Query {
  /// Queries a single value.
  Value,
  /// Queries some partial properties of an object.
  Object(LinearMap<Selection, Query>)
}

// TODO: doc
#[derive(Eq, PartialEq, Debug)]
pub enum Selection {
  Key(Key)
}

impl From<Pointer> for Query {
  fn from(pointer: Pointer) -> Self {
    // Reverse loop through the pointer to construct the query.
    pointer.iter().rev().fold(Query::Value, |acc, key| {
      let mut map = LinearMap::new();
      map.insert(Selection::Key(key.to_owned()), acc);
      Query::Object(map)
    })
  }
}

#[cfg(test)]
mod tests {
  use query::{Query, Selection};

  #[test]
  fn test_from_pointer() {
    assert_eq!(Query::from(point!["hello", "good", "world"]), Query::Object(linear_map! {
      Selection::Key("hello".to_string()) => Query::Object(linear_map! {
        Selection::Key("good".to_string()) => Query::Object(linear_map! {
          Selection::Key("world".to_string()) => Query::Value
        })
      })
    }));
    assert_eq!(Query::from(point!["good"]), Query::Object(linear_map! {
      Selection::Key("good".to_string()) => Query::Value
    }));
    assert_eq!(Query::from(point![]), Query::Value);
  }
}
