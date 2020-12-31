use crate::Result;

pub use self::kv::KvStore;
pub use self::sled::SledKvsEngine;

mod kv;
mod sled;
mod common;


pub trait KvsEngine {
    fn set(&self, key: String, value: String) -> Result<()>;
    fn get(&self, key: String) -> Result<Option<String>>;
    fn remove(&self, key: String) -> Result<()>;
}

