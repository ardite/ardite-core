//! This module focuses on handling errors generated when using Ardite in a
//! graceful manner.

#[cfg(test)]
use regex::Regex;
use std::io::Error as IOError;
use std::error::Error as ErrorTrait;
use serde_json::error::Error as JSONError;
use serde_yaml::error::Error as YAMLError;

/// The code of an error. Designed to easily map to [HTTP status codes][1].
///
/// [1]: http://www.restapitutorial.com/httpstatuscodes.html
#[derive(PartialEq, Clone, Debug)]
pub enum ErrorCode {
  /// A bad syntax was used. Maps to 400.
  BadRequest = 400,
  /// Permissions do not allow this to happen. Maps to 403.
  Forbidden = 403,
  /// Resource was not found. Maps to 404.
  NotFound = 404,
  /// The requested resource is not acceptable. Maps to 406.
  NotAcceptable = 406,
  /// Present data made the request fail. Maps to 409.
  Conflict = 409,
  /// There was an invalid range. Maps to 416.
  BadRange = 416,
  /// Something bad happened inside a driver. Maps to 500.
  Internal = 500,
  /// The feature has not been implemented. Maps to 501.
  NotImplemented = 501
}

/// Any error generated by Ardite or it‘s drivers should be output using this
/// type. This allows for a comprehensive display of the error when a service
/// reports it to the user.
///
/// Information included with the error includes an `ErrorCode` (which maps to
/// an HTTP status code), a message, and an optional hint telling the user how
/// to fix the error.
///
/// Typically hints should be included for what would be considered the `4xx`
/// (in HTTP language) class of error codes.
///
/// # Tips For Writing Good Hint Messages
/// - Write in the second person (“You should…”).
/// - Always recommend a solution (“You should try…”, not “You must do…”).
/// - Be as specific as possible, if you have line numbers give them. If you
///   have file paths provide them.
/// - If it is a common error which generally confuses developers, provide a
///   link to a page which better explains the error and specific steps to
///   fix it.
#[derive(PartialEq, Debug)]
pub struct Error {
  /// A specific error code which describes the error.
  code: ErrorCode,
  /// A message providing more detail beyond the error code.
  message: String,
  /// A hint to the user on what to do next to try and avoid the error
  /// happening again. This is optional.
  hint: Option<String>
}

impl Error {
  /// Easily create a new error.
  pub fn new<S>(code: ErrorCode, message: S, hint: Option<S>) -> Self where S: Into<String> {
    Error {
      code: code,
      message: message.into(),
      hint: hint.map(|string| string.into())
    }
  }

  /// Get the code for the error.
  pub fn code(&self) -> ErrorCode {
    self.code.to_owned()
  }

  /// Get the message for the error.
  pub fn message(&self) -> String {
    self.message.to_owned()
  }

  /// Get the hint—for the error (see what I did there?).
  pub fn hint(&self) -> Option<String> {
    self.hint.to_owned()
  }

  /// Convenience function for saying a thing failed validation using
  /// `ErrorCode::BadRequest`.
  ///
  /// # Example
  /// ```rust
  /// use ardite::error::{Error, ErrorCode};
  ///
  /// let error = Error::validation("Failed validation.", "Try fixing your syntax!");
  ///
  /// assert_eq!(error, Error::new(ErrorCode::BadRequest, "Failed validation.", Some("Try fixing your syntax!")));
  /// ```
  pub fn validation<S1, S2>(message: S1, hint: S2) -> Self where S1: Into<String>, S2: Into<String> {
    Error {
      code: ErrorCode::BadRequest,
      message: message.into(),
      hint: Some(hint.into())
    }
  }

  /// Convenience function for saying there was an internal error using
  /// `ErrorCode::Internal`.
  ///
  /// # Example
  /// ```rust
  /// use ardite::error::{Error, ErrorCode};
  ///
  /// let error = Error::internal("Something blew up.");
  ///
  /// assert_eq!(error, Error::new(ErrorCode::Internal, "Something blew up.", None));
  /// ```
  pub fn internal<S>(message: S) -> Self where S: Into<String> {
    Error {
      code: ErrorCode::Internal,
      message: message.into(),
      hint: None
    }
  }

  /// Convenience function for creating an unimplemented error with a plain
  /// message describing what is unimplemented using
  /// `ErrorCode::NotImplemented`.
  ///
  /// # Example
  /// ```rust
  /// use ardite::error::{Error, ErrorCode};
  ///
  /// let error = Error::unimplemented("Cache invalidation is hard.");
  ///
  /// assert_eq!(error, Error::new(ErrorCode::NotImplemented, "Cache invalidation is hard.", None));
  /// ```
  pub fn unimplemented<S>(message: S) -> Self where S: Into<String> {
    Error {
      code: ErrorCode::NotImplemented,
      message: message.into(),
      hint: None
    }
  }

  /// Special assertion for error messages. Takes a regular expression string
  /// argument which will automatically be constructed into a regualr
  /// expression. Only available in testing environments. Panics if the regular
  /// expression doesn’t match the error message string.
  #[cfg(test)]
  pub fn assert_message(&self, regex_str: &str) {
    if !Regex::new(regex_str).unwrap().is_match(&self.message) {
      panic!("Error message \"{}\" does not match regex /{}/", self.message, regex_str);
    }
  }
}

impl From<IOError> for Error {
  fn from(error: IOError) -> Self {
    Error {
      code: ErrorCode::Internal,
      message: error.description().to_owned(),
      hint: None
    }
  }
}

impl From<JSONError> for Error {
  fn from(error: JSONError) -> Self {
    match error {
      JSONError::Syntax(_, line, column) => {
        Error {
          code: ErrorCode::BadRequest,
          message: "Syntax error.".to_owned(),
          hint: Some(format!("Max sure your JSON syntax is correct around line {} column {}.", line, column))
        }
      },
      _ => {
        Error {
          code: ErrorCode::Internal,
          message: error.description().to_owned(),
          hint: None
        }
      }
    }
  }
}

impl From<YAMLError> for Error {
  fn from(error: YAMLError) -> Self {
    match error {
      YAMLError::Custom(ref message) => {
        Error {
          code: ErrorCode::BadRequest,
          message: message.to_owned(),
          hint: Some("Make sure your YAML syntax is correct.".to_owned())
        }
      },
      _ => {
        Error {
          code: ErrorCode::Internal,
          message: error.description().to_owned(),
          hint: None
        }
      }
    }
  }
}
