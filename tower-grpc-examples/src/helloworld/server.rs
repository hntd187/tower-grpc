extern crate bytes;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate log;
extern crate prost;
extern crate tokio;
extern crate tower_grpc;
extern crate hyper;
extern crate tower_hyper;
extern crate tower_service;

pub mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));
}

use hello_world::{server, HelloReply, HelloRequest};

use futures::{future, Future, Poll, Stream};
use tower_hyper::body::Body;
use tower_hyper::server::Server;
use tower_service::Service;
use tokio::net::TcpListener;
use tower_grpc::{Request, Response};

#[derive(Clone, Debug)]
struct Greet;

impl server::Greeter for Greet {
    type SayHelloFuture = future::FutureResult<Response<HelloReply>, tower_grpc::Status>;

    fn say_hello(&mut self, request: Request<HelloRequest>) -> Self::SayHelloFuture {
        println!("REQUEST = {:?}", request);

        let response = Response::new(HelloReply {
            message: "Zomg, it works!".to_string(),
        });

        future::ok(response)
    }
}

pub fn main() {
    let _ = ::env_logger::init();

    let new_service = server::GreeterServer::new(Greet);

    let mut hyper = Server::new(new_service);
    let addr = "[::1]:50051".parse().unwrap();
    let bind = TcpListener::bind(&addr).expect("bind");

    let serve = bind
        .incoming()
        .for_each(move |sock| {
            if let Err(e) = sock.set_nodelay(true) {
                return Err(e);
            }

            let serve = hyper.serve(sock);
            hyper::rt::spawn(serve.map_err(|e| error!("h2 error: {:?}", e)));

            Ok(())
        })
        .map_err(|e| eprintln!("accept error: {}", e));

    hyper::rt::run(serve)
}

impl<T> Service<hyper::Request<tower_hyper::Body>> for server::GreeterServer<T> {
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