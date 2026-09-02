use std::{sync::{Arc, Mutex, mpsc::{self}}, thread::{self}};
use elapse::*;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    _handle: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move || {
            println!("worker {:?} is running", thread::current().id());
            loop {
                let job = {
                    let lock = receiver.lock().unwrap();
                    lock.recv()
                };

                match job {
                    Ok(job) => job(),
                    Err(mpsc::RecvError) => { break }
                }
            }
        });
        Worker { _handle: Some(handle) }
    }
}

pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: mpsc::Sender<Job>
}

impl ThreadPool {
    pub fn new(size: usize) -> Option<Self>{
        if size == 0 {
            return None
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }
        Some(ThreadPool { _workers: workers, sender })
    }

    pub fn execute<F: FnOnce() + Send + 'static> (&self, f: F) {
        let job = Box::new(f);
        self.sender.send(job).unwrap()
    }

    pub fn execute_with_reply<F: FnOnce() -> T + Send + 'static, T: Send + 'static> (&self, f: F) -> mpsc::Receiver<T> {
        let (tx, rx) = mpsc::channel();
        let job = Box::new(move || #[elapsed("")]{
            let result = f();
            let _ = tx.send(result);
        });

        
        self.sender.send(job).unwrap();
        rx
    }

    // move self here to comsume it
    pub fn join(mut self) {
        drop(self.sender);

        for worker in &mut self._workers {
            if let Some(handle) = worker._handle.take() {
                handle.join().unwrap();
            }
        }
        println!("all workers joined");
    }
}