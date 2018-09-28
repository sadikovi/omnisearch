use std::path::Path;

use ext::Extension;

/// Input struct that deserialized from JSON payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
  dir: String,
  pattern: String,
  extensions: Option<Vec<String>>
}

impl Params {
  /// Returns root directory.
  pub fn dir(&self) -> &Path {
    &Path::new(&self.dir)
  }

  /// Returns search pattern.
  pub fn pattern(&self) -> &str {
    &self.pattern
  }

  /// Returns list of extensions.
  pub fn extensions(&self) -> Vec<Extension> {
    match &self.extensions {
      Some(vec) => vec.iter().map(|ext| ext.parse().unwrap()).collect(),
      None => Vec::new()
    }
  }
}
