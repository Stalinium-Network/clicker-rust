extern crate chrono;
use chrono::prelude::*;
use crate::internal::conf::main::get_conf;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

lazy_static! {
    static ref MESSAGES: Mutex<Vec<MessageItem>> =Mutex::new(Vec::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageItem {
    id: String,
    text: String,
    time: String,
}


pub async fn add_msg(id: String, msg: String) -> MessageItem {
    let msg_obj = MessageItem {
        id,
        text: msg,
        time: Utc::now().to_string(),
    };

    let mut messages_lock = MESSAGES.lock().await;

    messages_lock.push(msg_obj.clone());

    if messages_lock.len() > get_conf().max_messages_len {
        messages_lock.remove(0);
    }

    return msg_obj;
}

pub async fn get_messages() -> Vec<MessageItem> {
    let messages_lock = MESSAGES.lock().await;

    messages_lock
        .clone()
}