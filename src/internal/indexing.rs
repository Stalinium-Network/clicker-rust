use mongodb::bson::{doc, Document};
use mongodb::{Collection, IndexModel};
use mongodb::options::{AggregateOptions};
use futures::stream::StreamExt;
use crate::internal::conf::main::get_conf;
use crate::internal::logger;
use crate::leaderboard::main::{LEADERBOARD, LEADERBOARD_MAP, LeaderBoardItem};

// загрузка пользователей в leaderboard при старте сервера
pub async fn main(collection: &Collection<Document>) -> mongodb::error::Result<()> {
    logger::time("indexing leaderboard");
    let index_model = IndexModel::builder()
        .keys(doc! {"gameStats.balance": -1})
        .build();

    let _ = collection.create_index(index_model, None).await;

    let config = get_conf();

    let pipeline = vec![
        doc! {
        "$addFields": {
            "numericBalance": {
                "$toDecimal": "$gameStats.balance"
            }
        }
    },
        doc! {
        "$sort": {
            "numericBalance": -1
        }
    },
        doc! {
        "$limit": config.max_leaderboard_arr as i64
    },
    ];

    let aggregate_options = AggregateOptions::builder().build();

    let mut cursor = collection.aggregate(pipeline, aggregate_options).await?;

    let mut i: usize = 0;

    let mut leaderboard_lock = LEADERBOARD.lock().await;
    leaderboard_lock.clear();
    LEADERBOARD_MAP.clear();


    while let Some(result) = cursor.next().await {
        match result {
            Ok(doc) => {
                if let Ok(id) = doc.get_str("_id") {
                    if let Ok(game_stats) = doc.get_document("gameStats") {
                        if let Ok(balance_str) = game_stats.get_str("balance") {
                            if let Ok(balance) = balance_str
                                .parse::<u128>()
                            {
                                leaderboard_lock.push(LeaderBoardItem { id: id.to_string(), balance });
                                LEADERBOARD_MAP.insert(id.to_string(), i);
                                i += 1;
                            } else {
                                eprintln!("Ошибка при получении balance");
                            }
                        } else {
                            eprintln!("Ошибка при получении balance_str");
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

    logger::time_end("indexing leaderboard");
    println!(" [info] - leaderboard loading finished");
    Ok(())
}
