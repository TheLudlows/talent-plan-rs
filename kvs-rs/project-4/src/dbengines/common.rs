use serde::{Deserialize, Serialize};

/// to write in file
#[derive(Serialize, Deserialize)]
pub enum Op {
    Set { key: String, value: String },
    Remove { key: String },
}

/// one line data index, TODO to in u64 ,key v spit to store
#[derive(Debug)]
pub struct Pos {
    pub id: u16,
    pub off: u32,
    pub size: u16,
}

impl Pos {
    pub fn new(id: u16, off: u32, size: u16) -> Self {
        Pos { id, off, size }
    }
}
