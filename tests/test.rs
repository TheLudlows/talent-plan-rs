use rs_kv::store::KvStore;
use rs_kv::DBEngine;
use std::env::current_dir;
use rs_kv::server::DBServer;

#[test]
fn test() {
    let mut store = KvStore::open("/Users/liuchao56/data/1.log").unwrap();
    store.set("key3".to_string(),"valuye4".to_string());
    let v = store.get("key2".to_string());
    println!("{:?}",v);
}

#[test]
fn test_string() {
    let s = String::from("a/sda/ad.log");
    let i = s.rfind("/").unwrap();
    let r = s.split_at(i);

    println!("{:?}",r);
}

#[test]
fn test_server() {
    DBServer::start("/Users/liuchao56/data/2.log","localhost:8888");
}
