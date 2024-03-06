use std::collections::HashMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};

lazy_static! {
    pub static ref LEADERBOARD: Mutex<Vec<LeaderBoardItem>> = Mutex::new(Vec::new());
}

lazy_static! {
    pub static ref LEADERBOARD_MAP: Mutex<HashMap<String, usize>> = Mutex::new(HashMap::new());
}

pub const MAX_ARR_LEN: usize = 10;

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
    return leaderboard_lock.clone();
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
    println!(" --  update_leaderboard_user_pos -1");

    let mut leaderboard_lock = LEADERBOARD.lock().await;
    let mut leaderboard_map_lock = LEADERBOARD_MAP.lock().await;
    let last_user = leaderboard_lock[leaderboard_lock.len() - 1].clone();

    let user_balance = user_data.balance.clone();

    if leaderboard_lock.len() < MAX_ARR_LEN {
        println!(" --  в доске есть незаполненое место");
        // если в доске есть незаполненое место

        if last_user.balance > user_balance {
            // добавить в конец
            leaderboard_lock.push(user_data.clone());
            leaderboard_map_lock.insert(user_data.id.clone(), leaderboard_lock.len());
        }

        let mut user_pos_in_leaderboard = leaderboard_map_lock.get(&user_data.id);
        let new_pos = find_user_pos(&user_balance, &leaderboard_lock).await;

        if let Some(pos) = user_pos_in_leaderboard {
            // пользователь уже в списке
            println!(" --  пользователь уже в списке");

            if *pos == new_pos {
                let old_balance = leaderboard_lock[new_pos].balance;
                leaderboard_lock[new_pos].balance = user_balance;
                println!(" --  обновлять позицию не требуется, old_balance({:?}), new({:?})", old_balance, user_balance);
                return;
            }
        } else {
            // пользователь еще не в списке
            println!(" --  пользователь еще не в списке");
        }

        // обновить инфу
        leaderboard_lock.insert(new_pos, user_data.clone());
        leaderboard_map_lock.insert(user_data.id.clone(), new_pos);

        return;
    }


    if last_user.balance > user_balance {
        // у нового пользователя слишком малый баланс
        println!(" --  у нового пользователя слишком малый баланс");
        return;
    }


    let user_in_leaderboard = leaderboard_map_lock.get_mut(&user_data.id);

    let new_pos = find_user_pos(&user_balance, &leaderboard_lock).await;

    // если пользователь в leaderboard
    if user_in_leaderboard.is_some() {
        let curr_pos = user_in_leaderboard.unwrap();
        println!(" --  пользователь в leaderboard, curr_pos = {:?}, new pos = {:?}", curr_pos, new_pos);

        if *curr_pos == new_pos {
            // обновлять позицию не требуется
            println!(" --  обновлять позицию не требуется");
            return;
        }

        // обновляем позицию
        leaderboard_lock.remove(*curr_pos);
        leaderboard_lock.insert(new_pos, user_data.clone());
    } else {
        // если пользователь еше не в списке
        println!(" --  пользователь еше не в списке");
        leaderboard_lock.insert(new_pos, user_data.clone());

        // убираем последнего пользователя
        leaderboard_lock.pop();
        leaderboard_map_lock.remove(&last_user.id);
    }

    // обновляем инфу о позиции пользователя в map
    *leaderboard_map_lock.get_mut(&user_data.id).unwrap() = new_pos;
    return;
}

async fn find_user_pos(target: &i64, leaderboard_lock: &MutexGuard<'_, Vec<LeaderBoardItem>>) -> usize {
    if leaderboard_lock.len() == 0 {
        return 0;
    }

    if leaderboard_lock.len() == 1 {
        let last_user = &leaderboard_lock[0];

        if last_user.balance < *target {
            return 0;
        } else {
            return 1;
        }
    }

    let mut min: usize = 0;
    let mut max: usize = leaderboard_lock.len() as usize - 1;

    while min <= max {
        let guess = ((min + max) / 2) as usize;

        if leaderboard_lock[guess].balance == *target {
            return guess;
        } else if leaderboard_lock[guess].balance > *target {
            min = guess + 1;
        } else {
            if guess == 0 {
                break;
            }

            max = guess - 1;
        }
    }

    return min;
}