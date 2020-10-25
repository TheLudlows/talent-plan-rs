use std::io;
use std::io::Error;

pub mod store;

pub trait DBEngine {
    fn set(&mut self,key:String,value:String)->Result<(),Error>;
    fn get();
    fn remove();
}