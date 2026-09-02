#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]
mod pool;
use std::{thread, time::Duration};

use pool::ThreadPool;

fn main() {
    if let Some(pool) = ThreadPool::new(4) {
        let mut receivers = vec![];

        for i in 0..10 {
            let rx = pool.execute_with_reply(move || 
            // #[elapse::elapse_expr]    
            {
                thread::sleep(Duration::from_secs(1));
                format! {
                    "task {} done by {:?}",
                    i,
                    thread::current().id()
                }
            });
            receivers.push(rx);
        }

        for rx in receivers {
            let result = rx.recv().unwrap();
            println!("{}", result);
        }
        pool.join();
    } else {
        println!("the size of thread pool must bigger than 0")
    }
}
