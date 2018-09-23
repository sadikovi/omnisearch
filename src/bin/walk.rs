extern crate crossbeam_channel as channel;
extern crate grep;
extern crate ignore;
extern crate regex;

use std::collections;
use std::fmt;
use std::io;
use std::io::Write;
use std::str;
use std::thread;

use grep::regex::RegexMatcher;
use grep::searcher::*;
use ignore::{WalkBuilder, WalkState};
use regex::RegexBuilder;

const SEARCH_CONTEXT_BEFORE: usize = 2;
const SEARCH_CONTEXT_AFTER: usize = 2;
// Should be greater than SEARCH_CONTEXT_BEFORE + SEARCH_CONTEXT_AFTER + 1
const CHANNEL_SINK_ITEMS_START_CAPACITY: usize = 8;
// We do not expect more than 32 matches per file, in general
const CHANNEL_SINK_MATCHES_START_CAPACITY: usize = 32;

struct FileExt {
  extensions: collections::HashSet<&'static str>
}

impl FileExt {
  fn new() -> Self {
    let mut set = collections::HashSet::new();
    set.insert("bzl");
    set.insert("c");
    set.insert("coffee");
    set.insert("cpp");
    set.insert("css");
    set.insert("go");
    set.insert("h");
    set.insert("html");
    set.insert("java");
    set.insert("js");
    set.insert("json");
    set.insert("jsx");
    set.insert("m");
    set.insert("markdown");
    set.insert("md");
    set.insert("php");
    set.insert("pl");
    set.insert("proto");
    set.insert("py");
    set.insert("pyst");
    set.insert("rb");
    set.insert("rs");
    set.insert("scala");
    set.insert("scss");
    set.insert("sh");
    set.insert("sql");
    set.insert("swift");
    set.insert("tsx");
    set.insert("xml");
    set.insert("yaml");
    set.insert("yml");

    Self {
      extensions: set
    }
  }

  fn is_supported_extension(&self, ext: Option<&str>) -> bool {
    match ext {
      Some(value) => self.extensions.contains(value),
      None => false
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ContentItemKind {
  Match,
  Before,
  After
}

#[derive(Clone, Debug)]
struct ContentItem {
  kind: ContentItemKind,
  line_num: u64,
  bytes: Vec<u8>
}

impl ContentItem {
  fn new(kind: ContentItemKind, line_num: Option<u64>, bytes: &[u8]) -> Self {
    Self {
      kind: kind,
      line_num: line_num.unwrap_or(0),
      bytes: bytes.to_vec()
    }
  }

  #[inline]
  fn kind(&self) -> ContentItemKind {
    self.kind
  }
}

impl fmt::Display for ContentItem {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.kind() {
      ContentItemKind::Before => {
        write!(f, "==> {}: {}", self.line_num, str::from_utf8(&self.bytes).unwrap())
      },
      ContentItemKind::Match => {
        write!(f, "[!] {}: {}", self.line_num, str::from_utf8(&self.bytes).unwrap())
      },
      ContentItemKind::After => {
        write!(f, "<== {}: {}", self.line_num, str::from_utf8(&self.bytes).unwrap())
      }
    }
  }
}

#[derive(Clone, Debug)]
struct ContentMatch {
  items: Vec<ContentItem>
}

impl fmt::Display for ContentMatch {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "------------------------")?;
    for item in &self.items[..] {
      item.fmt(f)?;
    }
    writeln!(f, "------------------------")
  }
}

#[derive(Clone, Debug)]
struct ContentSearch {
  path: String,
  matches: Vec<ContentMatch>
}

impl fmt::Display for ContentSearch {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "### Path: {}", self.path)?;
    for mat in &self.matches[..] {
      mat.fmt(f)?;
    }
    writeln!(f, "")
  }
}

#[derive(Clone, Debug)]
struct FileSearch {
  path: String
}

// Example of data structure that will be returned as an outcome of the search.
struct SearchResult {
  files: Vec<FileSearch>,
  content: Vec<ContentSearch>
}

#[derive(Clone, Debug)]
struct ContentSink {
  sx: channel::Sender<ContentSearch>,
  path: String,
  items: Vec<ContentItem>,
  matches: Vec<ContentMatch>
}

impl ContentSink {
  fn new(sx: channel::Sender<ContentSearch>, path: String) -> Self {
    Self {
      sx: sx,
      path: path,
      items: Vec::with_capacity(CHANNEL_SINK_ITEMS_START_CAPACITY),
      matches: Vec::with_capacity(CHANNEL_SINK_MATCHES_START_CAPACITY)
    }
  }

  #[inline]
  fn flush_items(&mut self) {
    // assert!(self.items.len() <= 8);
    // assert!(self.matches.len() <= 30);
    // close previous match
    let mut match_items = Vec::with_capacity(self.items.len());
    while let Some(item) = self.items.pop() {
      match_items.push(item);
    }
    match_items.reverse();
    let query_match = ContentMatch { items: match_items };
    self.matches.push(query_match);
  }
}

impl Sink for ContentSink {
  type Error = io::Error;

  fn matched(&mut self, _: &Searcher, mat: &SinkMatch) -> Result<bool, io::Error> {
    // we do not support multi-line matches; every match that precedes the match is
    // considered a new query match and should be flushed.
    let should_flush = self.items.last()
      .map(|prev| {
        prev.kind() == ContentItemKind::Match || prev.kind() == ContentItemKind::After
      })
      .unwrap_or(false);
    if should_flush {
      self.flush_items();
    }

    // TODO: handle long lines
    let item = ContentItem::new(ContentItemKind::Match, mat.line_number(), mat.bytes());
    self.items.push(item);
    Ok(true)
  }

  fn context(&mut self, _: &Searcher, ctx: &SinkContext) -> Result<bool, io::Error> {
    match ctx.kind() {
      SinkContextKind::Before => {
        let should_flush = self.items.last()
          .map(|prev| {
            prev.kind() == ContentItemKind::Match || prev.kind() == ContentItemKind::After
          })
          .unwrap_or(false);
        if should_flush {
          self.flush_items();
        }

        let item = ContentItem::new(ContentItemKind::Before, ctx.line_number(), ctx.bytes());
        self.items.push(item);
      },
      SinkContextKind::After => {
        if let Some(prev) = self.items.last() {
          // should never happen
          assert!(prev.kind() != ContentItemKind::Before, "Kind cannot be Before");
        }
        let item = ContentItem::new(ContentItemKind::After, ctx.line_number(), ctx.bytes());
        self.items.push(item);
      },
      // pass-through case
      _ => {}
    }
    Ok(true)
  }

  fn finish(&mut self, _: &Searcher, _: &SinkFinish) -> Result<(), Self::Error> {
    if self.items.len() > 0 || self.matches.len() > 0 {
      self.flush_items();
      let mut matches = Vec::with_capacity(self.matches.len());
      while let Some(mat) = self.matches.pop() {
        matches.push(mat);
      }
      // Make sure matches are in order from top to bottom of the file
      matches.reverse();
      let result = ContentSearch {
        path: self.path.clone(),
        matches: matches
      };
      self.sx.send(result);
    }
    Ok(())
  }
}

fn main() {
  let root = "/Users/sadikovi/developer/spark";
  let pattern = "execution";

  let walker = WalkBuilder::new(root)
    .follow_links(false)
    .standard_filters(true)
    .same_file_system(true)
    .build_parallel();
  let searcher = SearcherBuilder::new()
    .line_number(true)
    .before_context(SEARCH_CONTEXT_BEFORE)
    .after_context(SEARCH_CONTEXT_AFTER)
    .multi_line(false)
    .build();
  let content_matcher = RegexMatcher::new_line_matcher(pattern)
    .unwrap();
  let file_matcher = RegexBuilder::new(pattern)
    .case_insensitive(true)
    .multi_line(false)
    .build()
    .unwrap();

  let (csx, crx) = channel::unbounded::<ContentSearch>();
  let (fsx, frx) = channel::unbounded::<FileSearch>();

  let files_thread = thread::spawn(move || {
    let mut stdout = io::BufWriter::new(io::stdout());
    for result in frx {
      stdout.write(result.path.as_bytes()).unwrap();
      stdout.write(b"\n").unwrap();
    }
  });

  let content_thread = thread::spawn(move || {
    let mut stdout = io::BufWriter::new(io::stdout());
    for result in crx {
      stdout.write(result.to_string().as_bytes()).unwrap();
      stdout.write(b"\n").unwrap();
    }
  });

  walker.run(|| {
    let csx = csx.clone();
    let fsx = fsx.clone();
    let mut searcher = searcher.clone();
    let content_matcher = content_matcher.clone();
    let file_matcher = file_matcher.clone();
    let file_ext = FileExt::new();

    Box::new(move |res| {
      if let Ok(inode) = res {
        let is_file = inode.file_type().map(|ftype| ftype.is_file()).unwrap_or(false);
        if is_file && inode.path().to_str().is_some() {
          let fpath = inode.path().to_str().unwrap();
          let fname = inode.file_name().to_str().unwrap();

          // Search if file name matches pattern
          if file_matcher.is_match(fname) {
            fsx.send(FileSearch { path: fpath.to_owned() })
          }

          let ext = inode.path().extension().and_then(|os| os.to_str());
          if file_ext.is_supported_extension(ext) {
            let matcher = content_matcher.clone();
            let sink = ContentSink::new(csx.clone(), fpath.to_owned());
            searcher.search_path(matcher, inode.path(), sink).unwrap();
          }
        }
      }
      WalkState::Continue
    })
  });

  drop(csx);
  content_thread.join().unwrap();
  drop(fsx);
  files_thread.join().unwrap();
}
