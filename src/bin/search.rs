extern crate grep;

use std::io;
use std::path;
use std::str;

use grep::regex::RegexMatcher;
use grep::searcher::{Searcher, SearcherBuilder, Sink, SinkContext, SinkFinish, SinkMatch};

fn main() {
  let pattern = "execution";
  let matcher = RegexMatcher::new_line_matcher(pattern).unwrap();
  let mut searcher = SearcherBuilder::new()
    .line_number(true)
    .before_context(3)
    .after_context(3)
    .multi_line(true)
    .build();

  let sink = TestSink::new();

  // let path = path::Path::new("/Users/sadikovi/developer/omnisearch/LICENSE");
  let path = path::Path::new("/Users/sadikovi/developer/spark/pom.xml");
  println!("Start search");
  searcher.search_path(matcher, path, sink).unwrap();
  println!("Finish search");
}

struct TestSink {
}

impl TestSink {
  pub fn new() -> Self {
    Self { }
  }
}

impl Sink for TestSink {
  type Error = io::Error;

  fn matched(&mut self, _src: &Searcher, mat: &SinkMatch) -> Result<bool, io::Error> {
    println!("match [{:?}]: {:?}", mat.line_number(), str::from_utf8(mat.bytes()).unwrap());
    Ok(true)
  }

  fn context(&mut self, _src: &Searcher, ctx: &SinkContext) -> Result<bool, io::Error> {
    println!("context {:?} [{:?}]: {:?}", ctx.kind(), ctx.line_number(), str::from_utf8(ctx.bytes()).unwrap());
    Ok(true)
  }

  fn context_break(&mut self, src: &Searcher) -> Result<bool, io::Error> {
    println!("== [context break!] ==");
    Ok(true)
  }

  fn begin(&mut self, src: &Searcher) -> Result<bool, Self::Error> {
    println!("begin!");
    Ok(true)
  }

  fn finish(&mut self, src: &Searcher, fns: &SinkFinish) -> Result<(), io::Error> {
    println!("finish!");
    Ok(())
  }
}
