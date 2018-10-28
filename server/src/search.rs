use std::path::Path;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time;

use errors;
use ext::{Extension, Extensions};
use grep::matcher::{Match, Matcher, NoCaptures};
use grep::regex::{RegexMatcher, RegexMatcherBuilder};
use grep::searcher::*;
use ignore::{WalkBuilder, WalkState};
use result::*;

// Maximum number of files we collect.
const FILE_MAX_MATCHES: usize = 10;
// Maximum number of matches we collect.
const CONTENT_MAX_MATCHES: usize = 100;
// Number of lines of context ot fetch.
const CONTEXT_NUM_LINES: usize = 2;

// Direct matcher to match as substring.
#[derive(Clone, Debug)]
struct DirectMatcher {
  pattern: Vec<u8>
}

impl DirectMatcher {
  // Creates new direct matcher
  pub fn new(pattern: &str) -> Self {
    Self { pattern: pattern.as_bytes().to_vec() }
  }
}

impl Matcher for DirectMatcher {
  type Captures = NoCaptures;
  type Error = errors::Error;

  fn find_at(&self, haystack: &[u8], _at: usize) -> Result<Option<Match>, Self::Error> {
    let plen = self.pattern.len();
    let hlen = haystack.len();
    if plen > hlen {
      return Ok(None);
    }
    for i in 0..hlen - plen + 1 {
      if &self.pattern[..] == &haystack[i..i + plen] {
        return Ok(Some(Match::new(i, i+plen)))
      }
    }
    Ok(None)
  }

  fn new_captures(&self) -> Result<Self::Captures, Self::Error> {
    Ok(NoCaptures::new())
  }
}

#[derive(Clone)]
struct MatcherSpec {
  direct: Option<DirectMatcher>,
  regex: Option<RegexMatcher>
}

impl MatcherSpec {
  // Creates regex matcher.
  fn regex(matcher: RegexMatcher) -> Self {
    Self { direct: None, regex: Some(matcher) }
  }

  // Creates direct matcher.
  fn direct(matcher: DirectMatcher) -> Self {
    Self { direct: Some(matcher), regex: None }
  }

  // Returns true if regex is set.
  #[inline]
  fn is_regex(&self) -> bool {
    self.regex.is_some()
  }

  // Converts spec into RegexMatcher.
  #[inline]
  fn as_regex(self) -> RegexMatcher {
    self.regex.unwrap()
  }

  // Converts spec into DirectMatcher.
  #[inline]
  fn as_direct(self) -> DirectMatcher {
    self.direct.unwrap()
  }

  // Finds match in haystack.
  #[inline]
  fn find(&self, haystack: &[u8]) -> Result<Option<Match>, errors::Error> {
    let res = if self.is_regex() {
      self.regex.as_ref().unwrap().find(haystack)?
    } else {
      self.direct.as_ref().unwrap().find(haystack)?
    };
    Ok(res)
  }

  #[inline]
  fn is_match(&self, haystack: &str) -> bool {
    if self.is_regex() {
      self.regex.as_ref().unwrap().is_match(haystack.as_bytes()).unwrap_or(false)
    } else {
      self.direct.as_ref().unwrap().is_match(haystack.as_bytes()).unwrap_or(false)
    }
  }
}

// Sink implementation for search.
#[derive(Clone)]
struct Collector {
  sx: mpsc::Sender<ContentItem>,
  counter: Arc<AtomicUsize>,
  path: String,
  ext: Extension,
  lines: Vec<ContentLine>,
  matches: Vec<ContentMatch>,
  // Used to find location of the match
  spec: MatcherSpec
}

impl Collector {
  /// Creates a new collector.
  pub fn new(
    sx: mpsc::Sender<ContentItem>,
    counter: Arc<AtomicUsize>,
    path: String,
    spec: MatcherSpec,
    ext: Extension
  ) -> Self {
    Self {
      sx: sx,
      counter: counter,
      path: path,
      ext: ext,
      lines: Vec::with_capacity(32),
      matches: Vec::with_capacity(32),
      spec: spec
    }
  }

  #[inline]
  fn flush_lines(&mut self) {
    let mut match_lines = Vec::with_capacity(self.lines.len());
    while let Some(line) = self.lines.pop() {
      match_lines.push(line);
    }
    match_lines.reverse();
    self.matches.push(ContentMatch::new(match_lines));
  }
}

impl Sink for Collector {
  type Error = errors::Error;

  fn matched(&mut self, _: &Searcher, mat: &SinkMatch) -> Result<bool, Self::Error> {
    if let Some(line_number) = mat.line_number() {
      self.counter.fetch_add(1, Ordering::Relaxed);
      let loc = self.spec.find(mat.bytes())?;
      let start = loc.map(|m| m.start());
      let end = loc.map(|m| m.end());
      let line =
        ContentLine::new(ContentKind::Match, line_number, mat.bytes(), start, end);
      self.lines.push(line);
      Ok(true)
    } else {
      err!("Line numbers are not enabled")
    }
  }

  fn context(&mut self, _: &Searcher, ctx: &SinkContext) -> Result<bool, Self::Error> {
    if let Some(line_number) = ctx.line_number() {
      match ctx.kind() {
        SinkContextKind::Before => {
          let line =
            ContentLine::without_match(ContentKind::Before, line_number, ctx.bytes());
          self.lines.push(line);
        },
        SinkContextKind::After => {
          let line =
            ContentLine::without_match(ContentKind::After, line_number, ctx.bytes());
          self.lines.push(line);
        },
        // pass-through case
        _ => {}
      }
      Ok(true)
    } else {
      err!("Line numbers are not enabled")
    }
  }

  fn context_break(&mut self, _: &Searcher) -> Result<bool, Self::Error> {
    if self.counter.load(Ordering::Relaxed) > CONTENT_MAX_MATCHES {
      return Ok(false);
    }
    if self.lines.len() > 0 {
      self.flush_lines();
    }
    Ok(true)
  }

  fn finish(&mut self, _: &Searcher, _: &SinkFinish) -> Result<(), Self::Error> {
    if self.lines.len() > 0 {
      self.flush_lines();
    }
    if self.matches.len() > 0 {
      let mut matches = Vec::with_capacity(self.matches.len());
      while let Some(mat) = self.matches.pop() {
        matches.push(mat);
      }
      // Make sure matches are in order from top to bottom of the file
      matches.reverse();
      self.sx.send(ContentItem::new(self.path.clone(), self.ext, matches))?;
    }
    Ok(())
  }
}

// Perform search within provided directory using provided pattern
pub fn find(
  dir: &Path,
  pattern: &str,
  use_regex: bool,
  extensions: Vec<Extension>
) -> Result<SearchResult, errors::Error> {
  let start_time = time::Instant::now();

  let dir = dir.canonicalize()?;
  let path = dir.as_path();
  if !path.is_dir() {
    return err!("Path {} is not a directory", path.to_str().unwrap_or(""));
  }

  if pattern.len() == 0 {
    return err!("Empty pattern, expected a valid search word or regular expression");
  }

  // Set of extensions to check against.
  let ext_check = if extensions.len() > 0 {
    Extensions::with_extensions(extensions)
  } else {
    Extensions::all()
  };

  let walker = WalkBuilder::new(path)
    .follow_links(false)
    .standard_filters(true)
    .same_file_system(true)
    .build_parallel();

  let searcher = SearcherBuilder::new()
    .line_number(true)
    .before_context(CONTEXT_NUM_LINES)
    .after_context(CONTEXT_NUM_LINES)
    .multi_line(false)
    .build();

  let content_matcher = if use_regex {
    MatcherSpec::regex(
      RegexMatcherBuilder::new()
        .line_terminator(Some(b'\n'))
        .multi_line(false)
        .case_smart(true)
        .build(pattern)?
    )
  } else {
    MatcherSpec::direct(DirectMatcher::new(pattern))
  };

  let (fsx, frx) = mpsc::channel::<FileItem>();
  let (csx, crx) = mpsc::channel::<ContentItem>();

  let files_thread = thread::spawn(move || {
    let mut vec = Vec::with_capacity(FILE_MAX_MATCHES * 2);
    for result in frx {
      vec.push(result);
    }
    vec
  });

  let content_thread = thread::spawn(move || {
    let mut vec = Vec::with_capacity(CONTENT_MAX_MATCHES * 2);
    for result in crx {
      vec.push(result);
    }
    vec
  });

  let content_counter = Arc::new(AtomicUsize::new(0));
  let file_counter = Arc::new(AtomicUsize::new(0));

  walker.run(|| {
    let fsx = fsx.clone();
    let csx = csx.clone();
    let mut searcher = searcher.clone();
    let file_matcher = content_matcher.clone();
    let content_matcher = content_matcher.clone();
    let ext_check = ext_check.clone();

    let file_counter = file_counter.clone();
    let content_counter = content_counter.clone();

    Box::new(move |res| {
      if let Ok(inode) = res {
        let is_file = inode.file_type().map(|ftype| ftype.is_file()).unwrap_or(false);
        if is_file && inode.path().to_str().is_some() {
          // Path and name must exist at this point.
          let fpath = inode.path().to_str().unwrap();
          let fname = inode.file_name().to_str().unwrap();
          // It is okay to unwrap when parsing extension - always returns a valid enum.
          let ext = inode.path().extension()
            .and_then(|os| os.to_str())
            .unwrap_or("")
            .parse::<Extension>()
            .unwrap();

          // Search if file name matches pattern.
          if file_matcher.is_match(fname) {
            if file_counter.fetch_add(1, Ordering::Relaxed) <= FILE_MAX_MATCHES {
              let _ = fsx.send(FileItem::new(fpath.to_owned(), ext));
            }
          }

          if ext_check.is_supported_extension(ext) {
            if content_counter.load(Ordering::Relaxed) <= CONTENT_MAX_MATCHES {
              let content_matcher = content_matcher.clone();
              let collector = Collector::new(
                csx.clone(),
                content_counter.clone(),
                fpath.to_owned(),
                content_matcher.clone(),
                ext
              );
              if content_matcher.is_regex() {
                let matcher = content_matcher.as_regex().clone();
                searcher.search_path(matcher, inode.path(), collector).unwrap();
              } else {
                let matcher = content_matcher.as_direct().clone();
                searcher.search_path(matcher, inode.path(), collector).unwrap();
              }
            }
          }
        }
      }

      if file_counter.load(Ordering::Relaxed) > FILE_MAX_MATCHES &&
          content_counter.load(Ordering::Relaxed) > CONTENT_MAX_MATCHES {
        WalkState::Quit
      } else {
        WalkState::Continue
      }
    })
  });

  drop(fsx);
  let files = files_thread.join().unwrap();
  drop(csx);
  let content = content_thread.join().unwrap();

  let file_matches = if files.len() <= FILE_MAX_MATCHES {
    Matched::Exact(files.len())
  } else {
    Matched::AtLeast(files.len())
  };

  let content_matches = if content.len() <= CONTENT_MAX_MATCHES {
    Matched::Exact(content.len())
  } else {
    Matched::AtLeast(content.len())
  };

  let duration = start_time.elapsed();
  let exec_time = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;

  Ok(SearchResult::new(exec_time, files, file_matches, content, content_matches))
}
