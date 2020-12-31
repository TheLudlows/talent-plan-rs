use std::path::Path;

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