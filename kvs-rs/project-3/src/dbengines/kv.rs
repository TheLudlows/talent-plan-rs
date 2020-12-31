use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use serde_json::Deserializer;

use crate::{KvsEngine, Result};
use crate::dbengines::kv::Op::{Remove, Set};
use crate::error::KvsError::KeyNotFound;
use crate::utils::{del_file, format_path, ls_logs};
use crate::dbengines::common::*;
use crate::dbengines::common::Pos;

/// set 1.add index 2.append log
/// get 1.read index 2.read file
/// remove 1.remove index 2.append log 3.add unCompact count 4.if ness compact 5.add cur gen

const MAX_UN_COMPACT: u64 = 1024 * 1024;

#[derive(Debug)]
pub struct KvStore {
    path: PathBuf,
    pub index: HashMap<String, Pos>,
    readers: HashMap<u32, BufferReader>,
    writer: BufferWriter,
    un_compact: u64,
    cur_file_id: u32,
}

#[derive(Debug)]
pub struct BufferWriter {
    file_writer: BufWriter<File>,
    file_pos: u64
}

impl BufferWriter {
    pub fn new(p: PathBuf) -> Result<Self> {
        let mut file = OpenOptions::new().append(true)
            .read(true)
            .write(true)
            .create(true)
            .open(&p)?;
        let pos = file.seek(SeekFrom::Start(0)).unwrap();
        Ok(Self {
            file_writer: BufWriter::new(file),
            file_pos: pos,
        })
    }
}

impl Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.file_writer.write(buf)?;
        self.file_pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file_writer.flush()?;
        Ok(())
    }
}

#[derive(Debug)]
struct BufferReader {
    reader: BufReader<File>,
}

impl BufferReader {
    pub fn new(p: PathBuf) -> Result<Self> {
        let file = File::open(p)?;
        let reader = BufReader::new(file);
        Ok(Self {
            reader,
        })
    }
}

impl Read for BufferReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        Ok(len)
    }
}

impl Seek for BufferReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.reader.seek(pos)
    }
}

impl KvStore {
    pub fn open(path: &Path) -> Result<KvStore> {
        create_dir_all(path)?;
        let mut index = HashMap::new();
        let mut readers = HashMap::new();
        let log_ids = ls_logs(path);
        //println!("{:?}",log_ids);
        let mut un_compact = 0;
        for id in log_ids.iter() {
            un_compact += Self::load_file(path, &mut index, &mut readers, *id)?;
        }
        let cur_file_id = log_ids.last().unwrap_or(&0) + 1;
        let writer = BufferWriter::new(format_path(path, cur_file_id))?;
        readers.insert(cur_file_id, BufferReader::new(format_path(path, cur_file_id))?);
        Ok(Self {
            path: path.to_path_buf(),
            index,
            readers,
            writer,
            un_compact,
            cur_file_id,
        })
    }
    fn compact(&mut self) -> Result<()> {
        self.cur_file_id += 1;
        let path = self.path.as_path();
        let mut new_writer = BufferWriter::new(format_path(path, self.cur_file_id))?;
        let mut new_index = HashMap::new();
        for (key, pos) in self.index.iter() {
            //println!("{:?}",pos);
            let reader = self.readers.get_mut(&pos.id).expect("no exist");
            reader.seek(SeekFrom::Start(pos.off))?;
            let mut reader = reader.take(pos.size);
            if let Set { value, .. } = serde_json::from_reader(&mut reader)? {
                let op = Set { key: key.clone(), value };
                let off = new_writer.file_pos;
                serde_json::to_writer(&mut new_writer, &op)?;
                new_writer.flush()?;
                new_index.insert(key.clone(), Pos { id: self.cur_file_id, off, size: new_writer.file_pos - off });
            }
        }
        self.readers.keys().for_each(|id| {
            del_file(format_path(self.path.as_path(), *id)).unwrap();
        });
        self.readers.clear();
        self.readers.insert(self.cur_file_id, BufferReader::new(format_path(path, self.cur_file_id))?);

        self.index = new_index;
        self.writer = new_writer;
        Ok(())
    }

    /// load file to index
    /// return the un compact size
    fn load_file(path: &Path, index: &mut HashMap<String, Pos>, readers: &mut HashMap<u32, BufferReader>, id: u32) -> Result<u64> {
        let mut reader = BufferReader::new(format_path(path, id))?;
        reader.seek(SeekFrom::Start(0))?;
        let mut start = 0;
        let mut un_compact = 0;
        let mut stream = Deserializer::from_reader(&mut reader).into_iter::<Op>();
        while let Some(op) = stream.next() {
            let off = stream.byte_offset() as u64;
            match op? {
                Set { key, .. } => {
                    index.insert(key, Pos::new(id, start, off - start));
                }
                Remove { key } => {
                    index.remove(&key);
                    un_compact += off - start;
                }
            }
            start = off;
        }
        //println!("load file is {:?}",index);
        readers.insert(id, reader);
        Ok(un_compact)
    }
}

impl KvsEngine for KvStore {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let op = Set { key: key.clone(), value };
        let off = self.writer.file_pos;
        serde_json::to_writer(&mut self.writer, &op)?;
        self.writer.flush()?;
        if let Some(old) = self.index.insert(key, Pos { id: self.cur_file_id, off, size: self.writer.file_pos - off }) {
            self.un_compact += old.size;
        }
        if self.un_compact > MAX_UN_COMPACT {
            self.compact()?;
        }
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(pos) = self.index.get(&key) {
            let reader = self.readers.get_mut(&pos.id).expect("no exist");
            reader.seek(SeekFrom::Start(pos.off))?;
            let mut reader = reader.take(pos.size);
            if let Set { value, .. } = serde_json::from_reader(&mut reader)? {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        //println!("{:?}",&self.index);
        if let Some(old) = self.index.remove(&key) {
            // rm index
            self.un_compact += old.size;
            // append rm log
            let op = Remove { key };
            serde_json::to_writer(&mut self.writer, &op)?;
            self.writer.flush()?;
            if self.un_compact > MAX_UN_COMPACT {
                self.compact()?;
            }
            //println!("{:?}",&self.index);
            Ok(())
        } else {
            Err(KeyNotFound)
        }
    }
}