use std::io;

pub mod store;

pub trait DBEngine {
    fn set();
    fn get();
    fn remove();
}
pub type Result<T> = std::result::Result<T, KvsError>;

/// Error type for kvs.
#[derive(Debug)]
pub enum KvsError {
    /// IO error.
    Io(io::Error),
    /// Serialization or deserialization error.
    Serde(serde_json::Error),
    /// Removing non-existent key error.
    KeyNotFound,
    /// Unexpected command type error.
    /// It indicated a corrupted log or a program bug.
    UnexpectedCommandType,
}