use std::sync::Arc;
use axum::{Extension, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use crate::auth::sha256::hash_password;



#[derive(Deserialize)]
pub struct LoginRequest {
    id: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    id: String,
    password: String,
    error: Option<String>,
}


pub async fn login(
    Json(body): Json<LoginRequest>,
    client: Arc<Collection<Document>>,
) -> impl IntoResponse  {
    println!("login Req");

    let filter = doc! { "_id": &body.id, "password": hash_password(&body.password) };
    let user = client.find_one(filter, None).await.unwrap();

    if !user.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(create_error("Неправильный логин или пароль")),
        );
    }


    println!("{:?}", user);

    // Отправка ответа
    return (
        StatusCode::OK,
        Json(LoginResponse {
            id: body.id.clone(),
            password: body.password.clone(),
            error: None,
        })
    )
}


// ==== [РЕГИСТРАЦИЯ] ====
pub async fn register(
    Json(body): Json<LoginRequest>,
    client: Arc<Collection<Document>>,
) -> impl IntoResponse {
    let start: Instant = Instant::now();
    println!("register Req");

    let user = client
        .find_one(doc! {"_id": body.id.clone()}, None)
        .await
        .unwrap();

    if user.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(create_error("User already exists")),
        );
    }


    if body.id.len() > 100 || body.password.len() > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(create_error("too big name or password")),
        );
    }

    if body.id.len() < 4 || body.password.len() < 4 {
        return (
            StatusCode::BAD_REQUEST,
            Json(create_error("minimum name or password length = 4")),
        );
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

    return (
        StatusCode::OK,
        Json(LoginResponse {
            id: body.id.clone(),
            password: body.password.clone(),
            error: None,
        })
    );
}

fn create_error(msg: &str) -> LoginResponse {
    let res = LoginResponse {
        id: "".to_string(),
        password: "".to_string(),
        error: Some(msg.to_string()),
    };
    return res;
}