use mongodb::bson::{doc, Document};
use mongodb::{Collection, IndexModel};
use mongodb::options::FindOptions;
use futures::stream::StreamExt;
use crate::leaderboard::main::{LEADERBOARD, LEADERBOARD_MAP, LeaderBoardItem, MAX_ARR_LEN};

pub async fn main(collection: &Collection<Document>) -> mongodb::error::Result<()> {
    let index_model = IndexModel::builder()
        .keys(doc! {"gameStats.balance": -1})
        .build();

    let index_name = collection.create_index(index_model, None).await;

    let sort = doc! {"gameStats.balance": -1};
    let find_options = FindOptions::builder().sort(sort).limit(MAX_ARR_LEN as i64).build();

    let mut cursor = collection.find(None, find_options).await?;

    let mut leaderboard_lock = LEADERBOARD.lock().await;
    let mut leaderboard_map_lock = LEADERBOARD_MAP.lock().await;

    let mut i: usize = 0;

    while let Some(result) = cursor.next().await {
        match result {
            Ok(doc) => {
                if let Ok(id) = doc.get_str("_id") {
                    if let Ok(game_stats) = doc.get_document("gameStats") {
                        if let Ok(balance) = game_stats.get_i64("balance") {
                            leaderboard_lock.push(LeaderBoardItem { id: id.to_string(), balance });
                            leaderboard_map_lock.insert(id.to_string(), i);
                            i += 1;
                        } else {
                            eprintln!("Ошибка при получении balance");
                        }
                    } else {
                        eprintln!("Ошибка при получении gameStats");
                    }
                } else {
                    eprintln!("Ошибка при получении id");
                }
            }
            Err(e) => return Err(e)
        }
    }

    println!("leaderboard_lock: {:?}", leaderboard_lock);

    println!(" - indexed {:?}\n\n", index_name);

    Ok(())
}