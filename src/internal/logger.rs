use std::sync::Arc;
use dashmap::DashMap;
use lazy_static::lazy_static;
use tokio::time::Instant;

lazy_static! {
    static ref MAP: Arc<DashMap<String, Instant>> = Arc::new(DashMap::new());
}


pub fn time(id: &str) {
    let _time = Instant::now();
    MAP.insert(id.to_string(), _time);
}

pub fn time_end(id: &str) {
    if let Some(target) = MAP.get(id) {
        println!("[{:?}] {:?}", target.elapsed(), id)
    } else {
        println!(" logger err nf-2")
    }
}