//! This module focuses on handling errors generated when using Ardite in a
//! graceful manner.

#[cfg(test)]
use regex::Regex;
use std::io::Error as IOError;
use std::error::Error as ErrorTrait;
use serde_json::error::Error as JSONError;

/// The code of an error. Designed to easily map to [HTTP status codes][1].
///
/// [1]: http://www.restapitutorial.com/httpstatuscodes.html
#[derive(PartialEq, Debug)]
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
// TODO: Remove `pub` when `impl`ing the standard `Error` trait.
#[derive(PartialEq, Debug)]
pub struct Error {
  /// A specific error code which describes the error.
  pub code: ErrorCode,
  /// A message providing more detail beyond the error code.
  pub message: String,
  /// A hint to the user on what to do next to try and avoid the error
  /// happening again. This is optional.
  pub hint: Option<String>
}

impl Error {
  /// Easily create a new error.
  pub fn new<S>(code: ErrorCode, message: S) -> Self where S: Into<String> {
    Error {
      code: code,
      message: message.into(),
      hint: None
    }
  }

  /// Convenience function for saying a thing failed validation.
  pub fn validation<S1, S2>(message: S1, hint: S2) -> Self where S1: Into<String>, S2: Into<String> {
    Error {
      code: ErrorCode::BadRequest,
      message: message.into(),
      hint: Some(hint.into())
    }
  }
  
  /// Convenience function for saying there was an internal error.
  pub fn internal<S>(message: S) -> Self where S: Into<String> {
    Error {
      code: ErrorCode::Internal,
      message: message.into(),
      hint: None
    }
  }

  /// Convenience function for creating an unimplemented error with a plain
  /// message describing what is unimplemented.
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
      message: error.description().to_string(),
      hint: None
    }
  }
}

impl From<JSONError> for Error {
  fn from(error: JSONError) -> Self {
    match &error {
      &JSONError::Syntax(_, line, column) => {
        Error {
          code: ErrorCode::BadRequest,
          message: error.description().to_string(),
          hint: Some(format!("Fix your JSON syntax around line {} column {}.", line, column))
        }
      },
      _ => {
        Error {
          code: ErrorCode::Internal,
          message: error.description().to_string(),
          hint: None
        }
      }
    }
  }
}
