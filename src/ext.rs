use std::collections;
use std::fmt;
use std::str;

use errors;
use serde::ser::{Serialize, Serializer};

/// Enumeration for all text-based supported file extensions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize)]
pub enum Extension {
  BZL,
  C,
  COFFEE,
  CPP,
  CSS,
  GO,
  H,
  HTML,
  JAVA,
  JS,
  JSON,
  JSX,
  M,
  MARKDOWN,
  MD,
  PHP,
  PL,
  PROTO,
  PY,
  PYST,
  RB,
  RS,
  SCALA,
  SCSS,
  SH,
  SQL,
  SWIFT,
  THRIFT,
  TSX,
  XML,
  YAML,
  YML,
  UNKNOWN
}

impl fmt::Display for Extension {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Extension::BZL => write!(f, "bzl"),
      Extension::C => write!(f, "c"),
      Extension::COFFEE => write!(f, "coffee"),
      Extension::CPP => write!(f, "cpp"),
      Extension::CSS => write!(f, "css"),
      Extension::GO => write!(f, "go"),
      Extension::H => write!(f, "h"),
      Extension::HTML => write!(f, "html"),
      Extension::JAVA => write!(f, "java"),
      Extension::JS => write!(f, "js"),
      Extension::JSON => write!(f, "json"),
      Extension::JSX => write!(f, "jsx"),
      Extension::M => write!(f, "m"),
      Extension::MARKDOWN => write!(f, "markdown"),
      Extension::MD => write!(f, "md"),
      Extension::PHP => write!(f, "php"),
      Extension::PL => write!(f, "pl"),
      Extension::PROTO => write!(f, "proto"),
      Extension::PY => write!(f, "py"),
      Extension::PYST => write!(f, "pyst"),
      Extension::RB => write!(f, "rb"),
      Extension::RS => write!(f, "rs"),
      Extension::SCALA => write!(f, "scala"),
      Extension::SCSS => write!(f, "scss"),
      Extension::SH => write!(f, "sh"),
      Extension::SQL => write!(f, "sql"),
      Extension::SWIFT => write!(f, "swift"),
      Extension::THRIFT => write!(f, "thrift"),
      Extension::TSX => write!(f, "tsx"),
      Extension::XML => write!(f, "xml"),
      Extension::YAML => write!(f, "yaml"),
      Extension::YML => write!(f, "yml"),
      Extension::UNKNOWN => write!(f, "<unknown>")
    }
  }
}

impl str::FromStr for Extension {
  type Err = errors::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.trim().to_lowercase().as_ref() {
      "bzl" => Ok(Extension::BZL),
      "c" => Ok(Extension::C),
      "coffee" => Ok(Extension::COFFEE),
      "cpp" => Ok(Extension::CPP),
      "css" => Ok(Extension::CSS),
      "go" => Ok(Extension::GO),
      "h" => Ok(Extension::H),
      "html" => Ok(Extension::HTML),
      "java" => Ok(Extension::JAVA),
      "js" => Ok(Extension::JS),
      "json" => Ok(Extension::JSON),
      "jsx" => Ok(Extension::JSX),
      "m" => Ok(Extension::M),
      "markdown" => Ok(Extension::MARKDOWN),
      "md" => Ok(Extension::MD),
      "php" => Ok(Extension::PHP),
      "pl" => Ok(Extension::PL),
      "proto" => Ok(Extension::PROTO),
      "py" => Ok(Extension::PY),
      "pyst" => Ok(Extension::PYST),
      "rb" => Ok(Extension::RB),
      "rs" => Ok(Extension::RS),
      "scala" => Ok(Extension::SCALA),
      "scss" => Ok(Extension::SCSS),
      "sh" => Ok(Extension::SH),
      "sql" => Ok(Extension::SQL),
      "swift" => Ok(Extension::SWIFT),
      "thrift" => Ok(Extension::THRIFT),
      "tsx" => Ok(Extension::TSX),
      "xml" => Ok(Extension::XML),
      "yaml" => Ok(Extension::YAML),
      "yml" => Ok(Extension::YML),
      _ => Ok(Extension::UNKNOWN)
    }
  }
}

impl Serialize for Extension {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    serializer.serialize_str(&self.to_string())
  }
}

/// Container struct to provide methods for checking supported extensions.
#[derive(Clone, Debug)]
pub struct Extensions {
  set: collections::HashSet<Extension>
}

impl Extensions {
  /// Creates new set with all supported extensions, except UNKNOWN.
  pub fn new() -> Self {
    let extensions = vec![
      Extension::BZL,
      Extension::C,
      Extension::COFFEE,
      Extension::CPP,
      Extension::CSS,
      Extension::GO,
      Extension::H,
      Extension::HTML,
      Extension::JAVA,
      Extension::JS,
      Extension::JSON,
      Extension::JSX,
      Extension::M,
      Extension::MARKDOWN,
      Extension::MD,
      Extension::PHP,
      Extension::PL,
      Extension::PROTO,
      Extension::PY,
      Extension::PYST,
      Extension::RB,
      Extension::RS,
      Extension::SCALA,
      Extension::SCSS,
      Extension::SH,
      Extension::SQL,
      Extension::SWIFT,
      Extension::THRIFT,
      Extension::TSX,
      Extension::XML,
      Extension::YAML,
      Extension::YML
    ];
    Self::with_extensions(extensions)
  }

  /// Creates set with provided extensions.
  pub fn with_extensions(extensions: Vec<Extension>) -> Self {
    let mut set = collections::HashSet::new();
    for ext in extensions {
      set.insert(ext);
    }
    Self { set }
  }

  /// Checks whether or not provided extension is in the set.
  pub fn is_supported_extension(&self, ext: Extension) -> bool {
    self.set.contains(&ext)
  }
}
