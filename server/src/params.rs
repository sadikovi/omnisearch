use std::path::Path;

/// Input struct that deserialized from JSON payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryParams {
  dir: String,
  pattern: String,
  use_regex: Option<bool>
}

impl QueryParams {
  /// Returns root directory.
  pub fn dir(&self) -> &Path {
    &Path::new(&self.dir)
  }

  /// Returns search pattern.
  pub fn pattern(&self) -> &str {
    &self.pattern
  }

  /// Use regex flag.
  pub fn use_regex(&self) -> bool {
    self.use_regex.unwrap_or(false)
  }
}
