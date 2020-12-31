use std::env::current_dir;

use talent_plan_rs::KvStore;

#[test]
fn test() {
    for j in 0..2 {
        let mut db = KvStore::open(current_dir().unwrap().as_path()).unwrap();
        for i in 0..1000 {
            db.set(i.to_string(), i.to_string());
            assert_eq!(db.get(i.to_string()).unwrap().unwrap(), i.to_string());
        }
    }
}