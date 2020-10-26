use rs_kv::client::DBClient;

fn main() {
    let mut client = DBClient::connect("localhost:8888").unwrap();
    client.set("1".to_string(),"2".to_string());
    let res = client.get("3".to_string());
    println!("{}",res.unwrap())
}