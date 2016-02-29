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
  Object(LinearMap<Key, Query>)
}

impl From<Pointer> for Query {
  fn from(pointer: Pointer) -> Self {
    // Reverse loop through the pointer to construct the query.
    pointer.iter().rev().fold(Query::Value, |acc, key| {
      let mut map = LinearMap::new();
      map.insert(key.to_owned(), acc);
      Query::Object(map)
    })
  }
}

#[cfg(test)]
mod tests {
  use query::Query;

  #[test]
  fn test_from_pointer() {
    let hello = || String::from("hello");
    let good = || String::from("good");
    let world = || String::from("world");
    assert_eq!(Query::from(vec![hello(), good(), world()]), Query::Object(linear_map!{
      String::from("hello") => Query::Object(linear_map!{
        String::from("good") => Query::Object(linear_map!{
          String::from("world") => Query::Value
        })
      })
    }));
    assert_eq!(Query::from(vec![good()]), Query::Object(linear_map!{
      String::from("good") => Query::Value
    }));
    assert_eq!(Query::from(vec![]), Query::Value);
  }
}
