#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde;

use futures::future::{self, Future};
use hyper::service::service_fn;
use hyper::{rt::Stream, Body, Method, Request, Response, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;

use std::env;

mod db;
mod models;
mod schema;

use db::{add_tickets, connect_to_db, search};

static NOTFOUND: &[u8] = b"Not Found";

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type FutureBox<T> = Box<Future<Item = T, Error = GenericError> + Send>;

#[derive(Deserialize, Serialize, Debug)]
struct AddSuccess {
    status: String,
}

fn parse_request<T>(req: Request<Body>) -> impl Future<Item = T, Error = GenericError>
where
    T: DeserializeOwned + Send,
{
    Box::new(
        req.into_body()
            .concat2()
            .from_err()
            .and_then(|body_chunk| {
                let body_str = String::from_utf8(body_chunk.to_vec())?;
                Ok(body_str)
            })
            .and_then(|s| {
                let parsed = serde_json::from_str::<T>(&s)?;
                Ok(parsed)
            }),
    )
}

fn make_success_response<T: Serialize>(payload: T) -> FutureBox<Response<Body>> {
    Box::new(future::ok(
        Response::builder()
            .header("Content-Type", "application/json")
            .status(StatusCode::OK)
            .body(Body::from(json!(payload).to_string()))
            .unwrap(),
    ))
}

fn make_error_response(e: GenericError) -> FutureBox<Response<Body>> {
    println!("{:?}", e);
    let json = json!({ "status": format!("{:?}", e) }).to_string();
    Box::new(future::ok(
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(Body::from(json))
            .unwrap(),
    ))
}

fn ticket_service(req: Request<Body>) -> FutureBox<Response<Body>> {
    let db_connection = match connect_to_db() {
        Ok(connection) => connection,
        Err(e) => return make_error_response(e),
    };
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/batch_insert") => Box::new(
            parse_request(req)
                .and_then(move |ticket_list| add_tickets(ticket_list, &db_connection))
                .map(|_| AddSuccess {
                    status: String::from("success"),
                })
                .and_then(make_success_response)
                .or_else(make_error_response),
        ),
        (&Method::POST, "/search") => Box::new(
            parse_request(req)
                .and_then(move |search_request| search(search_request, db_connection))
                .and_then(make_success_response)
                .or_else(make_error_response),
        ),
        _ => {
            let body = Body::from(NOTFOUND);
            Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body)
                    .unwrap(),
            ))
        }
    }
}

fn main() {
    let addr_s = format!(
        "0.0.0.0:{}",
        env::var("PORT").unwrap_or(String::from("8080"))
    );

    let addr = addr_s.parse().unwrap();

    let server = hyper::server::Server::bind(&addr)
        .serve(|| service_fn(ticket_service))
        .map_err(|e| println!("{:?}", e));

    println!("Listening on http://{}", addr_s);

    hyper::rt::run(server);
}
