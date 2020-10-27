use std::io::{Error, BufReader, BufWriter, Write};
use std::net::TcpStream;
use crate::common::{Request, GetResponse};
use serde_json::Deserializer;
use serde::Deserialize;

pub struct DBClient{
    reader:BufReader<TcpStream>,
    writer:BufWriter<TcpStream>
}

impl DBClient {
    pub fn connect(addr:&str) -> Result<Self,Error> {
        let tcp_stream = TcpStream::connect(addr)?;
        let writer =  BufWriter::new(tcp_stream.try_clone()?);
        let reader =  BufReader::new(tcp_stream);

        Ok(Self{
            reader,
            writer
        })
    }

    pub fn set(&mut self, key:String, value:String) -> Result<(),Error>{
        serde_json::to_writer(&mut self.writer,&Request::Set {key,value})?;
        self.writer.flush()?;
       /* if let SetResponse::Err(e) = serde_json::from_reader(&mut self.reader).unwrap(){
            error!("err")
        }*/
        Ok(())
    }

    pub fn get(&mut self, key:String) -> Option<String> {
        serde_json::to_writer(&mut self.writer,&Request::Get {key}).unwrap();
        self.writer.flush().unwrap();
        let mut stream = Deserializer::from_reader(&mut self.reader);
        if let GetResponse::Ok(value) =  GetResponse::deserialize(&mut stream).unwrap(){
            return Some(value)
        }
        None
    }
}