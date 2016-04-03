//! Defines complex queries over Ardite driver data structures.
// TODO: This needs *lots* of review and expirementation.

use std::cmp::Ordering;

use itertools::misc::GenericRange;

use value::Value;

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
  Key(String, Box<Condition>),
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
      Key(ref key, ref cond) => value.get(key).map_or(false, |value| cond.is_true(value)),
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
pub struct Sort {
  /// The exacty property to order by.
  property: Vec<String>,
  /// The direction to order the property in.
  direction: Direction
}

impl Sort {
  /// Create a new sorting rule from the property path and a boolean
  /// specifying if we are ascending or descending.
  pub fn new(path: Vec<String>, ascending: bool) -> Self {
    Sort {
      property: path,
      direction: if ascending { Direction::Ascending } else { Direction::Descending }
    }
  }

  /// Get the property path the struct is sorting against.
  pub fn path(&self) -> Vec<&str> {
    self.property.iter().map(String::as_str).collect()
  }

  /// Is the struct sorting the property in ascending order?
  pub fn is_ascending(&self) -> bool {
    if let Direction::Ascending = self.direction {
      true
    } else {
      false
    }
  }

  /// Is the struct sorting the property in descending order?
  pub fn is_descending(&self) -> bool {
    if let Direction::Descending = self.direction {
      true
    } else {
      false
    }
  }

  pub fn partial_cmp(&self, a: &Value, b: &Value) -> Option<Ordering> {
    a.get_path(&self.path())
    .partial_cmp(&b.get_path(&self.path()))
    .map(|ord| if self.is_descending() { ord.reverse() } else { ord })
  }
}

/// The direction in which an order occurs.
pub enum Direction {
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

#[cfg(test)]
mod tests {
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
}
