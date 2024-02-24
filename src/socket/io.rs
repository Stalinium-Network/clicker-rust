use std::sync::Arc;

use axum;
use lazy_static::lazy_static;
use mongodb::{bson::{doc, Document}, options::ClientOptions, Client, Collection};
use socketioxide::{extract::{SocketRef, Data}, SocketIo};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;


fn on_connect(socket: SocketRef, shared_collection: Arc<Collection<Document>>) {
    info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);
    socket.emit("auth", "auth").ok();

    socket.on("msg", |socket: SocketRef, Data::<String>(msg)| {
        info!("Received event: {:?}", msg);


        socket.emit("message-back", doc! {"id": "1"}).ok();
    });
}

// #[tokio::main]
pub async fn main(shared_collection: Arc<Collection<Document>>) -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let (layer, io) = SocketIo::new_layer();

    io.ns("/", |socket: SocketRef| {
        on_connect(socket, shared_collection)
    }
);

    let app = axum::Router::new()
    .layer(
        ServiceBuilder::new()
            .layer(CorsLayer::permissive())
            .layer(layer),
    );

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
