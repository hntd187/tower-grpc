extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate http;
extern crate log;
extern crate prost;
extern crate tokio;
extern crate tower_grpc;
extern crate tower_request_modifier;
extern crate tower_service;
extern crate tower_util;
extern crate tower_hyper;
extern crate hyper;

use futures::{Future, Poll};
use tokio::net::tcp::{ConnectFuture, TcpStream};
use tower_grpc::Request;
use tower_service::Service;
use tower_util::MakeService;

pub mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));
}

pub fn main() {
    let _ = ::env_logger::init();

    let uri: http::Uri = format!("http://[::1]:50051").parse().unwrap();
    let mut make_client = tower_hyper::Connect::new(Dst, tower_hyper::client::Builder::new());

    let say_hello = make_client
        .make_service(())
        .map(move |conn| {
            use hello_world::client::Greeter;

            let conn = tower_request_modifier::Builder::new()
                .set_origin(uri)
                .build(conn)
                .unwrap();

            Greeter::new(conn)
        })
        .and_then(|mut client| {
            use hello_world::HelloRequest;

            client
                .say_hello(Request::new(HelloRequest {
                    name: "What is in a name?".to_string(),
                }))
                .map_err(|e| panic!("gRPC request failed; err={:?}", e))
        })
        .and_then(|response| {
            println!("RESPONSE = {:?}", response);
            Ok(())
        })
        .map_err(|e| {
            println!("ERR = {:?}", e);
        });

    hyper::rt::run(say_hello);
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
