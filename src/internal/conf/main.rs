use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

static CONF: once_cell::sync::OnceCell<Arc<ConfStruct>> = once_cell::sync::OnceCell::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfStruct {
    pub port: usize,
    pub max_leaderboard_arr: usize,
    pub max_messages_len: usize,
    pub max_mun_of_users2send: usize,
    pub max_message_length: usize,
    pub send_leaderboard_interval: usize,
}

// загрузка файла конфигурации
pub async fn load_conf() {
    let mut file = match File::open("./conf.yaml") {
        Ok(file) => file,
        Err(e) => {
            println!(" [error] Error while loading config: {:?}", e);
            return;
        }
    };

    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let conf: ConfStruct = serde_yaml::from_str(&contents).unwrap();

    CONF.set(Arc::new(conf))
        .map_err(|_| println!("Configuration was already initialized"))
        .unwrap();

    println!(" [info] Load Config");
}

// получение концигурации
pub fn get_conf() -> Arc<ConfStruct> {
    CONF.get().expect("Configuration is not initialized").clone()
}