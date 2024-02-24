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

    let port = 3000;

    println!("Server started on port {:?}", port);

    socket::io::main(shared_collection_clone).await;

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}


