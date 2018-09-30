use std::net::SocketAddr;
use std::path::Path;

use ext::Extension;

/// Input struct that deserialized from JSON payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryParams {
  dir: String,
  pattern: String,
  extensions: Option<Vec<String>>
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

  /// Returns list of extensions.
  pub fn extensions(&self) -> Vec<Extension> {
    match &self.extensions {
      Some(vec) => vec.iter().map(|ext| ext.parse().unwrap()).collect(),
      None => Vec::new()
    }
  }
}

/// Connection params that are returned when server is started.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionParams {
  address: SocketAddr,
  pid: u32
}

impl ConnectionParams {
  /// Creates new connection params.
  pub fn new(address: SocketAddr, pid: u32) -> Self {
    Self { address, pid }
  }

  /// Returns address to connect.
  pub fn address(&self) -> SocketAddr {
    self.address
  }

  /// Returns process id.
  pub fn pid(&self) -> u32 {
    self.pid
  }
}
