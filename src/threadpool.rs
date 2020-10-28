use std::thread;

use crossbeam::channel::{self, Sender};

trait ThreadPool {
    fn new(size: usize) -> Self;
    fn spawn<F: FnOnce() + Send + 'static>(&self, f: F);

    fn spawn1(f:impl FnOnce() + Send);
}

pub struct MyThreadPool {
    sender: Sender<Box<dyn FnOnce() + Send>>
}

impl ThreadPool for MyThreadPool {
    fn new(size: usize) -> Self {
        let (tx, rx) = channel::unbounded::<Box<dyn FnOnce() + Send + 'static>>();
        for _ in 0..size {
            let r = rx.clone();
            thread::spawn(move || {
                loop {
                    match r.recv() {
                        Ok(task) => {
                            task();
                        }
                        Err(_) => debug!("Thread exits because the thread pool is destroyed."),
                    }
                }
            });
        }
        Self {
            sender: tx
        }
    }

    fn spawn<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.sender.send(Box::new(f)).unwrap();
    }

    fn spawn1(f: impl FnOnce() + Send) {
        unimplemented!()
    }
}

#[test]
fn test() {
    let pool = MyThreadPool::new(3);
    pool.spawn(||{println!("1")});
    pool.spawn(||{println!("2")});
    pool.spawn(||{println!("3")});
    pool.spawn(||{println!("4")});
}