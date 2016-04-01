//! Defines complex queries over Ardite driver data structures.
// TODO: This needs *lots* of review and expirementation.

use std::cmp;
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
  limit: Option<usize>,
  /// How many items should be skipped from the full set in this range.
  offset: Option<usize>
}

impl Range {
  /// Creates a new range from a limit and an offset.
  pub fn new(offset: Option<usize>, limit: Option<usize>) -> Self {
    Range {
      limit: limit,
      offset: offset
    }
  }

  /// Get the limit of the range.
  pub fn limit(&self) -> Option<usize> {
    self.limit
  }

  /// Get the offset of the range.
  pub fn offset(&self) -> Option<usize> {
    self.offset
  }

  /// Gets a slice (subset) of the `Vec` argument returning another `Vec` with
  /// references to the items.
  ///
  /// The returned `Vec` may have less items them limit and may even have no
  /// items at all depending on whether or not the range exists in the `Vec`
  /// parameter.
  // TODO: Consider having `Range` implement `Iterator`.
  pub fn view<'a, 'b, V>(&'a self, vec: &'b Vec<V>) -> Vec<&'b V> {
    let offset = cmp::min(self.offset().unwrap_or(0), vec.len());
    let max_limit = vec.len() - offset;
    // We are maxing out our limit, because we will be looping later for that
    // length.
    let limit = cmp::min(self.limit().unwrap_or(max_limit), max_limit);
    let mut new_vec = Vec::new();
    let mut progress: usize = 0;

    while progress != limit {
      if let Some(item) = vec.get(offset + progress) {
        new_vec.push(item);
      }
      progress += 1;
    }

    new_vec
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
  use super::*;

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

  #[test]
  fn test_range_view() {
    let vec = vec!["a", "b", "c", "d", "e"];
    assert_eq!(Range::new(None, None).view(&vec), vec![&"a", &"b", &"c", &"d", &"e"]);
    assert_eq!(Range::new(Some(0), None).view(&vec), vec![&"a", &"b", &"c", &"d", &"e"]);
    assert_eq!(Range::new(Some(1), None).view(&vec), vec![&"b", &"c", &"d", &"e"]);
    assert_eq!(Range::new(Some(4), None).view(&vec), vec![&"e"]);
    assert_eq!(Range::new(Some(20), None).view(&vec), vec![] as Vec<&&str>);
    assert_eq!(Range::new(None, Some(2)).view(&vec), vec![&"a", &"b"]);
    assert_eq!(Range::new(None, Some(0)).view(&vec), vec![] as Vec<&&str>);
    assert_eq!(Range::new(None, Some(5)).view(&vec), vec![&"a", &"b", &"c", &"d", &"e"]);
    assert_eq!(Range::new(None, Some(20)).view(&vec), vec![&"a", &"b", &"c", &"d", &"e"]);
    assert_eq!(Range::new(Some(0), Some(0)).view(&vec), vec![] as Vec<&&str>);
    assert_eq!(Range::new(Some(1), Some(1)).view(&vec), vec![&"b"]);
    assert_eq!(Range::new(Some(4), Some(1)).view(&vec), vec![&"e"]);
    assert_eq!(Range::new(Some(3), Some(2)).view(&vec), vec![&"d", &"e"]);
    assert_eq!(Range::new(Some(2), Some(2)).view(&vec), vec![&"c", &"d"]);
    assert_eq!(Range::new(Some(1), Some(3)).view(&vec), vec![&"b", &"c", &"d"]);
    assert_eq!(Range::new(Some(3), Some(50)).view(&vec), vec![&"d", &"e"]);
  }
}
