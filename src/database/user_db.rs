use std::sync::Arc;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use crate::auth::sha256::hash_password;

pub async fn set_new_user(db: Arc<Collection<Document>>, password: &str, id: &str) -> mongodb::results::InsertOneResult {
    let hashed_password = hash_password(password);
    let result: mongodb::results::InsertOneResult = db
        .insert_one(
            doc! {
                "_id": id,
                "password": hashed_password,
                "gameStats": {
                    "balance": 0,
                    "mpc": { "cost": 50, "amount": 1, "value": 0 },
                    "auto": { "cost": 200, "amount": 0, "value": 0 },
                    "triple": { "cost": 500, "amount": 0, "value": 0 },
                    "superAuto": { "cost": 10000, "amount": 0, "value": 0 },
                    "speed": { "cost": 500000, "unlocked": false, "multiplier": 1 },
                    "reset": { "minCost": 1000000 },
                    "otherUpgrades": {
                        "higherHackAmount": false,
                        "betterFirewall": false,
                        "covertHacks": false,
                        "doubleHacks": false,
                    }
                }
            },
            None,
        )
        .await
        .unwrap();
    result
}


// pub async  fn update_game_stats(db: Arc<Collection<Document>>, new_data) {

// }