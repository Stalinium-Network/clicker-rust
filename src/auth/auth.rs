use crate::auth::sha256::hash_password;
use mongodb::bson::{doc, Document};
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use warp::http::StatusCode;
use std::time::Instant;


#[derive(Deserialize)]
pub struct LoginRequest {
    id: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    id: String,
    password: String,
}

pub async fn login(
    body: LoginRequest,
    client: Arc<Collection<Document>>,
) -> Result<impl warp::Reply, Infallible> {
    let filter = doc! { "_id": &body.id, "password": hash_password(&body.password) };
    let user = client.find_one(filter, None).await.unwrap();

    if !user.is_some() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"Неправильный логин или пароль"),
            StatusCode::NOT_FOUND,
        ));
    }



    println!("{:?}", user);

    let response: LoginResponse = LoginResponse {
        id: body.id.clone(),
        password: body.password.clone(),
    };

    // Отправка ответа
    Ok(warp::reply::with_status(
        warp::reply::json(&response), 
        StatusCode::OK
    ))
}


// ==== [РЕГИСТРАЦИЯ] ====
pub async fn register(
    body: LoginRequest,
    client: Arc<Collection<Document>>,
) -> Result<impl warp::Reply, Infallible> {
    let start: Instant = Instant::now();

    let user = client
        .find_one(doc! {"_id": body.id.clone()}, None)
        .await
        .unwrap();

    if user.is_some() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"User already exists"),
            StatusCode::CONFLICT,
        ));
    }


    if body.id.len() > 100 || body.password.len() > 100 {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"too big name or password"),
            StatusCode::BAD_REQUEST,
        ));
    }

    if body.id.len() < 4 || body.password.len() < 4 {
        return Ok(warp::reply::with_status(
            warp::reply::json(&"minimum name or password length = 4"),
            StatusCode::BAD_REQUEST,
        ));
    }

    let hashed_password = hash_password(&body.password);

    let result: mongodb::results::InsertOneResult = client
        .insert_one(
            doc! {
                "_id": &body.id,
                "password": hashed_password,
                "balance": 0,
                "tools": {
                    "autoclicker": 0,
                    "click": 0
                }
            },
            None,
        )
        .await
        .unwrap();

    println!("{:?}", result);

    let duration = start.elapsed(); // Окончание замера времени
    println!("Время выполнения: {:?}", duration);

    return Ok(warp::reply::with_status(
        warp::reply::json(&"OK"),
        StatusCode::OK,
    ));
}
