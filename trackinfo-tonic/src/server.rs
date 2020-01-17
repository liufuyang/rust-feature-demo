use gid_to_uuid::to_uuid;
use tonic::{Code, metadata::MetadataValue, Request, Response, Status, transport::Channel, transport::Server};

use metadata::{get_track_response::Entity, GetTrackRequest};
use metadata::metadata_client::MetadataClient;
use track_info::{TitleAndArtist, TrackToStringRequest};
use track_info::golden_path_example_service_server::{GoldenPathExampleService, GoldenPathExampleServiceServer};

pub mod track_info {
    tonic::include_proto!("spotify.goldenpathexamples");
}

pub mod metadata {
    tonic::include_proto!("spotify.metadata.v1beta1");
}


pub struct Service {
    metadata_client: MetadataClient<Channel>
}

// https://ghe.spotify.net/fabric/golden-path-examples/blob/master/src/main/java/com/spotify/goldenpathexamples/GrpcTrackProvider.java#L46
#[tonic::async_trait]
impl GoldenPathExampleService for Service {
    async fn track_to_string(
        &self,
        request: Request<TrackToStringRequest>,
    ) -> Result<Response<TitleAndArtist>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let track_id = request.into_inner().track_id; // We must use .into_inner() as the fields of gRPC requests and responses are private
        let track_uuid = match to_uuid(&track_id).await {
            Ok(uuid) => uuid,
            Err(e) => return Err(Status::new(Code::InvalidArgument, e.to_string()))
        };

        let track_request= tonic::Request::new(GetTrackRequest {
            gid: track_uuid,
            country: "US".into(),
            catalogue: "free".into(),
            accept_language: vec![],
            preview: true,
            view: 0,
            etag: vec![],
        });

        println!("Request -> {:?}", track_request);
        //  tonic's clients are always backed by channels so cloning them is cheap
        let mut client = self.metadata_client.clone();
        let response = client.get_track(track_request).await?;

        match response.into_inner().entity {
            Some(Entity::Track(track)) => {
                let reply = TitleAndArtist {
                    track_string: format!("{}, {}",
                                          track.name.unwrap_or("".to_string()),
                                          track.album.map(|a| a.name).flatten().unwrap_or("".to_string()))
                };
                Ok(Response::new(reply))
            }
            _ => {
                Err(Status::new(Code::NotFound, "No track entity found"))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50052".parse().unwrap();

    let channel = Channel::from_static("http://gew1-metadataproxygrpc-b-m9mx.gew1.spotify.net.:8080").connect().await?;
    let token = MetadataValue::from_static("IgJ1c3IgY2RiM2EzOTA4NWEzNDg2MzkxZDA1NDIxMWUwZTUyOGM=");
    let time = MetadataValue::from_str("5000000u")?; // 5 sec
    let metadata_client = MetadataClient::with_interceptor(channel, move |mut req: Request<()>| {

        // Noticing below "insert_bin" has to be used rather than "insert", as we have key tagged with -bin.
        // token is made with MetadataValue::from_static() which will not do b64 encode again.
        req.metadata_mut().insert_bin("spotify-userinfo-bin", token.clone());
        req.metadata_mut().insert("grpc-timeout", time.clone());

        Ok(req)
    });
    let service = Service {
        metadata_client,
    };

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(GoldenPathExampleServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

/*
    Use command like this to send request to server:

    grpcurl -plaintext -import-path ./proto -proto trackinfo.proto \
        -d '{"trackId": "1WHzHtbCV4OoB0TLgG7eMD"}' \
        localhost:50052 spotify.goldenpathexamples.GoldenPathExampleService/TrackToString


    https://spotify.stackenterprise.co/questions/3381/6285#6285
    curl http://b62.spotify.net/spotify:track:0KKeRdzSUP5yEPLakW6CFE
    dig -t srv +short _spotify-$service._grpc.services.gew1.spotify.net | awk 'NR==1{printf("%s:%s", $4, $3)}'

    grpcurl -plaintext -max-time 5 -d '{"gid": "18c5dd0479ad446b9f1bbbcfea8ce59e", "country": "DK", "catalogue": "free"}' \
        -H "spotify-userinfo-bin: IgJ1c3IgY2RiM2EzOTA4NWEzNDg2MzkxZDA1NDIxMWUwZTUyOGM=" \
        gew1-metadataproxygrpc-b-m9mx.gew1.spotify.net.:8080 spotify.metadata.v1beta1.Metadata/GetTrack
*/