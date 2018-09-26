use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicUsize, Ordering};

use errors;
use ext::Extension;
use grep::searcher::*;
use res::{ContentItem, ContentKind, ContentLine, ContentMatch};

// Maximum number of matches we collect.
const CONTENT_MAX_MATCHES: usize = 200;

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
