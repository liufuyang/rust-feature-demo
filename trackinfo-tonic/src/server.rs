use tonic::{Request, Response, Status, transport::Server};

use track_info::{TrackToStringRequest, TitleAndArtist};
use track_info::golden_path_example_service_server::{GoldenPathExampleService, GoldenPathExampleServiceServer};

pub mod track_info {
    tonic::include_proto!("spotify.goldenpathexamples");
}

pub mod metadata {
    tonic::include_proto!("spotify.metadata");
}

#[derive(Default)]
pub struct Service {}

#[tonic::async_trait]
impl GoldenPathExampleService for Service {
    async fn track_to_string(
        &self,
        request: Request<TrackToStringRequest>,
    ) -> Result<Response<TitleAndArtist>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let request = request.into_inner(); // We must use .into_inner() as the fields of gRPC requests and responses are private

        let reply = TitleAndArtist {
            track_string: format!("Hello"),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50052".parse().unwrap();
    let service = Service::default();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(GoldenPathExampleServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

/* Use command like this to send request to server:
   grpcurl -plaintext -import-path ./proto -proto trackinfo.proto \
     -d '{"trackId": "Joe"}' \
     localhost:50052 spotify.goldenpathexamples.GoldenPathExampleService/TrackToString
*/