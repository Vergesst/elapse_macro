#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]

use std::{thread, time::Duration};

use elapse::elapsed_multi_thread;

fn main() {
    // target_speed_test_function();
    // println!();
    // loop_speed_test_function();
    // println!();
    // certain_test_func();
    let a = 3;
    let _result = test2(a);
    println!();
    loop_speed_test_function();
}

// #[elapsed("")]
// fn target_speed_test_function() -> i32 {
//     let a = 3;
//     let b = a + 3;
//     b
// }

fn loop_speed_test_function() {
    let mut result = 3;
    #[elapsed_multi_thread("loop")]
    loop {
        result += 1;
        thread::sleep(Duration::from_secs(1));

        if result > 10 {
            break;
        }
    }
}

// #[elapsed("")]
// fn certain_test_func() {
//     println!("this is a testing function");
// }

#[elapsed_multi_thread("", 0x13)]
fn test2(a: i32) -> i32 {
    thread::sleep(Duration::from_secs(1));
    a
}
