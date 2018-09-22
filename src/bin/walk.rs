extern crate crossbeam_channel as channel;
extern crate grep;
extern crate ignore;

use std::fmt;
use std::io;
use std::io::Write;
use std::str;
use std::thread;

use grep::regex::RegexMatcher;
use grep::searcher::*;
use ignore::{WalkBuilder, WalkState};

#[derive(Clone, Debug)]
pub struct Args {
  root: String,
  pattern: String,
  max_context_size: usize
}

impl Args {
  #[inline]
  pub fn root(&self) -> &str {
    self.root.as_ref()
  }

  #[inline]
  pub fn pattern(&self) -> &str {
    self.pattern.as_ref()
  }

  #[inline]
  pub fn max_context_size(&self) -> usize {
    self.max_context_size
  }
}

#[derive(Clone, Debug)]
struct ChannelSink {
  sx: channel::Sender<QueryResult>,
  path: String,
  items: Vec<QueryItem>,
  matches: Vec<QueryMatch>
}

impl ChannelSink {
  pub fn new(sx: channel::Sender<QueryResult>, path: String) -> Self {
    Self {
      sx: sx,
      path: path,
      // we keep "context * 2 + match" values
      items: Vec::with_capacity(8),
      // generally we do not expect more than 30 matches per file
      matches: Vec::with_capacity(32)
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
    let query_match = QueryMatch { items: match_items };
    self.matches.push(query_match);
  }
}

impl Sink for ChannelSink {
  type Error = io::Error;

  fn matched(&mut self, _: &Searcher, mat: &SinkMatch) -> Result<bool, io::Error> {
    // we do not support multi-line matches; every match that precedes the match is
    // considered a new query match and should be flushed.
    let should_flush = self.items.last()
      .map(|prev| {
        prev.kind() == QueryItemKind::Match || prev.kind() == QueryItemKind::After
      })
      .unwrap_or(false);
    if should_flush {
      self.flush_items();
    }

    let item = QueryItem::new(QueryItemKind::Match, mat.line_number(), mat.bytes());
    self.items.push(item);
    Ok(true)
  }

  fn context(&mut self, _: &Searcher, ctx: &SinkContext) -> Result<bool, io::Error> {
    match ctx.kind() {
      SinkContextKind::Before => {
        let should_flush = self.items.last()
          .map(|prev| {
            prev.kind() == QueryItemKind::Match || prev.kind() == QueryItemKind::After
          })
          .unwrap_or(false);
        if should_flush {
          self.flush_items();
        }

        let item = QueryItem::new(QueryItemKind::Before, ctx.line_number(), ctx.bytes());
        self.items.push(item);
      },
      SinkContextKind::After => {
        if let Some(prev) = self.items.last() {
          // should never happen
          assert!(prev.kind() != QueryItemKind::Before, "Kind cannot be Before");
        }
        let item = QueryItem::new(QueryItemKind::After, ctx.line_number(), ctx.bytes());
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
      let result = QueryResult {
        path: self.path.clone(),
        matches: matches
      };
      self.sx.send(result);
    }
    Ok(())
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryItemKind {
  Match,
  Before,
  After
}

#[derive(Clone, Debug)]
pub struct QueryItem {
  kind: QueryItemKind,
  line_num: u64,
  bytes: Vec<u8>
}

impl QueryItem {
  pub fn new(kind: QueryItemKind, line_num: Option<u64>, bytes: &[u8]) -> Self {
    // TODO: handle long lines
    Self {
      kind: kind,
      line_num: line_num.unwrap_or(0),
      bytes: bytes.to_vec()
    }
  }

  #[inline]
  pub fn kind(&self) -> QueryItemKind {
    self.kind
  }

  #[inline]
  pub fn is_match(&self) -> bool {
    self.kind == QueryItemKind::Match
  }
}

impl fmt::Display for QueryItem {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.kind() {
      QueryItemKind::Before => {
        write!(f, "==> {}: {}", self.line_num, str::from_utf8(&self.bytes).unwrap_or("Error\n"))
      },
      QueryItemKind::Match => {
        write!(f, "[!] {}: {}", self.line_num, str::from_utf8(&self.bytes).unwrap_or("Error\n"))
      },
      QueryItemKind::After => {
        write!(f, "<== {}: {}", self.line_num, str::from_utf8(&self.bytes).unwrap_or("Error\n"))
      }
    }
  }
}

#[derive(Clone, Debug)]
struct QueryMatch {
  items: Vec<QueryItem>
}

impl fmt::Display for QueryMatch {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "------------------------")?;
    for item in &self.items[..] {
      item.fmt(f)?;
    }
    writeln!(f, "------------------------")
  }
}

#[derive(Clone, Debug)]
struct QueryResult {
  path: String,
  matches: Vec<QueryMatch>
}

impl fmt::Display for QueryResult {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "### Path: {}", self.path)?;
    for mat in &self.matches[..] {
      mat.fmt(f)?;
    }
    writeln!(f, "")
  }
}

fn main() {
  let args = Args {
    root: "/Users/sadikovi/developer/spark".to_owned(),
    pattern: "execution".to_owned(),
    max_context_size: 2
  };

  let walk = WalkBuilder::new(args.root())
    .follow_links(false)
    .standard_filters(true)
    .same_file_system(true)
    .build_parallel();

  let (sx, rx) = channel::unbounded::<QueryResult>();

  let stdout_thread = thread::spawn(move || {
    let mut stdout = io::BufWriter::new(io::stdout());
    for result in rx {
      stdout.write(result.to_string().as_bytes()).unwrap();
      stdout.write(b"\n").unwrap();
    }
  });

  walk.run(|| {
    let sx = sx.clone();
    let args = args.clone();

    let matcher = RegexMatcher::new_line_matcher(args.pattern()).unwrap();
    let searcher = SearcherBuilder::new()
      .line_number(true)
      .before_context(args.max_context_size())
      .after_context(args.max_context_size())
      .multi_line(false)
      .build();

    Box::new(move |res| {
      if let Ok(dir) = res {
        if let Some(ftype) = dir.file_type() {
          if ftype.is_file() {
            // TODO: add check on supported extensions
            let sink = ChannelSink::new(
              sx.clone(),
              dir.path().to_str().unwrap().to_owned()
            );

            searcher.clone().search_path(
              matcher.clone(),
              dir.path(),
              sink
            ).unwrap();
          }
        }
      }
      WalkState::Continue
    })
  });

  drop(sx);
  stdout_thread.join().unwrap();
}
