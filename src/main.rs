use mongodb::{bson::Document, options::ClientOptions, Client};
use std::sync::Arc;
use axum::{Json, Router};
use axum::routing::{post};
use serde_json::to_string;
use socketioxide::extract::SocketRef;
use socketioxide::SocketIo;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use crate::auth::auth::LoginRequest;
use crate::internal::conf::main::{get_conf, load_conf};
use crate::internal::set_interval::set_interval;
use crate::leaderboard::main::get_leaderboard;
use crate::socket::io::io_on_connect;

mod auth;
mod socket;
mod database;
mod leaderboard;
mod internal;
mod chat;


#[tokio::main]
async fn main() {
    println!(" [info] Start server");

    let _ = load_conf().await;

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
                auth::auth::login(body, shared_collection).await
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

    io.ns("/leaderboard", move |_s: SocketRef| {
        async fn handler(_s: SocketRef) {
            let leaderboard = get_leaderboard(7).await; // Получаем leaderboard как Vec<LeaderBoardItem>
            let serialized_leaderboard = to_string(&leaderboard).expect("Не удалось сериализовать leaderboard");

            println!("sended");
            _s.emit("leaderboard", serialized_leaderboard).ok();
        }

        return handler(_s);
    });

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

    let _ = tokio::spawn(async move {
        let _ = axum::serve(listener, app.into_make_service())
            .await;
    });


    let io_clone = io.clone();
    set_interval(move || {
        let io_clone = io_clone.clone();

        tokio::spawn(async move {
            let conf = get_conf();
            
            let leaderboard = get_leaderboard(conf.max_mun_of_users2send).await; // Получаем leaderboard как Vec<LeaderBoardItem>
            let serialized_leaderboard = to_string(&leaderboard).expect("Не удалось сериализовать leaderboard");

            io_clone.emit("leaderboard", serialized_leaderboard).ok();
        });
    }, 1_500).await;
}

