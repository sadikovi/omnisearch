//! List of various errors used here.

use std::convert;
use std::fmt;
use std::sync::mpsc;

use grep::searcher::SinkError;

/// General error struct.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
  msg: String
}

impl Error {
  /// Creates a new error with provided message.
  pub fn new(msg: String) -> Self {
    Self { msg }
  }
}

macro_rules! err {
  ($fmt:expr) => (Err(errors::Error::new($fmt.to_owned())));
  ($fmt:expr, $($args:expr), *) => (Err(errors::Error::new(format!($fmt, $($args), *))));
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
