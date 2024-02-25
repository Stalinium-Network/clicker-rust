use mongodb::{bson::Document, options::ClientOptions, Client};
use std::sync::Arc;
use warp::Filter;

mod auth;
mod socket;


#[tokio::main]
async fn main() {
    let client_options: ClientOptions = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client: Client = Client::with_options(client_options).unwrap();
    let db: mongodb::Database = client.database("myApp");
    let collection: mongodb::Collection<Document> = db.collection::<Document>("users");
    let shared_collection: Arc<mongodb::Collection<Document>> = Arc::new(collection);

    let shared_collection_clone = Arc::clone(&shared_collection);
    let with_collection = warp::any().map(move || Arc::clone(&shared_collection.clone()));


    let login_route = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(with_collection.clone())
        .and_then(auth::auth::login);

    let register_route = warp::post()
        .and(warp::path("signup"))
        .and(warp::body::json())
        .and(with_collection)
        .and_then(auth::auth::register);

    let routes = login_route.or(register_route);

    let port = 3001;

    // Создание обработчиков для Warp
    let cors = warp::cors().allow_any_origin();
    let warp_routes = routes.with(cors);

    // Запуск HTTP сервера с использованием Warp
    let warp_server = warp::serve(warp_routes).run(([127, 0, 0, 1], port));

    // Запуск WebSocket (Socket.IO) сервера с использованием Axum
    let socket_io_server = socket::io::main(shared_collection_clone);

    // Параллельный запуск обоих серверов
    tokio::join!(warp_server, socket_io_server);
}


