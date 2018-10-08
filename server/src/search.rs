use std::path::Path;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use errors;
use ext::{Extension, Extensions};
use grep::regex::RegexMatcher;
use grep::searcher::*;
use ignore::{WalkBuilder, WalkState};
use regex::RegexBuilder;
use result::*;

// Maximum number of files we collect.
const FILE_MAX_MATCHES: usize = 10;
// Maximum number of matches we collect.
const CONTENT_MAX_MATCHES: usize = 100;
// Number of lines of context ot fetch.
const CONTEXT_NUM_LINES: usize = 2;

// Sink implementation for search.
#[derive(Clone, Debug)]
pub struct Collector {
  sx: mpsc::Sender<ContentItem>,
  counter: Arc<AtomicUsize>,
  path: String,
  ext: Extension,
  lines: Vec<ContentLine>,
  matches: Vec<ContentMatch>
}

impl Collector {
  /// Creates a new collector.
  pub fn new(
    sx: mpsc::Sender<ContentItem>,
    counter: Arc<AtomicUsize>,
    path: String,
    ext: Extension
  ) -> Self {
    Self {
      sx: sx,
      counter: counter,
      path: path,
      ext: ext,
      lines: Vec::with_capacity(32),
      matches: Vec::with_capacity(32)
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
      let line = ContentLine::new(ContentKind::Match, line_number, mat.bytes());
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
          let line = ContentLine::new(ContentKind::Before, line_number, ctx.bytes());
          self.lines.push(line);
        },
        SinkContextKind::After => {
          let line = ContentLine::new(ContentKind::After, line_number, ctx.bytes());
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
  extensions: Vec<Extension>
) -> Result<SearchResult, errors::Error> {
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
    Extensions::new()
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

  let content_matcher = RegexMatcher::new_line_matcher(pattern)?;

  let file_matcher = RegexBuilder::new(pattern)
    .case_insensitive(true)
    .multi_line(false)
    .build()?;

  let (fsx, frx) = mpsc::channel::<FileItem>();
  let (csx, crx) = mpsc::channel::<ContentItem>();

  let files_thread = thread::spawn(move || {
    let mut vec = Vec::new();
    for result in frx {
      vec.push(result);
    }
    vec
  });

  let content_thread = thread::spawn(move || {
    let mut vec = Vec::new();
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
    let file_matcher = file_matcher.clone();
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
              let matcher = content_matcher.clone();
              let collector = Collector::new(
                csx.clone(),
                content_counter.clone(),
                fpath.to_owned(),
                ext
              );
              searcher.search_path(matcher, inode.path(), collector).unwrap();
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

  Ok(SearchResult::new(files, file_matches, content, content_matches))
}
