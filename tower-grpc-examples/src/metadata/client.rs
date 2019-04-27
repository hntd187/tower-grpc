extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate http;
extern crate log;
extern crate prost;
extern crate tokio;
extern crate tower_grpc;
extern crate tower_hyper;
extern crate tower_request_modifier;
extern crate tower_service;
extern crate tower_util;
extern crate hyper;

use futures::{Future, Poll};
use tokio::net::tcp::{ConnectFuture, TcpStream};
use tower_grpc::Request;
use tower_hyper::client;
use tower_service::Service;
use tower_util::MakeService;

pub mod metadata {
    include!(concat!(env!("OUT_DIR"), "/metadata.rs"));
}

pub fn main() {
    let _ = ::env_logger::init();

    let uri: http::Uri = format!("http://[::1]:50051").parse().unwrap();
    let mut make_client = client::Connect::new(Dst, client::Builder::new());

    let doorman = make_client
        .make_service(())
        .map(move |conn| {
            use metadata::client::Doorman;

            let conn = tower_request_modifier::Builder::new()
                .set_origin(uri)
                .build(conn)
                .unwrap();

            Doorman::new(conn)
        })
        .and_then(|mut client| {
            use metadata::EnterRequest;

            let mut request = Request::new(EnterRequest {
                message: "Hello! Can I come in?".to_string(),
            });

            request
                .metadata_mut()
                .insert("metadata", "Here is a cookie".parse().unwrap());

            client
                .ask_to_enter(request)
                .map_err(|e| panic!("gRPC request failed; err={:?}", e))
        })
        .map(|response| {
            println!("RESPONSE = {:?}", response);
        })
        .map_err(|e| {
            println!("ERR = {:?}", e);
        });

    hyper::rt::run(doorman);
}

struct Dst;

impl Service<()> for Dst {
    type Response = TcpStream;
    type Error = ::std::io::Error;
    type Future = ConnectFuture;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, _: ()) -> Self::Future {
        let addr = "[::1]:50051".parse().unwrap();
        TcpStream::connect(&addr)
    }
}
