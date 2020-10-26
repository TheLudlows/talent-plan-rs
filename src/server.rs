use std::io::{BufReader, BufWriter, Error, Write};
use std::net::{TcpListener, TcpStream};

use serde_json::Deserializer;

use crate::common::*;
use crate::DBEngine;
use crate::store::KvStore;

pub struct DBServer;

impl DBServer {
    pub fn start(data_path: &'static str, addr: &str) {
        let mut db = KvStore::open(data_path).unwrap();
        let listener = TcpListener::bind(addr).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    DBServer::serve(&mut db,stream);
                }
                Err(_) => {}
            }
        }
    }

    fn serve(engine: &mut KvStore, tcp: TcpStream) -> Result<(), Error> {
        let peer_addr = tcp.peer_addr().unwrap();
        let reader = BufReader::new(&tcp);
        let mut writer = BufWriter::new(&tcp);
        let req_reader = Deserializer::from_reader(reader).into_iter::<Request>();

        for req in req_reader {
            let req = req.unwrap();
            debug!("Receive request from {}: {:?}", peer_addr, req);
            println!("req:{:?}",req);
            match req {
                Request::Get { key } => match engine.get(key) {
                    Some(value) => {
                        println!("find vale {}",value);
                        serde_json::to_writer(&mut writer, &GetResponse::Ok(value))?;
                        writer.flush()?;
                    }
                    _ => {
                        serde_json::to_writer(&mut writer, &GetResponse::Err("null".to_string()))?;
                        writer.flush()?;
                    },
                },
                Request::Set { key, value } => match engine.set(key, value) {
                    Ok(_) => {},
                    _ => {
                        serde_json::to_writer(&mut writer, &SetResponse::Err("err".to_string()))?;
                        writer.flush()?;
                    },
                },
                _ => {}
            }
        }
        println!("flush over");
        Ok(())
    }
}



