use rs_kv::store::KvStore;
use rs_kv::DBEngine;

#[test]
fn test() {
    let store = KvStore::open("/home/four/kvs/data".to_string());
}