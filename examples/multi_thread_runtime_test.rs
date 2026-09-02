use std::{sync::{Arc, Mutex}, thread, time::Duration};
use runtime::{ArcTimeGuard, ThreadPool};

const TIMES: usize = 10;

// pub struct ArcTimeGuard {
//     pub name: &'static str,
//     pub instant_counter: time::Instant,
//     pub duration_keeper: Arc<Mutex<Duration>>
// }

// impl ArcTimeGuard {
//     pub fn new(name: &'static str) -> Self {
//         ArcTimeGuard {
//             name,
//             instant_counter: time::Instant::now(),
//             duration_keeper: Arc::new(Mutex::new(Duration::new(0, 0)))
//         }
//     }
// }

// impl Drop for ArcTimeGuard {
//     fn drop(&mut self) {
//         let mut duration = self.duration_keeper.lock().unwrap();
//         *duration = duration.checked_add(self.instant_counter.elapsed()).expect("Duration overflow");
//     }
// }

fn target_test_func() -> i32 {
    thread::sleep(Duration::from_secs(1));
    1
}

fn main() {
    // initialize runtime 
    let counter = Arc::new(Mutex::new(Duration::ZERO));
    let name = "main task";
    let mut receivers = vec![];
    if let Some(thread_pool) = ThreadPool::new(TIMES) {
        for _ in 0..TIMES {
            let shared = Arc::clone(&counter);
            let rx = thread_pool.execute_with_reply(move || {
                let _guard = ArcTimeGuard::new(shared);
                // main task
                // thread::sleep(Duration::from_secs(1));
                target_test_func()
            });
            receivers.push(rx);
        }

        thread_pool.join();
    }

    match receivers[0].recv() {
        Ok(inner) => println!("inner value is {}", inner),
        Err(_) => panic!(),
    }

    let total = counter.lock().expect("Mutex is posioned");
    let average = *total / TIMES as u32;
    println!("average time usage of task {} is {:?}", name, average);
}