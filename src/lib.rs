use std::io::Error;
#[macro_use]
extern crate log;

pub mod store;
pub mod server;
pub mod client;
pub mod common;
pub mod threadpool;

pub trait DBEngine {
    fn set(&mut self, key: String, value: String) -> Result<(), Error>;
    fn get(&mut self, key: String) -> Option<String>;
    fn remove();
}