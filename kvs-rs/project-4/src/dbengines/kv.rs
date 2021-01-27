use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::atomic::Ordering::{Relaxed, SeqCst};

use crossbeam_skiplist::SkipMap;
use failure::_core::sync::atomic::AtomicU32;
use serde_json::Deserializer;

use crate::{KvsEngine, Result};
use crate::dbengines::common::*;
use crate::dbengines::common::Pos;
use crate::dbengines::kv::Op::{Remove, Set};
use crate::error::KvsError::KeyNotFound;
use crate::utils::{del_file, format_path, ls_logs};

/// set 1.add index 2.append log
/// get 1.read index 2.read file
/// remove 1.remove index 2.append log 3.add unCompact count 4.if ness compact 5.add cur gen

const MAX_UN_COMPACT: u64 = 1024 * 1024;

#[derive(Debug)]
pub struct KvStore {
    path: Arc<PathBuf>,
    // pub for debug
    pub index: Arc<SkipMap<String, Pos>>,
    reader: RefCell<HashMap<u32, BufferReader>>,
    writer: Arc<Mutex<RefCell<BufferWriter>>>,
    cur_file_id: Arc<AtomicU32>,
    un_compact_size: Arc<AtomicU64>,
}

impl Clone for KvStore {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            index: self.index.clone(),
            reader: RefCell::new(HashMap::new()),
            writer: self.writer.clone(),
            cur_file_id: self.cur_file_id.clone(),
            un_compact_size: self.un_compact_size.clone(),
        }
    }
}

#[derive(Debug)]
pub struct BufferWriter {
    file_writer: BufWriter<File>,
    file_pos: u64,
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
        let index = SkipMap::new();
        let mut readers = HashMap::new();
        let log_ids = ls_logs(path);
        //println!("{:?}",log_ids);
        let mut un_compact = 0;
        for id in log_ids.iter() {
            un_compact += Self::load_file(path, &index, &mut readers, *id)?;
        }
        let cur_file_id = log_ids.last().unwrap_or(&0) + 1;
        let writer = BufferWriter::new(format_path(path, cur_file_id))?;
        readers.insert(cur_file_id, BufferReader::new(format_path(path, cur_file_id))?);
        Ok(Self {
            path: Arc::new(path.to_path_buf()),
            index: Arc::new(index),
            reader: RefCell::new(readers),
            writer: Arc::new(Mutex::new(RefCell::new(writer))),
            cur_file_id: Arc::new(AtomicU32::new(cur_file_id)),
            un_compact_size: Arc::new(AtomicU64::new(un_compact)),
        })
    }
    fn compact(&self) -> Result<BufferWriter> {
        let cur_file_id = 1 + self.cur_file_id.load(Relaxed);
        let path = self.path.as_path();
        let mut new_writer = BufferWriter::new(format_path(path, cur_file_id))?;
        for entry in self.index.iter() {
            //println!("{:?}",pos);
            let (key, pos) = (entry.key(), entry.value());
            let mut reader_map = self.reader.borrow_mut();
            let reader = reader_map.entry(pos.id).or_insert(BufferReader::new(format_path(&self.path, pos.id))?);
            reader.seek(SeekFrom::Start(pos.off))?;
            let mut reader = reader.take(pos.size);
            if let Set { value, .. } = serde_json::from_reader(&mut reader)? {
                let op = Set { key: key.clone(), value };
                let off = new_writer.file_pos;
                serde_json::to_writer(&mut new_writer, &op)?;
                self.index.insert(key.clone(), Pos { id: cur_file_id, off, size: new_writer.file_pos - off });
            }
        }
        new_writer.flush()?;
        // del old file
        let log_ids = ls_logs(path);
        for id in log_ids.iter() {
            if *id < cur_file_id {
                del_file(format_path(&self.path, *id))?;
            }
        }
        self.cur_file_id.store(cur_file_id, Relaxed);
        Ok(new_writer)
    }

    /// load file to index
    /// return the un compact size
    fn load_file(path: &Path, index: &SkipMap<String, Pos>, readers: &mut HashMap<u32, BufferReader>, id: u32) -> Result<u64> {
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
    fn set(&self, key: String, value: String) -> Result<()> {
        let op = Set { key: key.clone(), value };
        // lock
        let writer_ref = self.writer.lock().unwrap();
        let mut writer = writer_ref.borrow_mut();

        let off = writer.file_pos;
        serde_json::to_writer(&mut *writer, &op)?;
        writer.flush()?;
        if let Some(old) = self.index.get(&key) {
            self.un_compact_size.fetch_add(old.value().size, SeqCst);
        }
        self.index.insert(key, Pos { id: self.cur_file_id.load(Ordering::Relaxed), off, size: writer.file_pos - off });
        if self.un_compact_size.load(Ordering::Relaxed) > MAX_UN_COMPACT {
            *writer = self.compact()?;
        }
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(entry) = self.index.get(&key) {
            let pos = entry.value();
            let mut reader_map = self.reader.borrow_mut();
            let reader = reader_map.entry(pos.id).or_insert(BufferReader::new(format_path(&self.path, pos.id))?);
            reader.seek(SeekFrom::Start(pos.off))?;
            let mut reader = reader.take(pos.size);
            if let Set { value, .. } = serde_json::from_reader(&mut reader)? {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    fn remove(&self, key: String) -> Result<()> {
        //println!("{:?}",&self.index);
        let writer_ref = self.writer.lock().unwrap();
        let mut writer = writer_ref.borrow_mut();
        if let Some(entry) = self.index.remove(&key) {
            // rm index
            self.un_compact_size.fetch_add(entry.value().size, Ordering::SeqCst);
            // append rm log
            let op = Remove { key };
            serde_json::to_writer(&mut *writer, &op)?;
            writer.flush()?;
            if self.un_compact_size.load(Ordering::Relaxed) > MAX_UN_COMPACT {
                *writer = self.compact()?;
            }

            //println!("{:?}",&self.index);
            Ok(())
        } else {
            Err(KeyNotFound)
        }
    }
}