use mongodb::{bson::Document, options::ClientOptions, Client};
use std::sync::Arc;
use axum::{Json, Router};
use axum::routing::{post};
use socketioxide::extract::SocketRef;
use socketioxide::SocketIo;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use crate::auth::auth::LoginRequest;
use crate::socket::io::io_on_connect;

mod auth;
mod socket;
mod database;
mod leaderboard;
mod internal;


#[tokio::main]
async fn main() {
    println!(" [info] Start server");

    let client_options: ClientOptions = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client: Client = Client::with_options(client_options).unwrap();
    let db: mongodb::Database = client.database("myApp");
    let collection: mongodb::Collection<Document> = db.collection::<Document>("users");
    let shared_collection: Arc<mongodb::Collection<Document>> = Arc::new(collection.clone());

    let shared_collection_clone = Arc::clone(&shared_collection);

    let _ = internal::indexing::main(&collection).await;

    let login_route = {
        let shared_collection = Arc::clone(&shared_collection);
        move |body: Json<LoginRequest>| {
            let shared_collection = Arc::clone(&shared_collection);
            async move {
                auth::auth::login(body, shared_collection).await // Используйте body.0 для доступа к LoginRequest
            }
        }
    };

    let register_route = {
        let shared_collection = Arc::clone(&shared_collection);
        move |body: Json<LoginRequest>| {
            let shared_collection = Arc::clone(&shared_collection);
            async move {
                auth::auth::register(body, shared_collection).await // Используйте body.0 для доступа к LoginRequest
            }
        }
    };

    // SocketIO
    let (layout, io) = SocketIo::new_layer();
    let io_clone = io.clone();

    let cors = CorsLayer::new().allow_origin(AllowOrigin::any()); // Разрешение запросов со всех источников

    io.ns("/", move |_s: SocketRef| {
        return io_on_connect(_s, shared_collection_clone, io_clone, shared_collection);
    });

    // Routing
    let app = Router::new()
        .route("/login", post(login_route))
        .route("/signup", post(register_route))
        .layer(layout) // Применение слоя Socket.IO
        .layer(cors) // Применение слоя CORS
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind("127.0.0.1:3001").await.unwrap();

    let _ = axum::serve(listener, app.into_make_service())
        .await;
}

