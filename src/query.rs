//! Defines complex queries over Ardite driver data structures.

use std::convert::From;
use value::*;

/// Specifies a complex driver query. The query is structured like a tree
/// except each node is unaware of its name (or if it even has a name). It
/// cannot be expected that a `Query` tree will map 1 to 1 with a `Value` tree.
#[derive(PartialEq, Debug)]
pub enum Query {
  /// No operation query. Useful for `Option` type things.
  Noop,
  /// Basic query of a value with some specified children.
  Object {
    /// Name of 
    name: Key,
    /// Child queries.
    children: Vec<Query>
  },
  /// Query all values of a single record. If an object it returns *all* of the
  /// properties. Works like a star select in SQL. Some properties may be
  /// hidden and therefore not included in the return value. This is up to the
  /// disgression of the driver.
  Property {
    /// The name of the value to return.
    name: Key
  },
  /// Directly query a collection. When thinking of the `Query` type as a tree
  /// which maps to a `Value` object, this query “skips” a level. For example,
  /// another query might have
  /// `Query::Item(users) -> Query::Item(1) -> Query::Item(name)` which maps to
  /// the expect JSON pointer `/users/1/name`. Whereas with a collection, the
  /// query may look like `Query::Collection(users) -> Query::Item(name)`. Note
  /// that there was no “middle” query item for the record. 
  Collection {
    /// The name of the collection we will be querying.
    name: Key,
    /// The range of records to query.
    range: Range,
    /// A set of conditions which are joined by “and” which specifies the
    /// values to be filtered out be the query.
    filter: Condition,
    /// Child queries.
    children: Vec<Query>
  }
}

impl From<Pointer> for Query {
  fn from(mut pointer: Pointer) -> Query {
    if pointer.len() == 0 {
      // Exit early if length is 0 to avoid errors later.
      Query::Noop
    } else {
      // Take out the last key and save it. We will use this to start our fold.
      // Unwrapping should be safe here because we exit if the pointer has a
      // length of 0 earlier.
      let last_key = pointer.pop().unwrap();
      // Reverse loop through the pointer to construct the query by setting the
      // only child to the rightmost query object. 
      pointer.iter().rev().fold(Query::Property {
        name: last_key
      }, |acc, key| Query::Object {
        name: key.to_owned(),
        children: vec![acc]
      })
    }
  }
}

/// Specifies a positive numeric range of data.
#[derive(PartialEq, Default, Debug)]
pub struct Range {
  /// The inclusive lower bound of the range. If `None`, the range is unbounded
  /// on the bottom and thus goes to 0.
  from: Option<u64>,
  /// The exclusive upper bound of the range. If `None`, the range is unbounded
  /// on the top and thus goes to infinity.
  to: Option<u64>
}

/// A condition which will resolve to a boolean value after comparing a certain
/// value with a set rule.
// TODO: Add more conditions.
#[derive(PartialEq, Debug)]
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

#[cfg(test)]
mod tests {
  use query::Query;
  
  #[test]
  fn from_pointer() {
    let hello = || String::from("hello");
    let good = || String::from("good");
    let world = || String::from("world");    
    
    assert_eq!(Query::from(vec![hello(), good(), world()]), Query::Object {
      name: hello(),
      children: vec![Query::Object {
        name: good(),
        children: vec![Query::Property {
          name: world()
        }]
      }]
    });
    
    assert_eq!(Query::from(vec![good()]), Query::Property {
      name: good()
    });
    
    assert_eq!(Query::from(vec![]), Query::Noop);
  }
}
