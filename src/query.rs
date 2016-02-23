//! Defines complex queries over Ardite driver data structures.

use value::*;

/// Specifies a complex driver query. The query is structured like a tree
/// except each node is unaware of its name (or if it even has a name). It
/// cannot be expected that a `Query` tree will map 1 to 1 with a `Value` tree.
pub enum Query {
  /// Basic query of a value with some specified children.
  Item {
    /// Name of 
    name: String,
    /// Child queries.
    children: Vec<Query>
  },
  /// Query all values of a single record. If an object it returns *all* of the
  /// properties. Works like a star select in SQL. Some properties may be
  /// hidden and therefore not included in the return value. This is up to the
  /// disgression of the driver.
  Value {
    /// The name of the value to return.
    name: String
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
    name: String,
    /// The range of records to query.
    range: Range,
    /// A set of conditions which are joined by “and” which specifies the
    /// values to be filtered out be the query.
    filter: Condition,
    /// Child queries.
    children: Vec<Query>
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
