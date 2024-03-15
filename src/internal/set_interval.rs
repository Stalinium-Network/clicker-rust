use std::time::Duration;
use tokio::time;

// функция setInterval()
pub async fn set_interval<F>(mut func: F, ms: u64)
    where
        F:  FnMut() + Send + 'static,
{
    let mut interval = time::interval(Duration::from_millis(ms));

    loop {
        interval.tick().await;
        func()
    }
}
