use std::io;

pub mod store;

pub trait DBEngine {
    fn set();
    fn get();
    fn remove();
}