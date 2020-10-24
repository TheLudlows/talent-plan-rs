use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::fs::File;
use std::path::PathBuf;
use crate::{DBEngine,Result};

pub struct KvStore{
    index:HashMap<String,Pos>,
    reader_index:HashMap<u32,BufReader<File>>,
    writer:BufWriter<File>,
    path:PathBuf,
    cur_file:u32,
    // remove to compacted
    un_del:u64
}
pub struct Pos{
    file_id:u32,
    off:u64,
    size:u64
}

impl KvStore {
    pub fn open() -> Result<KvStore> {
        unimplemented!()
    }
}

impl DBEngine for KvStore{

    fn set() {
        unimplemented!()
    }

    fn get() {
        unimplemented!()
    }

    fn remove() {
        unimplemented!()
    }
}