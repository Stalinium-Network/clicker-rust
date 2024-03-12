use std::sync::Arc;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use crate::auth::sha256::hash_password;

pub async fn set_new_user(db: &Arc<Collection<Document>>, password: &str, id: &str) -> mongodb::results::InsertOneResult {
    let hashed_password = hash_password(&password);
    let result: mongodb::results::InsertOneResult = db
        .insert_one(
            doc! {
                "_id": id,
                "password": hashed_password,
                "gameStats": {
                    "balance":  "0",
                    "mpc": { "cost": 50i64, "amount": 1i64, "value": 0i64 },
                    "auto": { "cost": 200i64, "amount": 0i64, "value": 0i64 },
                    "triple": { "cost": 500i64, "amount": 0i64, "value": 0i64 },
                    "superAuto": { "cost": 10000i64, "amount": 0i64, "value": 0i64 },
                    "speed": { "cost": 500000i64, "unlocked": false, "multiplier": 1i64 },
                    "reset": { "minCost": 1000000i64 },
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

