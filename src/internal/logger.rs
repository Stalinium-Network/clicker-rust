use std::sync::Arc;
use dashmap::DashMap;
use lazy_static::lazy_static;
use tokio::time::Instant;

lazy_static! {
    static ref MAP: Arc<DashMap<String, Instant>> = Arc::new(DashMap::new());
}

/**
Аналог console.time() и console.timEnd() из NodeJS для измерения времени выполнения кода
 **/

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

/*
Функция для вывода данных в консоль только в debug сборке
*/
#[cfg(debug_assertions)]
pub fn debug(text: &str) {
    println!(" [debug] {}", text);
}

#[cfg(not(debug_assertions))]
pub fn debug(text: &str) {}