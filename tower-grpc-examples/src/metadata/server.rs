extern crate bytes;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate http;
extern crate prost;
extern crate tokio;
extern crate tower_grpc;
extern crate tower_hyper;
extern crate hyper;
extern crate tower_service;

pub mod metadata {
    include!(concat!(env!("OUT_DIR"), "/metadata.rs"));
}

use metadata::{server, EnterReply, EnterRequest};

use futures::{future, Future, Stream, Poll};
use tokio::net::TcpListener;
use tower_hyper::{Server, Body};
use tower_grpc::{Request, Response};
use tower_service::Service;

#[derive(Clone, Debug)]
struct Door;

impl server::Doorman for Door {
    type AskToEnterFuture = future::FutureResult<Response<EnterReply>, tower_grpc::Status>;

    fn ask_to_enter(&mut self, request: Request<EnterRequest>) -> Self::AskToEnterFuture {
        println!("REQUEST = {:?}", request);

        let metadata = request
            .metadata()
            .get("metadata")
            .and_then(|header| header.to_str().ok());

        let message = match metadata {
            Some("Here is a cookie") => "Yummy! Please come in.".to_string(),
            _ => "You cannot come in!".to_string(),
        };

        let response = Response::new(EnterReply { message });

        future::ok(response)
    }
}

pub fn main() {
    let _ = ::env_logger::init();

    let new_service = server::DoormanServer::new(Door);

    let mut server = Server::new(new_service);

    let addr = "[::1]:50051".parse().unwrap();
    let bind = TcpListener::bind(&addr).expect("bind");

    let serve = bind
        .incoming()
        .for_each(move |sock| {
            if let Err(e) = sock.set_nodelay(true) {
                return Err(e);
            }

            let serve = server.serve(sock);
            hyper::rt::spawn(serve.map_err(|e| error!("h2 error: {:?}", e)));

            Ok(())
        })
        .map_err(|e| eprintln!("accept error: {}", e));

    hyper::rt::run(serve);
}

impl<T> Service<hyper::Request<Body>> for server::DoormanServer<T> {
    type Response = hyper::Response<tower_hyper::Body>;
    type Error = hyper::Error;
    type Future = future::FutureResult<Self::Response, Self::Error>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, req: hyper::Request<Body>) -> Self::Future {
        let body = req.into_body();
        let res = hyper::Response::new(body);
        future::ok(res)
    }
}