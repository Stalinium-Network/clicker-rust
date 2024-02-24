use axum;
use mongodb::bson::doc;
use socketioxide::{extract::{SocketRef, Data}, SocketIo};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

enum Res {
    Message {
        username: String,
        message: String,
    },
}

fn on_connect(socket: SocketRef) {
    info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);
    socket.emit("auth", "auth").ok();

    socket.on("msg", |socket: SocketRef, Data::<String>(msg)| {
        info!("Received event: {:?}", msg);

        let msg = Res::Message {
            username: "test".to_string(),
            message: msg,
        };

        socket.emit("message-back", doc! {"id": "1"}).ok();
    });
}

// #[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let (layer, io) = SocketIo::new_layer();

    io.ns("/", on_connect);

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
