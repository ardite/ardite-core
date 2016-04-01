//! Defines complex queries over Ardite driver data structures.
// TODO: This needs *lots* of review and expirementation.

use std::convert::From;

use itertools::misc::GenericRange;
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
  /// Partial condition on a key of an object. To combine may keys use the
  /// `And` condition.
  Key(Key, Box<Condition>),
  /// If the compared value is exactly equal to this one, the condition passes.
  Equal(Value)
}

impl Condition {
  /// Evaluates if a value is true against the condition.
  pub fn is_true(&self, value: &Value) -> bool {
    use self::Condition::*;
    match *self {
      True => true,
      False => false,
      Not(ref cond) => cond.is_false(value),
      And(ref conds) => conds.iter().all(|cond| cond.is_true(value)),
      Or(ref conds) => conds.iter().any(|cond| cond.is_true(value)),
      Key(ref key, ref cond) => value.get(vec![key.to_owned()]).map_or(false, |value| cond.is_true(value)),
      Equal(ref other_value) => value == other_value
    }
  }

  /// Evaluates if a value is false against the condition.
  pub fn is_false(&self, value: &Value) -> bool {
    !self.is_true(value)
  }
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
}

impl Default for Range {
  fn default() -> Self {
    Range::new(None, None)
  }
}

impl GenericRange for Range {
  fn start(&self) -> Option<usize> {
    Some(self.offset().unwrap_or(0))
  }

  fn end(&self) -> Option<usize> {
    self.limit().map(|limit| self.offset().unwrap_or(0) + limit)
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
  fn test_condition_is_true() {
    use super::Condition::*;
    assert!(True.is_true(&value!([1, 2, 3])));
    assert!(!True.is_false(&value!("hello")));
    assert!(False.is_false(&value!(8)));
    assert!(Not(Box::new(False)).is_true(&value!(false)));
    assert!(And(vec![True, True, True]).is_true(&value!(2)));
    assert!(And(vec![True, False, True]).is_false(&value!(80)));
    assert!(Or(vec![True, False, True]).is_true(&value!("world")));
    assert!(Or(vec![False, False, False]).is_false(&value!({ "hello" => "world" })));
    assert!(Key(str!("key"), Box::new(True)).is_true(&value!({ "key" => "value" })));
    assert!(Key(str!("key"), Box::new(True)).is_false(&value!({ "yo" => "yo" })));
    assert!(Key(str!("key"), Box::new(True)).is_false(&value!(8)));
    assert!(And(vec![
      Key(str!("hello"), Box::new(True)),
      Key(str!("world"), Box::new(True))
    ]).is_true(&value!({
      "hello" => 2,
      "world" => 30
    })));
    assert!(And(vec![
      Key(str!("hello"), Box::new(True)),
      Key(str!("world"), Box::new(True))
    ]).is_false(&value!({
      "hello" => 2
    })));
    assert!(Equal(value!(42)).is_true(&value!(42)));
    assert!(Equal(value!(42)).is_false(&value!(41)));
    assert!(Equal(value!("hello")).is_false(&value!("world")));
    assert!(Equal(value!("hello")).is_true(&value!("hello")));
    assert!(Equal(value!({
      "hello" => "world",
      "goodbye" => { "moon" => true }
    })).is_true(&value!({
      "hello" => "world",
      "goodbye" => { "moon" => true }
    })));
    assert!(Equal(value!({
      "hello" => "world",
      "goodbye" => { "moon" => true }
    })).is_false(&value!({
      "hello" => "world",
      "goodbye" => { "moon" => false }
    })));
  }

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
