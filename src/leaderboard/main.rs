use std::collections::HashMap;
use std::ffi::c_void;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

lazy_static! {
    pub static ref LEADERBOARD: Mutex<Vec<LeaderBoardItem>> = Mutex::new(Vec::new());
}

lazy_static! {
    static ref LEADERBOARD_MAP: Mutex<HashMap<String, usize>> = Mutex::new(HashMap::new());
}

const MAX_ARR_LEN: usize = 10;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeaderBoardItem {
    pub id: String,
    pub balance: i64,
}

struct BinarySearchReturn {
    guess: usize,
}

pub async fn get_leaderboard() -> Vec<LeaderBoardItem> {
    let leaderboard_lock = LEADERBOARD.lock().await;
    let leaderboard_lock_clone = leaderboard_lock.clone();

    return leaderboard_lock_clone;
}

// вызывать только при регистрании
pub async fn req_add_user2leaderboard(user_data: LeaderBoardItem) {
    // если свободное место еше есть
    let mut leaderboard_lock = LEADERBOARD.lock().await;

    if leaderboard_lock.len() < MAX_ARR_LEN {
        leaderboard_lock.push(user_data)
    }
}

pub async fn update_leaderboard_user_pos(user_data: LeaderBoardItem) {
    println!("update_leaderboard_user_pos -1");

    let mut leaderboard_lock = LEADERBOARD.lock().await;
    let user_balance = user_data.balance.clone();

    if leaderboard_lock.len() < MAX_ARR_LEN {
        // если в доске есть незаполненое место
        let new_pos = find_user_pos(&user_balance).await;
        if new_pos == (leaderboard_lock.len() - 1) {
            // добавить пользователя в конец
            leaderboard_lock.push(user_data);

            return;
        }

        println!("update_leaderboard_user_pos -2");
        leaderboard_lock.insert(new_pos, user_data.clone());

        return;
    }

    let last_user = leaderboard_lock[leaderboard_lock.len() - 1].clone();

    if last_user.balance > user_balance {
        // у нового пользователя слишком малый баланс
        return;
    }

    let mut leaderboard_map_lock = LEADERBOARD_MAP.lock().await;
    let mut user_in_leaderboard = leaderboard_map_lock.get_mut(&user_data.id);

    let new_pos = find_user_pos(&user_balance).await;

    // если пользователь в leaderboard
    if user_in_leaderboard.is_some() {
        let curr_pos = user_in_leaderboard.unwrap();

        if *curr_pos == new_pos {
            // обновлять позицию не требуется
            return;
        }

        // обновляем позицию
        leaderboard_lock.remove(*curr_pos);
        leaderboard_lock.insert(new_pos, user_data.clone());
    } else {
        // если пользователь еше не в списке
        leaderboard_lock.insert(new_pos, user_data.clone());

        // убираем последнего пользователя
        leaderboard_lock.pop();
        leaderboard_map_lock.remove(&last_user.id);
    }
    // обновляем инфу о позиции пользователя в map
    *leaderboard_map_lock.get_mut(&user_data.id).unwrap() = new_pos;

    return;
}

async fn find_user_pos(target: &i64) -> usize {
    let leaderboard_lock = LEADERBOARD.lock().await;

    let mut min: usize = 0;
    let mut max: usize = leaderboard_lock.len() as usize - 1;

    while min <= max {
        let guess = ((min + max) / 2) as usize;

        if leaderboard_lock[guess].balance == *target {
            return guess;
        } else if leaderboard_lock[guess].balance > *target {
            min = guess + 1;
        } else {
            max = guess - 1;
        }
    }

    return min;
}