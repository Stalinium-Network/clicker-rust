use mongodb::{options::ClientOptions, Client};
use std::sync::Arc;
use warp::Filter;
mod auth;
mod socket;


#[tokio::main]
async fn main() {
    let client_options: ClientOptions = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client: Client = Client::with_options(client_options).unwrap();
    let client: Arc<Client> = Arc::new(client); // Обернуть в Arc для возможности клонирования между потоками
    
    let with_db = warp::any().map(move || Arc::clone(&client));

    let login_route = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(auth::auth::login);

    let register_route = warp::post()
        .and(warp::path("signup"))
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(auth::auth::register);

    let routes = login_route.or(register_route);

    let port = 3000;

    println!("Server started on port {:?}", port);

    socket::io::main().await;

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}


