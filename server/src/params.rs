use std::path::Path;

/// Input struct that deserialized from JSON payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryParams {
  dir: String,
  pattern: String
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
}
