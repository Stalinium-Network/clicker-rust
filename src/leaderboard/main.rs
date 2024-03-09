use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};
use dashmap::DashMap;

lazy_static! {
    pub static ref LEADERBOARD: Mutex<Vec<LeaderBoardItem>> = Mutex::new(Vec::new());
}

lazy_static! {
    pub static ref LEADERBOARD_MAP: DashMap<String, usize> = DashMap::new();
}

pub const MAX_ARR_LEN: usize = 10;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeaderBoardItem {
    pub id: String,
    pub balance: i64,
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

pub async fn update_leaderboard_user_pos(user_data: LeaderBoardItem, old_balance: &i64, new_balance: &i64) {
    if old_balance == new_balance {
        println!("обновление не пребуется (same balance)");
        return;
    }

    let mut leaderboard_lock = LEADERBOARD.lock().await;
    let last_user = leaderboard_lock[leaderboard_lock.len() - 1].clone();

    let user_balance = user_data.balance.clone();

    if leaderboard_lock.len() < MAX_ARR_LEN {
        println!(" --  в доске есть незаполненое место");
        // если в доске есть незаполненое место
        let user_pos_in_leaderboard = LEADERBOARD_MAP.get(&user_data.id);


        if last_user.balance > user_balance {
            // добавить в конец
            /*
                TODO проверить что пользователь уже в списке
            */

            if last_user.id == user_data.id {
                // обновление не требуется, пользователь и так в конце списка
                println!(" --  обновление не требуется, пользователь и так в конце списка");
                leaderboard_lock.last_mut().unwrap().balance = user_balance;
                return;
            }

            println!("добавить в конец, last user balance ({}) -- user balance ({})", last_user.balance, user_balance);

            if let Some(user_pos) = user_pos_in_leaderboard {
                leaderboard_lock.remove(*user_pos);
            }

            LEADERBOARD_MAP.insert(user_data.id.clone(), leaderboard_lock.len());
            leaderboard_lock.push(user_data.clone());
            return;
        }

        let new_pos = find_user_pos(&user_balance, &leaderboard_lock).await;

        if let Some(pos) = user_pos_in_leaderboard {
            // пользователь уже в списке
            println!(" --  пользователь уже в списке");

            if *pos == new_pos {
                leaderboard_lock[new_pos].balance = user_balance;
                println!(" --  обновлять позицию не требуется, old_balance({:?}), new({:?})", old_balance, user_balance);
                return;
            } else {
                // Удаляем старую запись пользователя из вектора
                if old_balance > new_balance && (new_pos - *pos) == 1 {
                    // если баланс уменьшился и разница в новой позиции равна 1, значит позиция не ихменилась (особенность binary search)
                    leaderboard_lock[new_pos].balance = user_balance;
                    return;
                } else {
                    println!(" -- [debug] | обновление позиции ( *pos ({:?}), new_pos ({:?}) )", *pos, new_pos);

                    {
                        let more_or_less = if old_balance < new_balance { '<' } else { '>' };
                        println!(" -- [debug] | old_b {:?} new_b ({:?} - {:?})", more_or_less, old_balance, new_balance);

                        let u2rm = leaderboard_lock[new_pos].clone();
                        println!("  ==       u2rm [> {:?}", u2rm);

                        let u_on_old_pos = leaderboard_lock[*pos].clone();
                        println!("  == u_old_pos  [> {:?}", u_on_old_pos);

                        println!("  == u_data     [> {:?}", user_data);
                    }

                    leaderboard_lock.remove(*pos);
                }
            }
        } else {
            // пользователь еще не в списке
            println!(" --  [error] пользователь еще не в списке");
            print!("{:?}", user_pos_in_leaderboard);
        }

        // обновить инфу
        println!(" --  обновить инфу");

        leaderboard_lock.insert(new_pos, user_data.clone());
        LEADERBOARD_MAP.insert(user_data.id.clone(), new_pos);

        return;
    }


    if last_user.balance > user_balance {
        // у нового пользователя слишком малый баланс
        println!(" --  у нового пользователя слишком малый баланс");
        return;
    }


    let new_pos = find_user_pos(&user_balance, &leaderboard_lock).await;

    // если пользователь в leaderboard
    if let Some(pos_ref) = LEADERBOARD_MAP.get(&user_data.id) {
        println!(" --  пользователь в leaderboard, curr_pos = {:?}, new pos = {:?}", *pos_ref, new_pos);


        if *pos_ref == new_pos {
            // обновлять позицию не требуется
            println!(" --  обновлять позицию не требуется");
            return;
        }

        // Удаляем старую запись пользователя из вектора
        leaderboard_lock.remove(*pos_ref);

        // Вставляем пользователя на новую позицию
        leaderboard_lock.insert(new_pos, user_data.clone());

        // Обновляем позицию пользователя в DashMap
        LEADERBOARD_MAP.insert(user_data.id.clone(), new_pos);
    } else {
        // если пользователь еше не в списке
        println!(" --  пользователь еше не в списке");
        leaderboard_lock.insert(new_pos, user_data.clone());

        // убираем последнего пользователя
        leaderboard_lock.pop();
        LEADERBOARD_MAP.remove(&last_user.id);
    }

    // обновляем инфу о позиции пользователя в map
    LEADERBOARD_MAP.insert(user_data.id, new_pos);
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