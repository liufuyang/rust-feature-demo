use tonic::{Request, Response, Status, transport::Server};

use hello_world::{HelloReply, HelloRequest};
use hello_world::greeter_server::{Greeter, GreeterServer};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let request = request.into_inner(); // We must use .into_inner() as the fields of gRPC requests and responses are private

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.name),
            length: request.name.len() as u32,
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}

/* Use command like this to send request to server:
   grpcurl -plaintext -import-path ./proto -proto helloworld.proto \
     -d '{"name": "Joe"}' \
     localhost:50051 helloworld.Greeter/SayHello
*/