use std::path::Path;

use sled::Db;

use crate::{KvsEngine, Result};
use crate::error::KvsError::KeyNotFound;

#[derive(Clone, Debug)]
pub struct SledKvsEngine(Db);

impl SledKvsEngine {
    pub fn new(db: Db) -> Self {
        Self(db)
    }

    pub fn open(p: &Path) -> Result<Self> {
        Ok(Self(sled::open(p)?))
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&self, key: String, value: String) -> Result<()> {
        let db = &self.0;
        db.insert(key, value.into_bytes())?;
        //db.flush()?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let db = &self.0;
        Ok(db.get(key)?.map_or(None, |v| Some(String::from_utf8(v.to_vec()).unwrap())))
    }

    fn remove(&self, key: String) -> Result<()> {
        self.0.remove(key)?.ok_or(KeyNotFound)?;
        self.0.flush()?;
        Ok(())
    }
}
