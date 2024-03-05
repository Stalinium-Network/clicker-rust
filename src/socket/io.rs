use std::sync::{Arc};
use crate::auth::sha256::hash_password;

use lazy_static::lazy_static;
use mongodb::{bson::{doc, Document}, bson, Collection};
use socketioxide::{extract::{SocketRef, Data}, SocketIo};
use std::collections::HashMap;
use mongodb::bson::to_document;
use serde::{Deserialize, Serialize};
use socketioxide::socket::DisconnectReason;
use tokio::sync::Mutex;
use tokio::time::Instant;
use crate::leaderboard::main::{get_leaderboard, LeaderBoardItem, update_leaderboard_user_pos};

lazy_static! {
    static ref USERS_ONLINE: Mutex<HashMap<String, UserDataLazyStatic>> = Mutex::new(HashMap::new());
}

struct UserDataLazyStatic {
    balance: i64,
    id: String,
}

#[derive(Serialize, Debug)]
struct UserDataSimpled {
    balance: i64,
    id: String,
}

struct UserData {
    raw: Document,
    data: DefaultGameStats,
    _id: String,
}

#[derive(Deserialize, Debug)]
struct BuyData {
    action: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct DefaultGameStatsItemStats {
    cost: i64,
    amount: i64,
    value: i64,
}

#[derive(Deserialize, Serialize, Debug)]
struct DefaultGameStatsSpeed {
    cost: i64,
    unlocked: bool,
    multiplier: i64,
}

#[derive(Deserialize, Serialize, Debug)]
struct DefaultGameStatsReset {
    minCost: i64,
}

#[derive(Deserialize, Serialize, Debug)]
struct DefaultGameStatsOtherUpgrades {
    higherHackAmount: bool,
    betterFirewall: bool,
    covertHacks: bool,
    doubleHacks: bool,
}

#[derive(Deserialize, Serialize, Debug)]
struct DefaultGameStats {
    balance: i64,
    mpc: DefaultGameStatsItemStats,
    auto: DefaultGameStatsItemStats,
    triple: DefaultGameStatsItemStats,
    superAuto: DefaultGameStatsItemStats,
    speed: DefaultGameStatsSpeed,
    reset: DefaultGameStatsReset,
    otherUpgrades: DefaultGameStatsOtherUpgrades,
}


// строим структуру для хранения цен и статистики
struct BasePrices {
    auto: u32,
    mpc: u32,
    super_auto: u32,
    triple: u32,
}

struct IncrementalExponents {
    auto: f64,
    mpc: f64,
    super_auto: f64,
    triple: f64,
}

struct UpgradeCosts {
    higher_hack_amount: u32,
    better_firewall: u32,
    covert_hacks: u32,
    double_hacks: u32,
}

// инициализируем структуру

impl BasePrices {
    const AUTO: u32 = 200;
    const MPC: u32 = 50;
    const SUPER_AUTO: u32 = 10000;
    const TRIPLE: u32 = 500;
}

impl IncrementalExponents {
    const AUTO: f64 = 1.145;
    const MPC: f64 = 1.1;
    const SUPER_AUTO: f64 = 1.145;
    const TRIPLE: f64 = 1.145;
}

impl UpgradeCosts {
    const HIGHER_HACK_AMOUNT: u32 = 1000000;
    const BETTER_FIREWALL: u32 = 1000000;
    const COVERT_HACKS: u32 = 1000000;
    const DOUBLE_HACKS: u32 = 10000000;
}

/*
 * Событие подключения к Socket.IO
 */

pub async fn io_on_connect(client: SocketRef, shared_collection: Arc<Collection<Document>>, io: SocketIo, db_client: Arc<Collection<Document>>) {
    let uri_string = client.req_parts().uri.clone().to_string(); // Создаем строку из URI
    let query = uri_string.split_once('?').map_or("", |(_, q)| q); // Теперь `uri_string` живет достаточно долго

    let params = parse_query_string(query);

    // Преобразуем Option<&String> в String, создавая копию при необходимости
    let id = params.get("id").cloned().unwrap_or_else(|| "".to_string());
    let password = params.get("password").cloned().unwrap_or_else(|| "".to_string());

    if id.is_empty() || password.is_empty() {
        client.emit("error", "401");
        client.disconnect();
        return;
    }

    if USERS_ONLINE.lock().await.contains_key(&id) {
        println!("user already connected");
        client.emit("error", "другое устройство уже вошло в аккаунт");
        client.disconnect();
        return;
    }
    println!("user already connected2");

    let result = shared_collection.find_one(doc! {"_id": id.clone(), "password": &hash_password(&password.clone())}, None).await;

    let mut user = match result.clone() {
        Ok(Some(user_doc)) => user_doc, // Пользователь найден
        Ok(None) => {
            // Пользователь не найден
            println!("ok2 {:?}", result);
            client.emit("error", "401");
            client.disconnect();
            return;
        }
        Err(_) => {
            client.emit("error", "401");
            client.disconnect();
            return;
        }
    };

    // удалить пароль из данных пользователя
    user.remove("password");

    println!("connected");

    let user_obj: DefaultGameStats = bson::from_document(user.get_document("gameStats").unwrap().clone()).unwrap();
    let user_info = Arc::new(Mutex::new(UserData { data: user_obj, _id: id.clone(), raw: user.clone() }));

    client.emit("data", user.clone()).ok();

    let user_info_lock = user_info.lock().await;

    USERS_ONLINE.lock().await
        .insert(id.clone(), UserDataLazyStatic {
            balance: user_info_lock.data.balance.clone(),
            id: user_info_lock._id.clone(),
        });


    let user_info_for_msg = user_info.clone();
    client.on("saveData", move |client: SocketRef, Data::<DefaultGameStats>(data)| async move {
        println!("saveData");
        let _start: Instant = Instant::now();

        println!("--");
        let mut user_data_lock = user_info_for_msg.lock().await; // Используйте .await здесь
        println!("--user_data_lock");

        let game_stats_doc = match to_document(&data) {
            Ok(doc) => doc,
            Err(e) => {
                eprintln!("Ошибка при сериализации данных игры: {:?}", e);
                client.emit("error", "Ошибка при сохранении данных").ok();
                return;
            }
        };

        user_data_lock.raw.insert("gameStats", game_stats_doc.clone());

        println!("update_leaderboard_user_pos()");
        update_leaderboard_user_pos(LeaderBoardItem { id: user_data_lock._id.clone(), balance: user_data_lock.data.balance }).await;
        println!("saveData -2");

        USERS_ONLINE.lock().await
            .insert(user_data_lock._id.clone(), UserDataLazyStatic {
                balance: user_data_lock.data.balance.clone(),
                id: user_data_lock._id.clone(),
            });

        let _durration = _start.elapsed();
        println!("saveData: {:?}", _durration);
    });

    client.on("getLeaderboard", move |client: SocketRef| async move {
        println!(" --- --- --- --- getLeaderboard");

        let leaderboard = get_leaderboard().await;

        println!("{:?}", leaderboard);
        client.emit("leaderboard", "".to_string()).ok();
    });

    let user_info_for_msg = user_info.clone();
    client.on_disconnect(move |client: SocketRef, reason: DisconnectReason| async move {
        println!("disconnect");
        let _start: Instant = Instant::now();

        let binding = user_info_for_msg.clone();
        let user_data_lock = binding.lock().await;
        let db_client_clone = db_client.clone();

        let data = user_data_lock.raw.clone();

        let game_stats_doc = match to_document(&data) {
            Ok(doc) => doc,
            Err(e) => {
                eprintln!("Ошибка при сериализации данных игры: {:?}", e);
                client.emit("error", "Ошибка при сохранении данных").ok();
                return;
            }
        };

        let game_stats = match game_stats_doc.get_document("gameStats") {
            Ok(doc) => doc,
            Err(e) => {
                eprintln!("Ошибка доступа к 'gameStats': {:?}", e);
                return;
            }
        };

        let _ = db_client_clone.update_one(
            doc! {
                "_id": user_data_lock._id.clone()
            },
            doc! {
                "$set": { "gameStats": game_stats }
            }, None,
        ).await;

        let id = user_data_lock._id.clone();
        let mut users_online_lock = USERS_ONLINE.lock().await;
        users_online_lock.remove(&id);

        let _durration = _start.elapsed();
        println!("disconnect handler: {:?}", _durration);
    });
}

fn parse_query_string(query: &str) -> HashMap<String, String> {
    query.split('&')
        .map(|part| part.split_once('=').unwrap_or((part, "")))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}
