use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize,Deserialize,Debug)]
struct Foo{
    id:usize,
    name:String
}

#[test]
fn test() {
    let f = Foo{
        id:100,
        name:"four".to_string()
    };
    let s = serde_json::to_string(&f);
    println!("{:?}",s);
    let f:Result<Foo> = serde_json::from_str::<Foo>(s.unwrap().as_str());
    println!("{:?}",f.unwrap())
}

