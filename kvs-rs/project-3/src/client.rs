use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream, ToSocketAddrs};

use serde::Deserialize;
use serde_json::Deserializer;

use crate::error::KvsError;
use crate::msg::Request::{Get, Set, Remove};
use crate::msg::Response;
use crate::Result;

pub struct KvsClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    pub fn connect(add: impl ToSocketAddrs) -> Result<KvsClient> {
        let reader = TcpStream::connect(add)?;
        let writer = reader.try_clone()?;
        Ok(Self {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
        })
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Set { key, value })?;
        self.writer.flush()?;

        let mut de = Deserializer::from_reader(&mut self.reader);
        let r = Response::deserialize(&mut de)?;
        match r {
            Response::Err(msg) => {
                Err(KvsError::StringError(msg))
            }
            Response::Set(_) => {
                Ok(())
            }
            _ => { Err(KvsError::UnexpectedCommandType) }
        }
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        serde_json::to_writer(&mut self.writer, &Get { key })?;
        self.writer.flush()?;

        let mut de = Deserializer::from_reader(&mut self.reader);
        let r = Response::deserialize(&mut de)?;

        match r {
            Response::Get(op) => {
                Ok(op)
            }
            Response::Err(msg) => {
                Err(KvsError::StringError(msg))
            }
            _ => { Err(KvsError::UnexpectedCommandType) }
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Remove { key })?;
        self.writer.flush()?;

        let mut de = Deserializer::from_reader(&mut self.reader);
        let r = Response::deserialize(&mut de)?;
        match r {
            Response::Remove => {
                Ok(())
            }
            Response::Err(msg) => {
                Err(KvsError::StringError(msg))
            }
            _ => { Err(KvsError::UnexpectedCommandType) }
        }
    }
}