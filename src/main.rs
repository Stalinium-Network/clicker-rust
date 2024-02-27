use mongodb::{bson::Document, options::ClientOptions, Client};
use std::sync::Arc;
use warp::Filter;
use warp::http::Method;

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


    let port = 3001;

    // Настройка CORS
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "User-Agent", "Sec-Fetch-Mode", "Referer", "Origin",
            "Access-Control-Request-Method", "Access-Control-Request-Headers",
            "Content-Type" // Убедитесь, что Content-Type разрешен
        ])
        .allow_methods(vec!["POST", "GET"]);

    // Добавление обработчика для OPTIONS
    let options_route = warp::options()
        .map(|| warp::reply::with_header("OK", "Access-Control-Allow-Origin", "*"));

    // Комбинирование всех маршрутов с поддержкой CORS
    let routes = login_route
        .or(register_route)
        .or(options_route)
        .with(cors);

    // Запуск HTTP сервера с использованием Warp
    let warp_server = warp::serve(routes).run(([127, 0, 0, 1], port));

    // Запуск Socket.IO сервера с использованием Axum
    let socket_io_server = socket::io::main(shared_collection_clone);

    // Параллельный запуск обоих серверов
    tokio::join!(warp_server, socket_io_server);
}