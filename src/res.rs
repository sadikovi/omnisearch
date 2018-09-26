use ext::Extension;

/// File search item where name matches user's regular expression.
#[derive(Clone, Debug)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContentKind {
  Before,
  Match,
  After
}

const MAX_PREFIX_LENGTH: usize = 120;
const MAX_SUFFIX_LENGTH: usize = 17;
// Length of 3 corresponds to the "..." bytes.
const MAX_LENGTH: usize = MAX_PREFIX_LENGTH + MAX_SUFFIX_LENGTH + 3;

/// Content search line that contains bytes matched by user's regular expression.
#[derive(Clone, Debug)]
pub struct ContentLine {
  kind: ContentKind,
  num: u64,
  bytes: Vec<u8>,
  truncated: bool
}

impl ContentLine {
  /// Creates new content line.
  /// Also checks if bytes exceed max length and truncates if necessary.
  pub fn new(kind: ContentKind, line_number: u64, bytes: &[u8]) -> Self {
    let len = bytes.len();
    let (all_bytes, is_truncated) = if len < MAX_LENGTH {
      (bytes.to_vec(), false)
    } else {
      let mut vec = Vec::with_capacity(MAX_LENGTH);
      vec.extend_from_slice(&bytes[..MAX_PREFIX_LENGTH]);
      vec.extend_from_slice(&[b'.', b'.', b'.']);
      vec.extend_from_slice(&bytes[len - MAX_SUFFIX_LENGTH..len]);
      (vec, true)
    };

    Self {
      kind: kind,
      num: line_number,
      bytes: all_bytes,
      truncated: is_truncated
    }
  }
}

/// Collection of lines that form a single match.
/// Contains context lines (before, after) and actual match lines.
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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
#[derive(Clone, Copy, Debug)]
pub enum Matched {
  Exact(usize),
  AtLeast(usize)
}

/// General search result that has file matches and content matches.
#[derive(Clone, Debug)]
pub struct SearchResult {
  files: Vec<FileItem>,
  file_matches: Matched,
  content: Vec<ContentItem>,
  content_matches: Matched
}
