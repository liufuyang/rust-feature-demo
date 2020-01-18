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
        m.insert("add1".to_string(), 0);
        m.insert("add2".to_string(), 0);
        m.insert("add3".to_string(), 0);
        Mutex::new(m)
    };
}

fn main() -> io::Result<()> {
    task::block_on(async {
        let mut app = tide::new();

        app.at("/show").get(|_req: tide::Request<()>| {
            async {
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

        app.listen("127.0.0.1:8081").await?;
        Ok(())
    })
}
// ab -n 500 -c 50 127.0.0.1/add
// curl -X POST -d '{"name": "Rulle"}' 127.0.0.1:8081/submit

#[cfg(test)]
mod tests {
    use threadpool::ThreadPool;

    use super::*;
    use std::{thread, time};
    use std::sync::Arc;

    #[test]
    fn test_add() {
        let map = HashMap::new();
        let sum = parallel_sum_to_10000(map);
        assert_eq!(10000, sum);
    }

    fn parallel_sum_to_10000(mut map: HashMap<String, u32>) -> u32 {
        map.insert("test".to_string(), 0);
        let pool = ThreadPool::new(10);

        for _ in 0..100 {
            pool.execute(|| {
                for _ in 0..100 {
                    *map.get_mut("test").unwrap() += 1;
                }
            }
            );
        }

        return map.get("test").unwrap().clone();
    }

    // A working example:

//    fn parallel_sum_to_10000(mut map: HashMap<String, u32>) -> u32 {
//        map.insert("test".to_string(), 0);
//        let pool = ThreadPool::new(10);
//
//        let arc = Arc::new(Mutex::new(map));
//        for _ in 0..100 {
//            let map = arc.clone();
//            pool.execute(move || {
//                for _ in 0..100 {
//                    *map.lock().unwrap().get_mut("test").unwrap() += 1;
//                    // map.lock().unwrap().get_mut("test").map(|v| *v +=1);
//                }
//            }
//            );
//        }
//        // simply wait for all task to finish
//        thread::sleep(time::Duration::from_millis(100));
//        return arc.lock().unwrap().get("test").unwrap().clone();
//    }
}