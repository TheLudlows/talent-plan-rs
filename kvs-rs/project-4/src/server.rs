use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use serde_json::Deserializer;

use crate::KvsEngine;
use crate::msg::{Request, Response};
use crate::Result;

pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine> KvsServer<E> {
    pub fn new(engine: E) -> Self {
        KvsServer { engine }
    }

    pub fn run(mut self, addr: impl ToSocketAddrs) -> Result<()> {
        let tcp_listener = TcpListener::bind(addr)?;
        for stream in tcp_listener.incoming() {
            match stream {
                Ok(s) => {
                    self.server(s)?;
                }
                Err(e) => eprintln!("connect err,{}", e),
            };
        }
        Ok(())
    }

    fn server(&mut self, stream: TcpStream) -> Result<()> {
        let addr = stream.local_addr()?;
        println!("connect from addr {}", addr);

        let reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);
        let reqs = Deserializer::from_reader(reader).into_iter::<Request>();

        for req in reqs {
            let req = req?;
            let resp = match req {
                Request::Get { key } => {
                    match self.engine.get(key) {
                        Ok(res) => Response::Get(res),
                        Err(e) => Response::Err(format!("{:?}", e))
                    }
                }
                Request::Set { key, value } => {
                    match self.engine.set(key, value.clone()) {
                        Ok(()) => Response::Set(value),
                        Err(e) => Response::Err(format!("{:?}", e))
                    }
                }
                Request::Remove { key } => {
                    match self.engine.remove(key) {
                        Ok(()) => Response::Remove,
                        Err(e) => Response::Err(format!("{:?}", e))
                    }
                }
            };
            //println!("server process result {:?}",resp);
            serde_json::to_writer(&mut writer, &resp)?;
            writer.flush()?;
        }
        Ok(())
    }
}