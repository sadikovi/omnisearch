extern crate futures;
extern crate grep;
extern crate hyper;
extern crate ignore;
extern crate serde;
extern crate serde_json as json;
#[macro_use]
extern crate serde_derive;

#[macro_use]
pub mod errors;
pub mod cache2;
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

// Function to convert error into a JSON string.
fn err2json(error: &errors::Error) -> String {
  json::to_string(&error)
    .unwrap_or("{\"err\":true,\"msg\":\"Server error\"}".to_owned())
}

fn service_inner(req: Request<Body>, cache: cache2::SharedCache) -> BoxFuture {
  match (req.method(), req.uri().path()) {
    (&Method::GET, "/ping") => {
      let mut response = Response::new(Body::empty());
      *response.status_mut() = StatusCode::OK;
      Box::new(future::ok(response))
    },
    (&Method::GET, "/cache/stats") => {
      let response = req
        .into_body()
        .concat2()
        .map(move |_| {
          let res = cache2::cache_stats(&cache)
            .and_then(|stats| json::to_string(&stats).map_err(|error| error.into()));
          match res {
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
        });
      Box::new(response)
    },
    (&Method::POST, "/cache/add") => {
      let response = req
        .into_body()
        .concat2()
        .map(move |chunk| {
          let body = chunk.iter().cloned().collect::<Vec<u8>>();
          let res = json::from_slice::<params::CacheParams>(&body)
            .map_err(|error| error.into())
            .and_then(|params| cache2::update_cache(&cache, params.dir()));
          match res {
            Ok(_) => {
              let mut response = Response::new(Body::empty());
              *response.status_mut() = StatusCode::OK;
              response
            }
            Err(error) => {
              let mut response = Response::new(Body::from(err2json(&error)));
              *response.status_mut() = StatusCode::BAD_REQUEST;
              response
            }
          }
        });
      Box::new(response)
    },
    (&Method::POST, "/search") => {
      let response = req
        .into_body()
        .concat2()
        .map(move |chunk| {
          let body = chunk.iter().cloned().collect::<Vec<u8>>();
          let res = json::from_slice::<params::QueryParams>(&body)
            .map_err(|error| error.into())
            .and_then(|params| search::find(&cache, params))
            .and_then(|res| json::to_string(&res).map_err(|error| error.into()));
          match res {
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
  let cache = cache2::create_cache();
  let tp = cache2::periodic_refresh(&cache);
  hyper::rt::run(hyper::rt::lazy(move || {
    let initial_addr = ([127, 0, 0, 1], 0).into();
    let server = Server::bind(&initial_addr)
      .serve(move || {
        let cache_arc = cache.clone();
        service_fn(move |req| {
          service_inner(req, cache_arc.clone())
        })
      });
    println!("{}", server.local_addr());
    hyper::rt::spawn(server.map_err(|e| eprintln!("Server error: {}", e)));
    Ok(())
  }));
  drop(tp);
}
