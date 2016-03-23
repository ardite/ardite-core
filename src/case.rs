use std::str::FromStr;

use inflector;

use error::Error;

/// Enum for converting one case into another. Uses the [`Inflector`][1] crate.
///
/// [1]: https://crates.io/crates/Inflector
pub enum Case {
  /// A case identity. When converting it makes sure to always return the
  /// original value.
  Same,
  /// Converts to camelCase, like JavaScript.
  Camel,
  /// Converts to kebab-case, like CSS.
  Kebab,
  /// Converts to snake_case, like Rust.
  Snake
}

// Release the case variants!
pub use self::Case::*;

impl Case {
  /// Converts a string into the case of `self`. Note that because the cases
  /// are unit variants, you will use `.` to call the methods and not `::`.
  ///
  /// # Example
  /// ```rust
  /// use ardite::case::{Same, Camel, Kebab, Snake};
  ///
  /// assert_eq!(Same.to_case("Hello world".to_owned()), "Hello world".to_owned());
  /// assert_eq!(Camel.to_case("Hello world".to_owned()), "helloWorld".to_owned());
  /// assert_eq!(Kebab.to_case("Hello world".to_owned()), "hello-world".to_owned());
  /// assert_eq!(Snake.to_case("Hello world".to_owned()), "hello_world".to_owned());
  /// ```
  pub fn to_case(&self, string: String) -> String {
    match *self {
      Same => string,
      Camel => inflector::cases::camelcase::to_camel_case(string),
      Kebab => inflector::cases::kebabcase::to_kebab_case(string),
      Snake => inflector::cases::snakecase::to_snake_case(string)
    }
  }

  /// Detects if a word is in the case of `self`. Note that because the cases
  /// are unit variants, you will use `.` to call the methods and not `::`.
  ///
  /// Note that `Case::Same` always returns `true`.
  ///
  /// # Example
  /// ```rust
  /// use ardite::case::{Same, Camel, Kebab, Snake};
  ///
  /// assert!(Same.is_case("Hello world"));
  /// assert!(Camel.is_case("helloWorld"));
  /// assert!(!Camel.is_case("hello_world"));
  /// assert!(Kebab.is_case("hello-world"));
  /// assert!(!Kebab.is_case("helloWorld"));
  /// assert!(Snake.is_case("hello_world"));
  /// assert!(!Snake.is_case("hello-world"));
  /// ```
  pub fn is_case(&self, string: &str) -> bool {
    match *self {
      Same => true,
      Camel => inflector::cases::camelcase::is_camel_case(string.to_owned()),
      Kebab => inflector::cases::kebabcase::is_kebab_case(string.to_owned()),
      Snake => inflector::cases::snakecase::is_snake_case(string.to_owned())
    }
  }
}

impl FromStr for Case {
  type Err = Error;

  fn from_str(string: &str) -> Result<Self, Error> {
    match string {
      "same" => Ok(Same),
      "camel" => Ok(Camel),
      "kebab" => Ok(Kebab),
      "snake" => Ok(Snake),
      string @ _ => Err(Error::invalid(
        format!("String '{}' is not a valid case variant.", string),
        "Use a valid case variant like same, camel, kebab, or snake."
      ))
    }
  }
}
