use std::thread;
use std::time::Duration;

pub fn set_interval(func: fn(), ms: u64) -> () {
    let interval = Duration::from_millis(ms);

    let handle = thread::spawn(move || {
        loop {
            thread::sleep(interval);

            func();
        }
    });

    handle.join().unwrap();
}