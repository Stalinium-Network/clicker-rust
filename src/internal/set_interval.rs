use std::thread;
use std::time::Duration;

pub async fn set_interval(func: fn(), ms: i32) -> () {
    let interval = Duration::from_millis(ms);

    let handle = thread::spawn(move || {
        loop {
            thread::sleep(interval);

            func();
        }
    });

    handle.join().unwrap();
}