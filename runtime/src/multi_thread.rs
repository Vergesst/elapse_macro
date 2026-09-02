use std::{sync::{Arc, Mutex, mpsc}, thread, time::{self, Duration}};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move || {
            loop {
                let job = {
                    let lock = receiver.lock().unwrap();
                    lock.recv()
                };

                match job {
                    Ok(job) => job(),
                    Err(mpsc::RecvError) => {break}
                }
            }
        });
        Worker { handle: Some(handle) }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>
}

impl ThreadPool {
    pub fn new(size: usize) -> Option<Self> {
        if size == 0 {
            return None
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }
        Some (ThreadPool { workers, sender })
    }

    pub fn execute_with_reply<F, T>(&self, f: F) -> mpsc::Receiver<T> 
        where F: FnOnce() -> T + Send + 'static,
              T: Send + 'static
    {
        let (sender, receiver) = mpsc::channel();
        let job = Box::new(move || {
            let result = f();
            let _ = sender.send(result);
        });

        self.sender.send(job).unwrap();
        receiver
    }

    pub fn join(mut self) {
        drop(self.sender);

        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}

pub struct ArcTimeGuard {
    pub instant_counter: time::Instant,
    pub duration_keeper: Arc<Mutex<Duration>>
}

impl ArcTimeGuard {
    pub fn new(duration_keeper: Arc<Mutex<Duration>>) -> Self {
        ArcTimeGuard {
            instant_counter: time::Instant::now(),
            duration_keeper
        }
    }
}

impl Drop for ArcTimeGuard {
    fn drop(&mut self) {
        let mut duration = self.duration_keeper.lock().unwrap();
        *duration = duration.checked_add(self.instant_counter.elapsed()).expect("Duration overflow");
    }
}
