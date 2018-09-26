extern crate crossbeam_channel as channel;
extern crate grep;
extern crate ignore;
extern crate regex;
extern crate serde;
extern crate serde_json as json;
#[macro_use]
extern crate serde_derive;

#[macro_use]
pub mod errors;
pub mod ext;
pub mod res;
pub mod search;

use std::path::Path;

fn main() {
  let dir = Path::new("/Users/sadikovi/developer/spark");
  let pattern = "execution";

  let res = search::find(dir, pattern, Vec::new()).unwrap();
  let j = json::to_string(&res).unwrap();
  println!("{}", j);
}
