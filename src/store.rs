use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Error, Seek, SeekFrom, Write};
use std::fs::{File, create_dir_all, OpenOptions};
use std::path::{PathBuf, Path};
use serde::{Deserialize, Serialize};
use crate::{DBEngine};
use crate::store::Op::Set;

pub struct KvStore{
    index:HashMap<String,Pos>,
    reader_index:HashMap<u32,BufReaderWithPos>,
    writer:Writer,
    path:PathBuf,
    cur_file:u32,
    // remove to compacted
    un_del:u64
}
// one line data index
pub struct Pos{
    file_id:u32,
    off:u64,
    size:u64
}
pub struct Writer{
    file_writer:BufWriter<File>,
    pos:u64
}
impl Writer {
    pub fn new(p:&PathBuf) -> Self{
        let mut file = OpenOptions::new().append(true)
            .read(true)
            .write(true)
            .create(true)
            .open(&p).unwrap();
        let pos = file.seek(SeekFrom::Start(0)).unwrap();
        Self{
            file_writer:BufWriter::new(file),
            pos
        }
    }
}
impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> Result<usize,Error> {
        let len = self.file_writer.write(buf)?;
        self.pos+=len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> Result<(),Error> {
        self.file_writer.flush()
    }
}
#[derive(Serialize,Deserialize)]
pub enum  Op{
    Set{key:String,value:String},
    Remove{key:String}
}

struct BufReaderWithPos {
    reader: BufReader<File>,
    pos: u64,
}
impl BufReaderWithPos {
    pub fn new(p:&PathBuf) -> Self{
        let mut file = File::open(p).unwrap();
        let pos = file.seek(SeekFrom::Start(0)).unwrap();
        let reader = BufReader::new(file);
        Self{
            reader,pos
        }
    }
}

impl KvStore {
    pub fn open(path:String) -> Result<KvStore, Error> {
        let p = PathBuf::from(path);
        create_dir_all(&p)?;
        let cur = &p.as_path().join("1");
        let writer = Writer::new(cur);
        let reader = BufReaderWithPos::new(cur);
        let mut  reader_index:HashMap<u32,BufReaderWithPos> = HashMap::new();
        reader_index.insert(1,reader);

        Ok(Self {
            index: Default::default(),
            reader_index,
            writer,
            path: p,
            cur_file: 1,
            un_del: 0
        })
    }
}

impl DBEngine for KvStore{
    fn set(&mut self,key:String,value:String) -> Result<(),Error>{
        let op = Set {key,value};
        let off = self.writer.pos;
        serde_json::to_writer(&mut self.writer,&op);
        self.writer.flush()?;
        if let Set{key,..} = op {
            self.index.insert(key,Pos{
                file_id: self.cur_file,
                off,
                size:self.writer.pos - off
            });
        }
        Ok(())
    }

    fn get() {
        unimplemented!()
    }

    fn remove() {
        unimplemented!()
    }
}