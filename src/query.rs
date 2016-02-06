//! Defines complex queries over Ardite database data structures.

use std::collections::BTreeMap;
use value::*;

/// Specifies a complex database query. The query is structured like a tree
/// except each node is unaware of its name (or if it even has a name).
pub enum Query {
  /// The most basic query node.
  Node {
    /// Child queries. If `None` then the entire value is returned.
    children: Option<BTreeMap<Key, Query>>
  },
  /// Directly query a collection.
  Collection {
    /// The range of records to query.
    range: Range,
    /// A set of conditions which are joined by “and” which specifies the
    /// values to be filtered out be the query.
    filter: Vec<Condition>,
    /// Child queries. If `None` then the entire value is returned.
    children: Option<BTreeMap<Key, Query>>
  }
}

/// Specifies a positive numeric range of data.
#[derive(Default)]
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
pub enum Condition {
  /// The condition always passes.
  True,
  /// The condition always fails.
  False,
  /// If the compared value is exactly equal to this one, the condition passes.
  Equal(Value)
}
