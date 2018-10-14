extern crate futures;
extern crate grep;
extern crate hyper;
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

use std::env;
use std::fs;
use std::panic;
use std::path;
use std::process;
use std::thread;
use std::time;

use futures::{future, Stream};
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode};
use hyper::header::CONTENT_TYPE;
use hyper::rt::Future;
use hyper::service::service_fn;

type BoxFuture = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

/// File name for storing connection parameters.
const PARAMS: &str = "PARAMS";
const LOCK: &str = "LOCK";
const LOCK_WAIT_MILLIS: u64 = 100;

/// Function to search and return JSON result.
fn find(params: params::QueryParams) -> Result<String, errors::Error> {
  let res = search::find(params.dir(), params.pattern(), Vec::new())?;
  json::to_string(&res).map_err(|err| errors::Error::new(err.to_string()))
}

// Function to convert error into a JSON string.
fn err2json(error: &errors::Error) -> String {
  json::to_string(&error)
    .unwrap_or("{\"err\":true,\"msg\":\"Server error\"}".to_owned())
}

fn service(req: Request<Body>) -> BoxFuture {
  match (req.method(), req.uri().path()) {
    (&Method::GET, "/ping") => {
      let mut response = Response::new(Body::empty());
      *response.status_mut() = StatusCode::OK;
      Box::new(future::ok(response))
    },
    (&Method::POST, "/search") => {
      let response = req
        .into_body()
        .concat2()
        .map(move |chunk| {
          let body = chunk.iter().cloned().collect::<Vec<u8>>();
          match json::from_slice::<params::QueryParams>(&body) {
            Ok(params) => {
              match find(params) {
                Ok(payload) => {
                  let mut response = Response::new(Body::from(payload));
                  *response.status_mut() = StatusCode::OK;
                  response.headers_mut().insert(
                    CONTENT_TYPE,
                    "application/json".parse().expect("correct content type value")
                  );
                  response
                }
                Err(error) => {
                  let mut response = Response::new(Body::from(err2json(&error)));
                  *response.status_mut() = StatusCode::BAD_REQUEST;
                  response
                }
              }
            },
            Err(error) => {
              let error = errors::Error::new(error.to_string());
              let mut response = Response::new(Body::from(err2json(&error)));
              *response.status_mut() = StatusCode::BAD_REQUEST;
              response
            }
          }
        });
      Box::new(response)
    },
    _ => {
      let error = errors::Error::new("404: Not Found".to_owned());
      let mut response = Response::new(Body::from(err2json(&error)));
      *response.status_mut() = StatusCode::NOT_FOUND;
      Box::new(future::ok(response))
    }
  }
}

/// Creates and synchronises lock.
fn sync_lock(lock: &path::Path) {
  let interval = time::Duration::from_millis(LOCK_WAIT_MILLIS);
  let max_interval = time::Duration::from_millis(100 * LOCK_WAIT_MILLIS);
  let now = time::Instant::now();
  while let Err(cause) = fs::OpenOptions::new().write(true).create_new(true).open(lock) {
    thread::sleep(interval);
    assert!(now.elapsed() <= max_interval, "Failed to acquire the lock: {}", cause);
  }
}

/// Drops existing lock.
fn drop_lock(lock: &path::Path) {
  if let Err(cause) = fs::remove_file(lock) {
    eprintln!("Failed to drop the lock: {}", cause);
  }
}

fn with_lock<F, T>(dir: &path::Path, func: F) -> T
    where F: Fn() -> T + panic::UnwindSafe {
  let lock = dir.join(LOCK);
  sync_lock(lock.as_path());
  let res = panic::catch_unwind(func);
  drop_lock(lock.as_path());
  match res {
    Ok(answer) => answer,
    Err(cause) => panic!("Internal error: {:?}", cause)
  }
}

/// Load connection parameters, if available.
fn load_connection_params(dir: &path::Path) -> Option<params::ConnectionParams> {
  let bytes = fs::read(dir.join(PARAMS)).ok();
  if let Some(bytes) = bytes {
    json::from_slice(&bytes).ok()
  } else {
    None
  }
}

/// Save connection parameters for this process.
fn save_connection_params(
  dir: &path::Path,
  opts: &params::ConnectionParams
) -> Option<()> {
  let bytes = json::to_vec(opts).ok();
  if let Some(bytes) = bytes {
    fs::write(dir.join(PARAMS), &bytes).ok()
  } else {
    None
  }
}

/// Ping the server, returns true if ping was successful.
fn ping(params: &params::ConnectionParams) -> bool {
  if let Ok(uri) = format!("http://{}/ping", params.address()).parse() {
    let client = Client::new();
    let (tx, rx) = futures::sync::oneshot::channel();
    let ping = client.get(uri)
      .map(|r| r.status().is_success())
      .or_else(|_| Ok(false))
      .map(|status| tx.send(status).expect("Channel send error"));
    hyper::rt::spawn(ping);
    rx.wait().unwrap_or(false)
  } else {
    false
  }
}

fn main() {
  let buf = env::current_dir().expect("Failed to retrieve current dir");
  hyper::rt::run(hyper::rt::lazy(move || {
    let dir = buf.as_path();
    with_lock(dir, || {
      match load_connection_params(dir).as_ref() {
        Some(ref params) if ping(params) => {
          println!("{}", params.address());
        },
        _ => {
          let initial_addr = ([127, 0, 0, 1], 0).into();
          let server = Server::bind(&initial_addr)
            .serve(|| service_fn(service));
          let opts = params::ConnectionParams::new(server.local_addr(), process::id());
          save_connection_params(dir, &opts);
          println!("{}", opts.address());
          hyper::rt::spawn(server.map_err(|e| eprintln!("Server error: {}", e)));
        }
      }
    });
    Ok(())
  }));
}
