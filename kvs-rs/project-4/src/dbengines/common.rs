
use serde::{Deserialize, Serialize};
/// to write in file
#[derive(Serialize, Deserialize)]
pub enum Op {
    Set { key: String, value: String },
    Remove { key: String },
}

/// one line data index
#[derive(Debug)]
pub struct Pos {
    pub id: u32,
    pub off: u64,
    pub size: u64,
}

impl Pos {
    pub fn new(id: u32, off: u64, size: u64) -> Self {
        Pos { id, off, size }
    }
}