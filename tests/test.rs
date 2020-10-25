use rs_kv::store::KvStore;
use rs_kv::DBEngine;

#[test]
fn test() {
    let mut store = KvStore::open("/home/four/kvs/data".to_string()).unwrap();
    store.set("key1".to_string(),"valuye1".to_string());
}