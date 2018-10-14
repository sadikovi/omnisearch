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

use futures::{future, Stream};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::header::CONTENT_TYPE;
use hyper::rt::Future;
use hyper::service::service_fn;

type BoxFuture = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

/// Function to search and return JSON result.
fn get_search_results(params: params::QueryParams) -> Result<String, errors::Error> {
  let res = search::find(params.dir(), params.pattern(), Vec::new())?;
  let json_str = json::to_string(&res)?;
  Ok(json_str)
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
              match get_search_results(params) {
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

fn main() {
  hyper::rt::run(hyper::rt::lazy(move || {
    let initial_addr = ([127, 0, 0, 1], 0).into();
    let server = Server::bind(&initial_addr)
      .serve(|| service_fn(service));
    println!("{}", server.local_addr());
    hyper::rt::spawn(server.map_err(|e| eprintln!("Server error: {}", e)));
    Ok(())
  }));
}
