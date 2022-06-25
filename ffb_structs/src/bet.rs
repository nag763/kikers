use crate::database::Database;
use crate::error::ApplicationError;
use crate::transaction_result::TransactionResult;
use crate::game;
use serde::{Deserialize, Serialize};
use mongodb::bson::doc;

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
    pub async fn get_bets() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let models: Vec<Model> = sqlx::query_as("SELECT * FROM USER_BET")
            .fetch_all(&mut conn)
            .await?;
        Ok(models)
    }

    pub async fn upsert_bet(
        user_id: u32,
        fixture_id: u32,
        game_result: GameResult,
        stake: f32,
    ) -> Result<TransactionResult, ApplicationError> {
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
        let database = Database::acquire_mongo_connection().await?;
        let mongo_result = database
            .collection::<Model>("fixture")
            .update_one(
                doc! {"fixture.id":fixture_id, "betters.user_id": user_id},
                doc! {"$set": {"betters.$.game_result": bson::to_bson(&game_result)?}},
                None
            ).await?;
        if mongo_result.modified_count == 0 {
            database               
            .collection::<Model>("fixture") 
            .update_one(
                doc! {"fixture.id":fixture_id},
                doc! {"$addToSet": {"betters" :{"user_id": user_id, "game_result": bson::to_bson(&game_result)?}}},
                None
            ).await?;
            
        }
        game::Entity::clear_cache()?;
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }
}
