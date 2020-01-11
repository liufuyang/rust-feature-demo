#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::sync::Mutex;

use async_std::io;
use async_std::task;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Cat {
    name: String,
}

lazy_static! {
    static ref MAP: Mutex<HashMap<String, u32>> = {
        let mut m: HashMap<String, u32> = HashMap::new();
        m.insert("add1".to_owned(), 0);
        m.insert("add2".to_owned(), 0);
        m.insert("add3".to_owned(), 0);
        Mutex::new(m)
    };
}

fn main() -> io::Result<()> {
    task::block_on(async {
        let mut app = tide::new();

        app.at("/show").get(|_req: tide::Request<()>| {
            async move {
                tide::Response::new(200).body_json(&*MAP).unwrap()
            }
        });

        app.at("/add").get(|_req: tide::Request<()>| {
            async {
                let map = &mut *MAP.lock().unwrap();
                map.get_mut("add1").map(|v| *v += 1);
                map.get_mut("add2").map(|v| *v += 2);
                map.get_mut("add3").map(|v| *v += 3);

                tide::Response::new(200).body_json(map).unwrap()
            }
        });

        app.at("/reset").get(|_req: tide::Request<()>| {
            async {
                let map = &mut *MAP.lock().unwrap();
                *map.get_mut("add1").unwrap() = 0;
                *map.get_mut("add2").unwrap() = 0;
                *map.get_mut("add3").unwrap() = 0;

                tide::Response::new(200).body_json(map).unwrap()
            }
        });

        app.at("/submit").post(|mut req: tide::Request<()>| {
            async move {
                let cat: Cat = req.body_json().await.unwrap();
                println!("cat name: {}", cat.name);

                let cat = Cat {
                    name: "chashu".into(),
                };
                tide::Response::new(200).body_json(&cat).unwrap()
            }
        });

        app.listen("127.0.0.1:8000").await?;
        Ok(())
    })
}
