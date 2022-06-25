use crate::database::Database;
use crate::error::ApplicationError;
use crate::transaction_result::TransactionResult;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash, sqlx::Type)]
#[repr(u32)]
pub enum Bet {
    Win = 1,
    Draw = 2,
    Loss = 3,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    pub user_id: u32,
    pub fixture_id: u32,
    pub bet_id: Bet,
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
        bet_id: Bet,
        stake: f32,
    ) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        println!("Bet : {:?}", bet_id);
        let result = sqlx::query(
            "INSERT INTO USER_BET(user_id, fixture_id, bet_id, stake) VALUES(?,?,?,?) ON DUPLICATE KEY UPDATE bet_id=?, stake=?",
        )
        .bind(user_id)
        .bind(fixture_id)
        .bind(bet_id)
        .bind(stake)
        .bind(bet_id)
        .bind(stake)
        .execute(&mut conn)
        .await?;
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }
}
