use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufReader, BufWriter, Error, Read, Seek, SeekFrom, Write};
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::DBEngine;
use crate::store::Op::{Remove, Set};

pub struct KvStore {
    index: HashMap<String, Pos>,
    reader: BufferReader,
    writer: BufferWriter,
    // remove to compacted
    un_del: u64,
}

// one line data index
pub struct Pos {
    off: u64,
    size: u64,
}

impl Pos {
    pub fn new(off: u64, size: u64) -> Self {
        Pos {
            off,
            size,
        }
    }
}

pub struct BufferWriter {
    file_writer: BufWriter<File>,
    // 当前长度
    pos: u64,
}

impl BufferWriter {
    pub fn new(p: &PathBuf) -> Self {

        let mut file = OpenOptions::new().append(true)
            .read(true)
            .write(true)
            .create(true)
            .open(&p).unwrap();
        let pos = file.seek(SeekFrom::Start(0)).unwrap();
        Self {
            file_writer: BufWriter::new(file),
            pos,
        }
    }
}

impl Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let len = self.file_writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.file_writer.flush()
    }
}

#[derive(Serialize, Deserialize)]
pub enum Op {
    Set { key: String, value: String },
    Remove { key: String },
}

struct BufferReader {
    reader: BufReader<File>,
    // 文件最大长度
    pos: u64,
}

impl BufferReader {
    pub fn new(p: &PathBuf) -> Self {
        let mut file = File::open(p).unwrap();
        let pos = file.seek(SeekFrom::End(0)).unwrap();
        let reader = BufReader::new(file);
        Self {
            reader,
            pos,
        }
    }
}

impl Read for BufferReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl Seek for BufferReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        Ok(self.reader.seek(pos)?)
    }
}

impl KvStore {
    pub fn open(all_path: &'static str) -> Result<KvStore, Error> {
        let i = all_path.rfind("/").unwrap();

        let sub_path = all_path.split_at(i + 1);
        let path_name = sub_path.0;
        let file = PathBuf::from(all_path);
        create_dir_all(PathBuf::from(path_name))?;
        let writer = BufferWriter::new(&file);
        let reader = BufferReader::new(&file);
        let mut kv_store = Self {
            index: Default::default(),
            reader,
            writer,
            un_del: 0,
        };
        kv_store.recover();
        Ok(kv_store)
    }

    pub fn recover(&mut self) {
        let reader = &mut self.reader;

        reader.seek(SeekFrom::Start(0)).unwrap();
        let reader = reader.take(reader.pos);
        let mut start = 0;
        let mut stream = Deserializer::from_reader(reader).into_iter::<Op>();
        while let Some(op) = stream.next() {
            let off = stream.byte_offset();
            match op.unwrap() {
                Set { key, .. } => {
                    self.index.insert(key, Pos::new(start, off as u64 - start as u64));
                },
                Remove { key } => {
                    self.index.remove(&key);
                }
                _ => {}
            }
            start = off as u64;
        }
    }
}

impl DBEngine for KvStore {
    fn set(&mut self, key: String, value: String) -> Result<(), Error> {
        let op = Set { key, value };
        let off = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &op)?;
        self.writer.flush()?;
        if let Set { key, .. } = op {
            if let Some(old) = self.index.insert(key, Pos {
                off,
                size: self.writer.pos - off,
            }) {
                self.un_del += old.size;
            }
        }
        if self.un_del > 1000 {
            // todo compact
        }
        Ok(())
    }

    fn get(&mut self, key: String) -> Option<String> {
        if let Some(pos) = self.index.get(&key) {
            let reader = &mut self.reader;
            reader.seek(SeekFrom::Start(pos.off)).unwrap();
            let reader = reader.take(pos.size);
            if let Set { value, .. } = serde_json::from_reader(reader).unwrap() {
                return Some(value)
            }
        }
        None
    }


    fn remove() {
        unimplemented!()
    }
}