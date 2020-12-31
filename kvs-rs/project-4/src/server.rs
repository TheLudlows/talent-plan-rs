use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use serde_json::Deserializer;

use crate::KvsEngine;
use crate::msg::{Request, Response};
use crate::Result;
use crate::thread_pool::ThreadPool;

pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    engine: E,
    pool: P,
}

impl<E: KvsEngine  + Clone, P: ThreadPool> KvsServer<E, P> {
    pub fn new(engine: E, pool: P) -> Self {
        KvsServer {
            engine,
            pool,
        }
    }

    pub fn run(self, addr: impl ToSocketAddrs) -> Result<()> {
        let tcp_listener = TcpListener::bind(addr)?;
        for stream in tcp_listener.incoming() {
            let e = self.engine.clone();
            self.pool.spawn(move ||
                match stream {
                    Ok(s) => {
                        server(e, s).unwrap();
                    }
                    Err(e) => eprintln!("connect err,{}", e),
                });
        }
        Ok(())
    }
}

fn  server <E: KvsEngine> (engine: E, stream: TcpStream) -> Result<()> {
    let addr = stream.local_addr()?;
    println!("connect from addr {}", addr);

    let reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let reqs = Deserializer::from_reader(reader).into_iter::<Request>();

    for req in reqs {
        let req = req?;
        let resp = match req {
            Request::Get { key } => {
                match engine.get(key) {
                    Ok(res) => Response::Get(res),
                    Err(e) => Response::Err(format!("{:?}", e))
                }
            }
            Request::Set { key, value } => {
                match engine.set(key, value.clone()) {
                    Ok(()) => Response::Set(value),
                    Err(e) => Response::Err(format!("{:?}", e))
                }
            }
            Request::Remove { key } => {
                match engine.remove(key) {
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