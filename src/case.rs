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
  Snake,
  /// Converts to ClassCase, like Java.
  Class,
  /// Converts to SCREAMING_CASE, like environment variables.
  Screaming,
  /// Converts to “Sentence case.”
  Sentence,
  /// Converts to “Title Case.”
  Title,
  /// Converts to “UPPER CASE”.
  Upper,
  /// Converts to “lower case”.
  Lower
}

// Release the case variants!
pub use self::Case::*;

impl Case {
  /// Converts a string into the case of `self`. Note that because the cases
  /// are unit variants, you will use `.` to call the methods and not `::`.
  ///
  /// # Example
  /// ```rust
  /// use ardite::case::*;
  ///
  /// assert_eq!(Same.to_case("Hello world".to_owned()), "Hello world".to_owned());
  /// assert_eq!(Camel.to_case("Hello world".to_owned()), "helloWorld".to_owned());
  /// assert_eq!(Kebab.to_case("Hello world".to_owned()), "hello-world".to_owned());
  /// assert_eq!(Snake.to_case("Hello world".to_owned()), "hello_world".to_owned());
  /// assert_eq!(Class.to_case("Hello world".to_owned()), "HelloWorld".to_owned());
  /// assert_eq!(Screaming.to_case("Hello world".to_owned()), "HELLO_WORLD".to_owned());
  /// assert_eq!(Sentence.to_case("hello_world".to_owned()), "Hello world".to_owned());
  /// assert_eq!(Title.to_case("hello_world".to_owned()), "Hello World".to_owned());
  /// assert_eq!(Upper.to_case("hello world".to_owned()), "HELLO WORLD".to_owned());
  /// assert_eq!(Upper.to_case("hello_world".to_owned()), "HELLO_WORLD".to_owned());
  /// assert_eq!(Lower.to_case("HELLO WORLD".to_owned()), "hello world".to_owned());
  /// assert_eq!(Lower.to_case("HELLO_WORLD".to_owned()), "hello_world".to_owned());
  /// ```
  pub fn to_case(&self, string: String) -> String {
    match *self {
      Same => string,
      Camel => inflector::cases::camelcase::to_camel_case(string),
      Kebab => inflector::cases::kebabcase::to_kebab_case(string),
      Snake => inflector::cases::snakecase::to_snake_case(string),
      Class => inflector::cases::classcase::to_class_case(string),
      Screaming => inflector::cases::screamingsnakecase::to_screaming_snake_case(string),
      Sentence => inflector::cases::sentencecase::to_sentence_case(string),
      Title => inflector::cases::titlecase::to_title_case(string),
      Upper => inflector::cases::uppercase::to_upper_case(string),
      Lower => inflector::cases::lowercase::to_lower_case(string)
    }
  }

  /// Detects if a word is in the case of `self`. Note that because the cases
  /// are unit variants, you will use `.` to call the methods and not `::`.
  ///
  /// Note that `Case::Same` always returns `true`.
  ///
  /// # Example
  /// ```rust
  /// use ardite::case::*;
  ///
  /// assert!(Same.is_case("Hello world"));
  /// assert!(!Camel.is_case("hello-world"));
  /// assert!(Camel.is_case("helloWorld"));
  /// assert!(!Kebab.is_case("hello_world"));
  /// assert!(Kebab.is_case("hello-world"));
  /// assert!(!Snake.is_case("HelloWorld"));
  /// assert!(Snake.is_case("hello_world"));
  /// assert!(!Class.is_case("HELLO_WORLD"));
  /// assert!(Class.is_case("HelloWorld"));
  /// assert!(!Screaming.is_case("helloWorld"));
  /// assert!(Screaming.is_case("HELLO_WORLD"));
  /// assert!(!Sentence.is_case("Hello World"));
  /// assert!(Sentence.is_case("Hello world"));
  /// assert!(!Title.is_case("Hello world"));
  /// assert!(Title.is_case("Hello World"));
  /// assert!(!Upper.is_case("Hello world"));
  /// assert!(Upper.is_case("HELLO WORLD"));
  /// assert!(!Lower.is_case("Hello world"));
  /// assert!(Lower.is_case("hello world"));
  /// ```
  pub fn is_case(&self, string: &str) -> bool {
    match *self {
      Same => true,
      Camel => inflector::cases::camelcase::is_camel_case(string.to_owned()),
      Kebab => inflector::cases::kebabcase::is_kebab_case(string.to_owned()),
      Snake => inflector::cases::snakecase::is_snake_case(string.to_owned()),
      Class => inflector::cases::classcase::is_class_case(string.to_owned()),
      Screaming => inflector::cases::screamingsnakecase::is_screaming_snake_case(string.to_owned()),
      Sentence => inflector::cases::sentencecase::is_sentence_case(string.to_owned()),
      Title => inflector::cases::titlecase::is_title_case(string.to_owned()),
      Upper => inflector::cases::uppercase::is_upper_case(string.to_owned()),
      Lower => inflector::cases::lowercase::is_lower_case(string.to_owned())
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
      "class" => Ok(Class),
      "screaming" => Ok(Screaming),
      "sentence" => Ok(Sentence),
      "title" => Ok(Title),
      "upper" => Ok(Upper),
      "lower" => Ok(Lower),
      string @ _ => Err(Error::invalid(
        format!("String '{}' is not a valid case variant.", string),
        "Use a valid case variant like same, camel, kebab, snake, class, screaming, sentence, title, upper, or lower."
      ))
    }
  }
}
