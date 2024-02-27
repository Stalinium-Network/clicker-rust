use std::sync::{Arc, Mutex};
use crate::auth::sha256::hash_password;

use axum;
use lazy_static::lazy_static;
use mongodb::{bson::{doc, Document}, options::ClientOptions, Client, Collection};
use socketioxide::{extract::{SocketRef, Data}, SocketIo};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use std::collections::HashMap;
use std::ptr::null;
use serde::Deserialize;
use tokio::net::TcpListener;


struct UserData {
    data: Document,
}

#[derive(Deserialize)]
struct BuyData {
    action: String,
}


async fn on_connect(client: SocketRef, shared_collection: Arc<Collection<Document>>, io: SocketIo) {
    info!("Socket.IO connected: {:?} {:?}", client.ns(), client.id);
    let uri_string = client.req_parts().uri.clone().to_string(); // Создаем строку из URI
    let query = uri_string.split_once('?').map_or("", |(_, q)| q); // Теперь `uri_string` живет достаточно долго

    let params = parse_query_string(query);

    // Преобразуем Option<&String> в String, создавая копию при необходимости
    let id = params.get("id").cloned().unwrap_or_else(|| "".to_string());
    let password = params.get("password").cloned().unwrap_or_else(|| "".to_string());

    if id.is_empty() || password.is_empty() {
        client.emit("error", "401");
        client.disconnect();
        return;
    }

    let result = shared_collection.find_one(doc! {"_id": id.clone(), "password": &hash_password(&password.clone())}, None).await;

    let user = match result.clone() {
        Ok(Some(user_doc)) => user_doc, // Пользователь найден
        Ok(None) => {
            // Пользователь не найден
            println!("ok2 {:?}", result);
            client.emit("error", "user not found, incorrect id or password");
            client.disconnect();
            return;
        }
        Err(_) => {
            client.emit("error", "Database query failed");
            client.disconnect();
            return;
        }
    };

    println!("id: {}, password: {}", id, password);

    let user_info = Arc::new(Mutex::new(UserData { data: user }));
    let user_info_for_msg = user_info.clone();

    client.on("msg", move |client: SocketRef, Data::<String>(msg)| {
        let user_data_lock = user_info_for_msg.lock().unwrap();
        info!("Received event: {:?}, from user: {:?}", msg, user_data_lock.data);
        println!("msg");

        client.emit("message-back", doc! {"id": "1"}).ok();
    });

    client.on("buy", move |client: SocketRef, Data::<BuyData>(data)| {
        // info!("Received event: {:?}, from user: {:?}", msg, user);


        client.emit("message-back", doc! {"id": "2"}).ok();
    });
}

// #[tokio::main]
pub async fn main(shared_collection: Arc<Collection<Document>>) -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let (layer, io) = SocketIo::new_layer();

    let io_clone = io.clone();

    io.ns("/", move |socket: SocketRef| {
        on_connect(socket, shared_collection, io_clone)
    });

    let app =
        axum::Router::new()
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
                    .layer(layer)
            );

    println!("socket io started");

    let listener = TcpListener::bind("127.0.0.1:3002").await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await?;

    Ok(())
}


fn parse_query_string(query: &str) -> HashMap<String, String> {
    query.split('&')
        .map(|part| part.split_once('=').unwrap_or((part, "")))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}