extern crate crossbeam_channel as channel;
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
pub mod args;
pub mod res;
pub mod search;

use futures::{future, Stream};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::header::CONTENT_TYPE;
use hyper::rt::Future;
use hyper::service::service_fn;

type BoxFuture = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

/// Function to search and return JSON result.
fn find(params: args::Params) -> Result<String, errors::Error> {
  let res = search::find(params.dir(), params.pattern(), Vec::new())?;
  json::to_string(&res).map_err(|err| errors::Error::new(err.to_string()))
}

fn service(req: Request<Body>) -> BoxFuture {
  match (req.method(), req.uri().path()) {
    (&Method::POST, "/search") => {
      let response = req
        .into_body()
        .concat2()
        .map(move |chunk| {
          let body = chunk.iter().cloned().collect::<Vec<u8>>();
          match json::from_slice::<args::Params>(&body) {
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
                  let mut response = Response::new(Body::from(error.to_string()));
                  *response.status_mut() = StatusCode::BAD_REQUEST;
                  response
                }
              }
            },
            Err(error) => {
              let mut response = Response::new(Body::from(error.to_string()));
              *response.status_mut() = StatusCode::BAD_REQUEST;
              response
            }
          }
        });
      Box::new(response)
    },
    _ => {
      let mut response = Response::new(Body::empty());
      *response.status_mut() = StatusCode::NOT_FOUND;
      Box::new(future::ok(response))
    }
  }
}

fn main() {
  // Socket address, bind to any available port
  let addr = ([127, 0, 0, 1], 0).into();
  let server = Server::bind(&addr)
    .serve(|| service_fn(service));
  println!("{}", server.local_addr());
  hyper::rt::run(server.map_err(|e| eprintln!("Server error: {}", e)));
}
