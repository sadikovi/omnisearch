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
pub mod params;
pub mod result;
pub mod search;

use std::io;
use std::io::Write;

// Function to convert error into a JSON string.
fn err2json(error: &errors::Error) -> String {
  json::to_string(&error)
    .unwrap_or("{\"err\":true,\"msg\":\"Server error\"}".to_owned())
}

// Function to return search result.
fn get_search_results(query: &str) -> Result<String, errors::Error> {
  let params = json::from_str::<params::QueryParams>(query)?;
  let res = search::find(params.dir(), params.pattern(), Vec::new())?;
  let json_str = json::to_string(&res)?;
  Ok(json_str)
}

fn main() {
  loop {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    match get_search_results(&input) {
      Ok(json_str) => io::stdout().write(&json_str.into_bytes()).unwrap(),
      Err(error) => io::stderr().write(&err2json(&error).into_bytes()).unwrap()
    };
  }
}
