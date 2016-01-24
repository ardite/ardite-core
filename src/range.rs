// TODO: Make this it's own module or use another implementation.

/// A numeric range. Inspired by Haskell‘s [ranged-sets][1].
///
/// [1]: http://hackage.haskell.org/package/Ranged-sets-0.3.0/docs/Data-Ranged-Ranges.html
pub struct Range(RangeBoundary, RangeBoundary);

impl Range {
  /// Creates a new range using an optional limt and offset. If offset is not
  /// defined, ir will be set to 0.
  fn new(optional_limit: Option<u32>, optional_offset: Option<u32>) -> Range {
    let offset = match optional_offset {
      Some(offset) => offset,
      None         => 0
    };

    Range(RangeBoundary::Above(offset), match optional_limit {
      Some(limit) => RangeBoundary::Below(offset + limit - 1),
      None        => RangeBoundary::BelowAll
    })
  }

  /// Extracts a limit value from the range.
  fn get_limit(&self) -> Option<u32> {
    match *self {
      Range(RangeBoundary::Above(from), RangeBoundary::Below(to)) => Some(to - from - 1),
      _ => None
    }
  }

  /// Extracts an offset value from the range.
  fn get_offset(&self) -> Option<u32> {
    match *self {
      Range(RangeBoundary::Above(offset), _) => Some(offset),
      _ => None
    }
  }
}

pub enum RangeBoundary {
  Above(u32),
  Below(u32),
  AboveAll,
  BelowAll
}
