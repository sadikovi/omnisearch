use std::path::Path;

/// Input struct that is deserialized from JSON payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryParams {
  dir: String,
  pattern: String,
  use_regex: Option<bool>,
  use_cache: Option<bool>
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

  /// Whether or not to use regex search.
  pub fn use_regex(&self) -> bool {
    self.use_regex.unwrap_or(false)
  }

  // Whether or not to use cache for search.
  pub fn use_cache(&self) -> bool {
    self.use_cache.unwrap_or(false)
  }
}

/// Input struct for cache parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheParams {
  dir: String
}

impl CacheParams {
  // Returns directory to cache.
  pub fn dir(&self) -> &Path {
    &Path::new(&self.dir)
  }
}
