use std::{sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use runtime::ThreadPool;
use std::time;

const TIMES: usize = 10;

fn main() {
    // initialize runtime 
    let counter = Arc::new(Mutex::new(time::Duration::new(0, 0)));

    let mut receivers = vec![];
    if let Some(thread_pool) = ThreadPool::new(TIMES) {
        for _ in 0..TIMES {
            let shared = Arc::clone(&counter);
            let rx = thread_pool.execute_with_reply(move || {
                let mut guard = shared.lock().unwrap();
                let timer = Instant::now();
                // main task
                thread::sleep(Duration::from_secs(1));
                let elapsed = timer.elapsed();
                *guard = guard.checked_add(elapsed).expect("Duration overflow");
            });
            receivers.push(rx);
        }

        thread_pool.join();
    }

    let total = Arc::try_unwrap(counter)
        .expect("Other reference still exists and cannot take the value")
        .into_inner()
        .expect("Mutex is posioned");

    let average = total / TIMES as u32;
    println!("average time usage {:?}", average);
}