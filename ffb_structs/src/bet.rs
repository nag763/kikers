use crate::database::Database;
use crate::error::ApplicationError;
use crate::transaction_result::TransactionResult;
use crate::game;
use serde::{Deserialize, Serialize};
use mongodb::bson::doc;
use chrono::{Utc, DateTime};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash, sqlx::Type)]
#[repr(u32)]
pub enum GameResult {
    Win = 1,
    Draw = 2,
    Loss = 3,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    pub user_id: u32,
    pub fixture_id: u32,
    pub result_id: GameResult,
    pub stake: f32,
    pub outcome: Option<u32>,
}

pub struct Entity;

impl Entity {

    pub async fn upsert_bet(
        user_id: u32,
        fixture_id: u32,
        game_result: GameResult,
        stake: f32,
    ) -> Result<TransactionResult, ApplicationError> {
        let now: DateTime<Utc> = Utc::now();
        let database = Database::acquire_mongo_connection().await?;
        let mongo_result = database
            .collection::<Model>("fixture")
            .update_one(
                doc! {"fixture.id":fixture_id, "betters.user_id": user_id, "fixture.timestamp": {"$gte":now.timestamp()}},
                doc! {"$set": {"betters.$.game_result": bson::to_bson(&game_result)?}},
                None
            ).await?;
        if mongo_result.modified_count == 0 {
            let mongo_result = database
            .collection::<Model>("fixture") 
            .update_one(
                doc! {"fixture.id":fixture_id,  "fixture.timestamp": {"$gte":now.timestamp()}},
                doc! {"$addToSet": {"betters" :{"user_id": user_id, "game_result": bson::to_bson(&game_result)?}}},
                None
            ).await?;
            if mongo_result.matched_count == 0 {
                return Err(ApplicationError::FormOutdated);
            }
        }
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query(
            "INSERT INTO USER_BET(user_id, fixture_id, result_id, stake) VALUES(?,?,?,?) ON DUPLICATE KEY UPDATE result_id=?, stake=?",
        )
        .bind(user_id)
        .bind(fixture_id)
        .bind(&game_result)
        .bind(stake)
        .bind(&game_result)
        .bind(stake)
        .execute(&mut conn)
        .await?;
        game::Entity::clear_cache()?;
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }
}
