use crate::database::Database;
use crate::error::ApplicationError;
use crate::transaction_result::TransactionResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow, Eq, Hash)]
pub struct Model {
    pub id: u32,
    pub name: String,
    pub is_main: bool,
    pub is_closed: bool,
}

pub struct Entity;

impl Entity {
    pub async fn get_all() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let models: Vec<Model> = sqlx::query_as("SELECT * FROM SEASON ORDER BY name DESC")
            .fetch_all(&mut conn)
            .await?;
        Ok(models)
    }

    pub async fn get_all_open() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let models: Vec<Model> =
            sqlx::query_as("SELECT * FROM SEASON WHERE is_closed=0 ORDER BY name DESC")
                .fetch_all(&mut conn)
                .await?;
        Ok(models)
    }

    pub async fn get_current_season_id() -> Result<u32, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let cache_result: Option<u32> = redis::cmd("GETEX")
            .arg("current_season_id")
            .arg("EX")
            .arg("300")
            .query(&mut redis_conn)?;
        if let Some(cache_result) = cache_result {
            Ok(cache_result)
        } else {
            let mut conn = Database::acquire_sql_connection().await?;
            let row: (u32,) =
                sqlx::query_as("SELECT id FROM SEASON WHERE is_closed=0 AND is_main=1 LIMIT 1")
                    .fetch_one(&mut conn)
                    .await?;
            redis::cmd("SET")
                .arg("current_season_id")
                .arg(row.0)
                .arg("EX")
                .arg("300")
                .query(&mut redis_conn)?;
            Ok(row.0)
        }
    }

    fn clear_cache() -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("DEL")
            .arg("current_season_id")
            .query(&mut conn)?;
        Ok(())
    }

    pub async fn add_new(name: &str) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query("INSERT INTO SEASON(name) VALUES (?)")
            .bind(&name)
            .execute(&mut conn)
            .await?;
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    pub async fn close(id: u32) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query("UPDATE SEASON SET is_closed=1 WHERE id=? AND is_main=0")
            .bind(&id)
            .execute(&mut conn)
            .await?;
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    pub async fn set_main(id: u32) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        sqlx::query("UPDATE SEASON SET is_main =(id=?)")
            .bind(&id)
            .execute(&mut conn)
            .await?;
        Self::clear_cache()?;
        Ok(())
    }
}
