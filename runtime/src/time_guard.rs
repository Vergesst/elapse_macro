use std::time;

pub struct TimeGuard {
    name: &'static str,
    time_counter: time::Instant,
}

impl TimeGuard {
    pub fn new(name: &'static str) -> Self {
        TimeGuard { name, time_counter: time::Instant::now() }
    }
}

impl Drop for TimeGuard {
    fn drop(&mut self) {
        eprintln!("task \"{}\" consumes {:?}", self.name, self.time_counter.elapsed())
    }
}