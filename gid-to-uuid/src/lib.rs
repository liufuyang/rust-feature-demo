#![allow(dead_code)]
#[macro_use]
extern crate serde_derive;

use bytes::buf::BufExt as _;
use hyper::Client;

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize, Debug)]
struct UrlInfo {
    #[serde(alias = "_hexId")]
    hex_id: String,
}

async fn to_uuid(gid: &str) -> Result<String> {
    let client = Client::new();

    let url = format!("http://b62.spotify.net/spotify:track:{}", gid);
    let url = url.parse::<hyper::Uri>().unwrap();
    let res = client.get(url).await?;

    // asynchronously aggregate the chunks of the body
    let body = hyper::body::aggregate(res).await?;

    // try to parse as json with serde_json
    let url_info: UrlInfo = serde_json::from_reader(body.reader())?;

    println!("{:?}", url_info);

    Ok(url_info.hex_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[tokio::test]
    async fn my_test() {
        let hex = to_uuid("0KKeRdzSUP5yEPLakW6CFE").await;
        assert_eq!("18c5dd0479ad446b9f1bbbcfea8ce59e", hex.unwrap());
    }
}
