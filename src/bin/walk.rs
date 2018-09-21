extern crate crossbeam_channel as channel;
extern crate grep;
extern crate ignore;

use std::io;
use std::io::Write;
use std::path::Path;
use std::thread;

use grep::regex::RegexMatcher;
use grep::searcher::{Searcher, SearcherBuilder, Sink, SinkContext, SinkFinish, SinkMatch};
use ignore::{DirEntry, WalkBuilder, WalkState};

// Supported file extensions
#[derive(Clone, Copy, Debug)]
pub enum FileExt {
  Scala,
  Java,
  Rust
}

impl FileExt {
  #[inline]
  pub fn has_ext(&self, file_name: &str) -> bool {
    match self {
      FileExt::Scala => file_name.ends_with(".scala"),
      FileExt::Java => file_name.ends_with(".java"),
      FileExt::Rust => file_name.ends_with(".rs")
    }
  }
}

#[derive(Clone, Debug)]
pub struct Args {
  root: String,
  pattern: String,
  max_context_size: usize,
  ext: Vec<FileExt>
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

  #[inline]
  pub fn has_ext(&self, file_name: &str) -> bool {
    for ext in &self.ext[..] {
      if ext.has_ext(file_name) {
        return true;
      }
    }
    false
  }
}

#[derive(Clone, Debug)]
struct ChannelSink {
  sx: channel::Sender<QueryResultGroup>,
  path: String,
  items: Vec<QueryItem>
}

impl ChannelSink {
  pub fn new(sx: channel::Sender<QueryResultGroup>, path: String) -> Self {
    Self {
      sx: sx,
      path: path,
      items: Vec::new()
    }
  }
}

impl Sink for ChannelSink {
  type Error = io::Error;

  fn matched(&mut self, _: &Searcher, mat: &SinkMatch) -> Result<bool, io::Error> {
    let item = QueryItem {
      line_num: mat.line_number().unwrap_or(0),
      bytes: mat.bytes().to_vec()
    };
    self.items.push(item);
    Ok(true)
  }

  fn context(&mut self, _: &Searcher, ctx: &SinkContext) -> Result<bool, io::Error> {
    let item = QueryItem {
      line_num: ctx.line_number().unwrap_or(0),
      bytes: ctx.bytes().to_vec()
    };
    self.items.push(item);
    Ok(true)
  }

  fn finish(&mut self, _: &Searcher, _: &SinkFinish) -> Result<(), Self::Error> {
    let group = QueryResultGroup {
      path: self.path.clone(),
      results: Vec::new()
    };
    self.sx.send(group);
    Ok(())
  }
}

#[derive(Clone, Debug)]
struct QueryItem {
  line_num: u64,
  bytes: Vec<u8>
}

#[derive(Clone, Debug)]
struct QueryResult {
  items: Vec<QueryItem>,
  matched_idx: usize
}

#[derive(Clone, Debug)]
struct QueryResultGroup {
  path: String,
  results: Vec<QueryResult>
}

fn main() {
  let args = Args {
    root: "/Users/sadikovi/developer/spark".to_owned(),
    pattern: "val path =".to_owned(),
    max_context_size: 2,
    ext: vec![FileExt::Scala, FileExt::Java]
  };

  let walk = WalkBuilder::new(args.root())
    .follow_links(false)
    .standard_filters(true)
    .same_file_system(true)
    .build_parallel();

  let (sx, rx) = channel::unbounded::<QueryResultGroup>();

  let stdout_thread = thread::spawn(move || {
    let mut stdout = io::BufWriter::new(io::stdout());
    for dir in rx {
      stdout.write(dir.path.as_bytes()).unwrap();
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
          if ftype.is_file() && args.has_ext(dir.file_name().to_str().unwrap_or("")) {
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
