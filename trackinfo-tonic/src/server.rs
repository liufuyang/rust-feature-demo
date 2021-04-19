use std::str::Utf8Error;
use std::task::{Context, Poll};

use http::{HeaderValue, Request as HttpRequest};
use tonic::transport::{Channel, Server};
use tonic::{metadata::MetadataValue, Code, Request, Response, Status};
use tower_service::Service as TowerService;

use metadata::metadata_client::MetadataClient;
use metadata::{get_track_response::Entity, GetTrackRequest};
use track_info::golden_path_example_service_server::{
    GoldenPathExampleService, GoldenPathExampleServiceServer,
};
use track_info::{TitleAndArtist, TrackToStringRequest};

pub mod track_info {
    tonic::include_proto!("spotify.goldenpathexamples");
}

pub mod metadata {
    tonic::include_proto!("spotify.metadata.v1beta1");
}

pub struct Service {
    metadata_client: MetadataClient<Timeout<Channel>>,
}

fn to_uuid(input: &str) -> Result<String, Utf8Error> {
    let hex = rb62::get_hex(input).unwrap();
    std::str::from_utf8(&hex).map(|s| s.to_string())
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
        let track_uuid = match to_uuid(&track_id) {
            Ok(uuid) => uuid,
            Err(e) => return Err(Status::new(Code::InvalidArgument, e.to_string())),
        };

        let track_request = tonic::Request::new(GetTrackRequest {
            gid: track_uuid,
            country: "US".into(),
            catalogue: "free".into(),
            accept_language: vec![],
            preview: true,
            view: 0,
            etag: vec![],
        });
        // track_request.metadata_mut().insert("grpc-timeout", MetadataValue::from_static("5S"));
        // not working as "grpc-timeout" is restricted

        println!("Request -> {:?}", track_request);
        //  tonic's clients are always backed by channels so cloning them is cheap
        let mut client = self.metadata_client.clone();
        let response = client.get_track(track_request).await?;
        println!("response -> {:?}", response);

        match response.into_inner().entity {
            Some(Entity::Track(track)) => {
                let reply = TitleAndArtist {
                    track_string: format!(
                        "{}, {}",
                        track.name.unwrap_or("".to_string()),
                        track
                            .album
                            .map(|a| a.name)
                            .flatten()
                            .unwrap_or("".to_string())
                    ),
                };
                Ok(Response::new(reply))
            }
            _ => Err(Status::new(Code::NotFound, "No track entity found")),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50052".parse().unwrap();

    let channel =
        Channel::from_static("http://gew1-metadataproxygrpc-a-q9gg.gew1.spotify.net.:20119")
            .connect()
            .await?;
    let token = MetadataValue::from_static("IgJ1c3IgY2RiM2EzOTA4NWEzNDg2MzkxZDA1NDIxMWUwZTUyOGM=");
    let timeout_client = Timeout::new(
        channel,
        "5S", // 5 sec, see timeout unit definition at https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md
    );

    let metadata_client =
        MetadataClient::with_interceptor(timeout_client, move |mut req: Request<()>| {
            // Noticing below "insert_bin" has to be used rather than "insert", as we have key tagged with -bin.
            // token is made with MetadataValue::from_static() which will not do b64 encode again.
            req.metadata_mut()
                .insert_bin("spotify-userinfo-bin", token.clone());
            println!("interceptor block req: {:?}", req.metadata());
            // above header change of `grpc-timeout` for now only works with a patch to tonic: https://github.com/hyperium/tonic/pull/603

            Ok(req)
        });

    let svc = Service { metadata_client };
    let svc = GoldenPathExampleServiceServer::new(svc);

    println!("Server listening on {}", addr);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

/// A tower service struct for attaching timeout to request to other gRPC server
#[derive(Debug, Clone)]
struct Timeout<S> {
    inner: S,
    timeout: HeaderValue,
}

impl<S> Timeout<S> {
    pub fn new(inner: S, timeout_str: &'static str) -> Self {
        Timeout {
            inner,
            timeout: HeaderValue::from_static(timeout_str),
        }
    }
}

impl<S, ReqBody> TowerService<HttpRequest<ReqBody>> for Timeout<S>
where
    S: TowerService<HttpRequest<ReqBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: HttpRequest<ReqBody>) -> Self::Future {
        println!("call block req before: {:?}", req.headers());
        req.headers_mut()
            .insert("grpc-timeout", self.timeout.clone());
        println!("call block req: {:?}", req.headers());

        self.inner.call(req)
    }
}

/*
    Use command like this to send request to server:

    grpcurl -plaintext -import-path ./proto -proto trackinfo.proto \
        -d '{"trackId": "1WHzHtbCV4OoB0TLgG7eMD"}' \
        localhost:50052 spotify.goldenpathexamples.GoldenPathExampleService/TrackToString


    https://spotify.stackenterprise.co/questions/3381/6285#6285
    curl http://b62.spotify.net/spotify:track:0KKeRdzSUP5yEPLakW6CFE
    dig -t srv +short _spotify-$service._grpc.services.gew1.spotify.net | awk 'NR==1{printf("%s:%s", $4, $3)}'
    # change $service to metadata

    grpcurl -plaintext -max-time 5 -d '{"gid": "18c5dd0479ad446b9f1bbbcfea8ce59e", "country": "DK", "catalogue": "free"}' \
        -H "spotify-userinfo-bin: IgJ1c3IgY2RiM2EzOTA4NWEzNDg2MzkxZDA1NDIxMWUwZTUyOGM=" \
        gew1-metadataproxygrpc-b-m9mx.gew1.spotify.net.:8080 spotify.metadata.v1beta1.Metadata/GetTrack
*/
