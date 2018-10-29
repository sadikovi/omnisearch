use ext::Extension;
use serde::ser::{Serialize, SerializeStruct, Serializer};

/// File search item where name matches user's regular expression.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileItem {
  path: String,
  ext: Extension
}

impl FileItem {
  /// Creates new file item from path and file extension.
  pub fn new(path: String, ext: Extension) -> Self {
    Self { path, ext }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub enum ContentKind {
  Before,
  Match,
  After
}

impl Serialize for ContentKind {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    match self {
      ContentKind::Before => serializer.serialize_str("before"),
      ContentKind::Match => serializer.serialize_str("match"),
      ContentKind::After => serializer.serialize_str("after")
    }
  }
}

const MAX_PREFIX_LENGTH: usize = 120;
const MAX_SUFFIX_LENGTH: usize = 17;
// Length of 3 corresponds to the "..." bytes.
const MAX_LENGTH: usize = MAX_PREFIX_LENGTH + MAX_SUFFIX_LENGTH + 3;

/// Content search line that contains bytes matched by user's regular expression.
#[derive(Clone, Debug, Deserialize)]
pub struct ContentLine {
  kind: ContentKind,
  num: u64,
  bytes: Vec<u8>,
  range: Option<(usize, usize)>,
  truncated: bool
}

impl ContentLine {
  /// Creates new content line.
  /// Also checks if bytes exceed max length and truncates if necessary.
  pub fn new(
    kind: ContentKind, line_number: u64, bytes: &[u8],
    start: Option<usize>, end: Option<usize>
  ) -> Self {
    let len = bytes.len();
    let (all_bytes, is_truncated) = if len < MAX_LENGTH {
      (bytes.to_vec(), false)
    } else {
      let mut vec = Vec::with_capacity(MAX_LENGTH);
      if start.is_some() && end.is_some() {
        // TODO: Add proper truncation mechanism
        vec.extend_from_slice(bytes);
      } else {
        vec.extend_from_slice(&bytes[..MAX_PREFIX_LENGTH]);
        vec.extend_from_slice(&[b'.', b'.', b'.']);
        vec.extend_from_slice(&bytes[len - MAX_SUFFIX_LENGTH..len]);
      }
      (vec, true)
    };

    let range = if start.is_some() && end.is_some() {
      Some((start.unwrap(), end.unwrap()))
    } else {
      None
    };

    Self {
      kind: kind,
      num: line_number,
      bytes: all_bytes,
      range: range,
      truncated: is_truncated
    }
  }

  /// Creates new content line without match range.
  pub fn without_match(kind: ContentKind, line_number: u64, bytes: &[u8]) -> Self {
    Self::new(kind, line_number, bytes, None, None)
  }
}

impl Serialize for ContentLine {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut s = serializer.serialize_struct("ContentLine", 4)?;
    s.serialize_field("kind", &self.kind)?;
    s.serialize_field("num", &self.num)?;
    if let Some((start, end)) = self.range {
      s.serialize_field(
        "before_bytes",
        &String::from_utf8_lossy(&self.bytes[0..start])
      )?;
      s.serialize_field(
        "bytes",
        &String::from_utf8_lossy(&self.bytes[start..end])
      )?;
      s.serialize_field(
        "after_bytes",
        &String::from_utf8_lossy(&self.bytes[end..])
      )?;
    } else {
      s.serialize_field("bytes", &String::from_utf8_lossy(&self.bytes))?;
    }
    s.serialize_field("truncated", &self.truncated)?;
    s.end()
  }
}

/// Collection of lines that form a single match.
/// Contains context lines (before, after) and actual match lines.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentMatch {
  lines: Vec<ContentLine>
}

impl ContentMatch {
  /// Creates a new content match with provided lines.
  pub fn new(lines: Vec<ContentLine>) -> Self {
    Self { lines }
  }
}

/// Content item that has matches for user's regular expression.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentItem {
  path: String,
  ext: Extension,
  matches: Vec<ContentMatch>
}

impl ContentItem {
  /// Creates a new content item.
  pub fn new(path: String, ext: Extension, matches: Vec<ContentMatch>) -> Self {
    Self { path, ext, matches }
  }
}

/// Number of matches found, either exact number (less or equal to) or
/// at least number (greater than).
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Matched {
  Exact(usize),
  AtLeast(usize)
}

impl Serialize for Matched {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let mut s = serializer.serialize_struct("Matched", 2)?;
    match self {
      Matched::Exact(value) => {
        s.serialize_field("count", &value)?;
        s.serialize_field("match", "exact")?;
      },
      Matched::AtLeast(value) => {
        s.serialize_field("count", &value)?;
        s.serialize_field("match", "atleast")?;
      }
    }
    s.end()
  }
}

/// General search result that has file matches and content matches.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
  time_sec: f64, // time taken to execute query
  used_cache: bool, // whether or not cache was used for search
  files: Vec<FileItem>,
  file_matches: Matched,
  content: Vec<ContentItem>,
  content_matches: Matched
}

impl SearchResult {
  /// Creates a new search result.
  pub fn new(
    time_sec: f64,
    used_cache: bool,
    files: Vec<FileItem>,
    file_matches: Matched,
    content: Vec<ContentItem>,
    content_matches: Matched
  ) -> Self {
    Self { time_sec, used_cache, files, file_matches, content, content_matches }
  }
}
