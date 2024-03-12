use lazy_static::lazy_static;
use serde::Serialize;
use tokio::sync::Mutex;
lazy_static! {
    static ref MESSAGES: Mutex<Vec<MessageItem>> =Mutex::new(Vec::new());
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageItem {
    id: String,
    text: String,
    time: String,
}


pub async fn add_msg(id: String, msg: String) -> MessageItem {
    let msg_obj = MessageItem {
        id,
        text: msg,
        time: "12:30".to_string(),
    };

    let mut messages_lock = MESSAGES.lock().await;

    messages_lock.push(msg_obj.clone());

    return msg_obj;
}

pub async fn get_messages(len: usize) {

}