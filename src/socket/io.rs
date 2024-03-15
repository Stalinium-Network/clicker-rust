use std::sync::{Arc};
use crate::auth::sha256::hash_password;
use dashmap::DashMap;
use lazy_static::lazy_static;
use mongodb::{bson::{doc, Document}, bson, Collection};
use socketioxide::{extract::{SocketRef, Data}, SocketIo};
use std::collections::HashMap;
use mongodb::bson::to_document;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use socketioxide::socket::DisconnectReason;
use tokio::sync::Mutex;
use tokio::time::Instant;
use crate::chat::db::{add_msg, get_messages, MessageItem};
use crate::internal::conf::main::get_conf;
use crate::leaderboard::main::{get_leaderboard, LeaderBoardItem, update_leaderboard_user_pos};
use crate::internal::logger;

lazy_static! {
    // список пользователей в сети
    // так же нужно для того что бы не разрешать подключения с нескольких устройств на один аккаунт
    static ref USERS_ONLINE: DashMap<String, UserDataLazyStatic> = DashMap::new();
}

struct UserDataLazyStatic {
    balance: u128,
    id: String,
}

#[derive(Serialize, Debug)]
struct UserDataSimpled {
    balance: u128,
    id: String,
}

struct UserData {
    raw: Document,
    _id: String,
    balance: u128,
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
#[allow(non_snake_case)]
struct DefaultGameStatsReset {
    minCost: i64,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct DefaultGameStatsOtherUpgrades {
    higherHackAmount: bool,
    betterFirewall: bool,
    covertHacks: bool,
    doubleHacks: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct DefaultGameStats {
    balance: String,
    mpc: DefaultGameStatsItemStats,
    auto: DefaultGameStatsItemStats,
    triple: DefaultGameStatsItemStats,
    superAuto: DefaultGameStatsItemStats,
    speed: DefaultGameStatsSpeed,
    reset: DefaultGameStatsReset,
    otherUpgrades: DefaultGameStatsOtherUpgrades,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct UserDataObj {
    _id: String,
    gameStats: DefaultGameStats,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct Data2User {
    userInfo: Document,
    messages: Vec<MessageItem>,
}

// строим структуру для хранения цен и статистики
// возможно понадобится в будующем, но сейчас убрано, что бы не мешались предупреждения при сборке
/*
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
*/
/*
 * Событие подключения к Socket.IO
 */

pub async fn io_on_connect(client: SocketRef, shared_collection: Arc<Collection<Document>>, _io: SocketIo, db_client: Arc<Collection<Document>>) {
    logger::time("connection handler");

    send_leaderboard(&client).await;

    let uri_string = client.req_parts().uri.clone().to_string(); // Создаем строку из URI
    let query = uri_string.split_once('?').map_or("", |(_, q)| q); // Теперь `uri_string` живет достаточно долго

    let params = parse_query_string(query);

    // Преобразуем Option<&String> в String, создавая копию при необходимости
    let id = params.get("id").cloned().unwrap_or_else(|| "".to_string());
    let password = params.get("password").cloned().unwrap_or_else(|| "".to_string());

    if id.is_empty() || password.is_empty() {
        let _ = client.emit("error", "401");
        let _ = client.disconnect();
        return;
    }

    if USERS_ONLINE.contains_key(&id) {
        println!("user already connected");
        let _ = client.emit("error", "другое устройство уже вошло в аккаунт");
        let _ = client.disconnect();
        return;
    }

    let result = shared_collection.find_one(doc! {"_id": id.clone(), "password": &hash_password(&password.clone())}, None).await;

    let mut user = match result.clone() {
        Ok(Some(user_doc)) => user_doc, // Пользователь найден
        Ok(None) => {
            // Пользователь не найден
            println!("ok2 {:?}", result);
            let _ = client.emit("error", "401");
            let _ = client.disconnect();
            return;
        }
        Err(_) => {
            let _ = client.emit("error", "401");
            let _ = client.disconnect();
            return;
        }
    };

    // удалить пароль из данных пользователя
    user.remove("password");


    println!("connected");

    let user_obj: DefaultGameStats = bson::from_document(user.get_document("gameStats").unwrap().clone()).unwrap();
    let user_info = Arc::new(Mutex::new(UserData {
        balance: user_obj.balance.parse::<u128>().unwrap_or(0),
        _id: id.clone(),
        raw: user.clone(),
    }));

    client.emit("data", Data2User {
        userInfo: user.clone(),
        messages: get_messages().await,
    }).ok();

    let user_info_lock = user_info.lock().await;

    USERS_ONLINE
        .insert(id.clone(), UserDataLazyStatic {
            balance: user_info_lock.balance.clone(),
            id: user_info_lock._id.clone(),
        });

    let user_info_for_msg = user_info.clone();

    logger::time_end("connection handler");

    /*
    Обработка события сохранения данных для протокола SocketIO
     */
    client.on("saveData", move |client: SocketRef, Data::<DefaultGameStats>(data)| async move {
        logger::time("saveData");
        let mut user_data_lock = user_info_for_msg.lock().await;

        let game_stats_doc = match to_document(&data) {
            Ok(doc) => doc,
            Err(e) => {
                eprintln!("Ошибка при сериализации данных игры: {:?}", e);
                client.emit("error", "Ошибка при сохранении данных").ok();
                return;
            }
        };

        println!("balance: {:?}", data.balance);
        let new_balance: u128 = data.balance.parse::<u128>().unwrap();

        logger::time("update_leaderboard_user_pos()");
        update_leaderboard_user_pos(
            LeaderBoardItem { id: user_data_lock._id.clone(), balance: new_balance },
            &user_data_lock.balance,
            &new_balance,
        ).await;
        logger::time_end("update_leaderboard_user_pos()");


        user_data_lock.raw.insert("gameStats", game_stats_doc.clone());
        user_data_lock.balance = new_balance;

        USERS_ONLINE
            .insert(user_data_lock._id.clone(), UserDataLazyStatic {
                balance: user_data_lock.balance.clone(),
                id: user_data_lock._id.clone(),
            });


        logger::time_end("saveData");
        println!("\n");
    });

    /*
    Обработка отправки сообшения
     */
    let user_info_clone = user_info.clone();
    client.on("message", move |client: SocketRef, Data::<String>(mut msg)| async move {
        let user_info_lock = user_info_clone.lock().await;
        msg = msg.trim().to_string();
        println!("{:?}", msg);

        if msg == "" {
            return;
        }

        if msg.len() > 1_000 {
            return;
        }

        let msg_obj = add_msg(user_info_lock._id.clone(), msg).await;
        _io.emit("message", msg_obj).ok();

        // TODO защита от спама
    });

    let user_info_for_msg = user_info.clone();
    client.on_disconnect(move |client: SocketRef, _reason: DisconnectReason| async move {
        println!("disconnect");
        let _start: Instant = Instant::now();

        let user_data_lock = user_info_for_msg.lock().await;
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
                "$set": { "gameS tats": game_stats }
            }, None,
        ).await;

        let id = user_data_lock._id.clone();
        USERS_ONLINE.remove(&id);

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


async fn send_leaderboard(s: &SocketRef) {
    let conf = get_conf();
    let leaderboard: Vec<LeaderBoardItem> = get_leaderboard(conf.max_mun_of_users2send).await; // Получаем leaderboard как Vec<LeaderBoardItem>
    let serialized_leaderboard = to_string(&leaderboard).expect("Не удалось сериализовать leaderboard");

    s.emit("leaderboard", serialized_leaderboard).ok();
}