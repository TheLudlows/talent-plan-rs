use std::path::Path;
use std::sync::Arc;
use std::thread;

use kvs::{KvsEngine, KvStore, SledKvsEngine};

#[test]
fn sled() {
    let path = Path::new("/tmp/");
    let mut db = SledKvsEngine::open(path).unwrap();
    set_get(&mut db);
}

#[test]
fn kvs() {
    let path = Path::new("/tmp/");
    let mut db = KvStore::open(path).unwrap();
    set_get(&mut db);
    println!("db size is {}", db.index.len())
}

fn set_get<DB: KvsEngine>(db: &mut DB) {
    for i in 1..1 << 12 {
        db.set(format!("{}", i), format!("{}", i)).unwrap();
    }
}

#[test]
fn bench_write() {
    let mut joins = vec![];
    let path = Path::new("/tmp/");

    let db = KvStore::open(path).unwrap();
    let step = 10000;
    for i in 0..3 {
        let tmp_db = db.clone();
        let j = thread::spawn(move || {
            for i in i * step - 1..(i+1) * step {
                tmp_db.set(i.to_string(), i.to_string()).unwrap();
            }
        });
        joins.push(j);
    }
    for join in joins {
        join.join().unwrap();
    }
}