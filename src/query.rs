//! Defines complex queries over Ardite driver data structures.
// TODO: This needs *lots* of review and expirementation.

use std::convert::From;
use linear_map::LinearMap;

use value::{Key, Pointer, Value};

/// A condition which will resolve to a boolean value after comparing a certain
/// value with a set rule.
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
  /// Partial conditions on some keys of an object.
  Keys(LinearMap<Key, Condition>),
  /// If the compared value is exactly equal to this one, the condition passes.
  Equal(Value)
}

impl Default for Condition {
  fn default() -> Self {
    Condition::True
  }
}

/// Specifies the order in which a property of a value should be ordered.
pub struct SortRule {
  /// The exacty property to order by.
  property: Pointer,
  /// The direction to order the property in.
  direction: SortDirection
}

impl SortRule {
  /// Create a new sorting rule from the property pointer and a boolean
  /// specifying if we are ascending or descending.
  pub fn new(property: Pointer, ascending: bool) -> Self {
    SortRule {
      property: property,
      direction: if ascending { SortDirection::Ascending } else { SortDirection::Descending }
    }
  }

  /// Get the property the struct is sorting against.
  pub fn property(&self) -> &Pointer {
    &self.property
  }

  /// Is the struct sorting the property in ascending order?
  pub fn is_ascending(&self) -> bool {
    if let SortDirection::Ascending = self.direction {
      true
    } else {
      false
    }
  }

  /// Is the struct sorting the property in descending order?
  pub fn is_descending(&self) -> bool {
    if let SortDirection::Descending = self.direction {
      true
    } else {
      false
    }
  }
}

/// The direction in which an order occurs.
enum SortDirection {
  Ascending,
  Descending
}

/// Specifies a positive integer range in a traditional SQL format.
pub struct Range {
  /// How many items should be included in this range.
  limit: Option<u64>,
  /// How many items should be skipped from the full set in this range.
  skip: Option<u64>
}

impl Range {
  /// Creates a new range from a limit and a skip.
  pub fn new(skip: Option<u64>, limit: Option<u64>) -> Self {
    Range {
      limit: limit,
      skip: skip
    }
  }

  /// Get the limit of the range.
  pub fn limit(&self) -> Option<u64> {
    self.limit
  }

  /// Get how many items the range skips over.
  pub fn skip(&self) -> Option<u64> {
    self.skip
  }
}

impl Default for Range {
  fn default() -> Self {
    Range::new(None, None)
  }
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

impl Default for Query {
  fn default() -> Self {
    Query::All
  }
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
      str!("hello") => Query::Keys(linear_map! {
        str!("good") => Query::Keys(linear_map! {
          str!("world") => Query::All
        })
      })
    }));
    assert_eq!(Query::from(point!["good"]), Query::Keys(linear_map! {
      str!("good") => Query::All
    }));
    assert_eq!(Query::from(point![]), Query::All);
  }
}
