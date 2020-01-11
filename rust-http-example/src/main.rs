#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::sync::Mutex;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;


lazy_static! {
    static ref MAP: Mutex<HashMap<String, u32>> = {
        let mut m: HashMap<String, u32> = HashMap::new();
        m.insert("add1".to_owned(), 0);
        m.insert("add2".to_owned(), 0);
        m.insert("add3".to_owned(), 0);
        Mutex::new(m)
    };
}

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn demo(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/show") => Ok(Response::new(Body::from(
            serde_json::to_string(&*MAP).unwrap()
        ))),
        (&Method::GET, "/add") => {
            let map = &mut *MAP.lock().unwrap();
            map.get_mut("add1").map(|v| *v += 1);
            map.get_mut("add2").map(|v| *v += 2);
            map.get_mut("add3").map(|v| *v += 3);

            Ok(Response::new(Body::from(
                serde_json::to_string(map).unwrap()
            )))
        }
        (&Method::GET, "/reset") => {
            let map = &mut *MAP.lock().unwrap();
            *map.get_mut("add1").unwrap() = 0;
            *map.get_mut("add2").unwrap() = 0;
            *map.get_mut("add3").unwrap() = 0;

            Ok(Response::new(Body::from(
                serde_json::to_string(map).unwrap()
            )))
        }
        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = ([0, 0, 0, 0], 3000).into();

    let service = make_service_fn(|_| async { Ok::<_, GenericError>(service_fn(demo)) });

    let server = Server::bind(&addr).serve(service);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
