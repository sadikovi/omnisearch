//! List of various errors used here.

use std::convert;
use std::fmt;
use std::io;
use std::sync::mpsc;

use grep::regex::{Error as GrepRegexError};
use grep::searcher::SinkError;
use regex::{Error as RegexError};

/// General error struct.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Error {
  err: bool,
  msg: String
}

impl Error {
  /// Creates a new error with provided message.
  pub fn new(msg: String) -> Self {
    Self { err: true, msg: msg }
  }
}

macro_rules! err {
  ($fmt:expr) => (Err(errors::Error::new($fmt.to_owned())));
  ($fmt:expr, $($args:expr), *) => (Err(errors::Error::new(format!($fmt, $($args), *))));
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.msg)
  }
}

impl SinkError for Error {
  fn error_message<T: fmt::Display>(message: T) -> Self {
    Self::new(message.to_string())
  }
}

impl<T> convert::From<mpsc::SendError<T>> for Error {
  fn from(value: mpsc::SendError<T>) -> Self {
    Error::new(value.to_string())
  }
}

impl convert::From<io::Error> for Error {
  fn from(value: io::Error) -> Self {
    Error::new(value.to_string())
  }
}

impl convert::From<GrepRegexError> for Error {
  fn from(value: GrepRegexError) -> Self {
    Error::new(value.to_string())
  }
}

impl convert::From<RegexError> for Error {
  fn from(value: RegexError) -> Self {
    Error::new(value.to_string())
  }
}
