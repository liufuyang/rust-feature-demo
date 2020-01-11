
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;

use async_std::io;
use async_std::task;
use serde::{Deserialize, Serialize};

fn main() -> io::Result<()> {
    task::block_on(async {
        let mut app = tide::new();

        let mutex: Mutex<HashMap<String, u32>> = {
            let mut m: HashMap<String, u32> = HashMap::new();
            m.insert("add1".to_owned(), 0);
            m.insert("add2".to_owned(), 0);
            m.insert("add3".to_owned(), 0);
            Mutex::new(m)
        };
        let arc = Arc::new(mutex);

        let _arc = Arc::clone(&arc);
        app.at("/show").get(move |_req: tide::Request<()>| {
            let map= Arc::clone(&_arc);
            async move {
                tide::Response::new(200).body_json(&*map).unwrap()
            }
        });

        let _arc = Arc::clone(&arc);
        app.at("/add").get(move |_req: tide::Request<()>| {
            let map= Arc::clone(&_arc);
            async move {
                let map = &mut *map.lock().unwrap();
                map.get_mut("add1").map(|v| *v += 1);
                map.get_mut("add2").map(|v| *v += 2);
                map.get_mut("add3").map(|v| *v += 3);

                tide::Response::new(200).body_json(&*map).unwrap()
            }
        });

        let _arc = Arc::clone(&arc);
        app.at("/reset").get(move |_req: tide::Request<()>| {
            let map= Arc::clone(&_arc);
            async move {
                let map = &mut *map.lock().unwrap();
                *map.get_mut("add1").unwrap() = 0;
                *map.get_mut("add2").unwrap() = 0;
                *map.get_mut("add3").unwrap() = 0;

                tide::Response::new(200).body_json(&*map).unwrap()
            }
        });

        app.listen("127.0.0.1:8000").await?;
        Ok(())
    })
}
