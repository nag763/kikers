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
        Ok(())
    }
}
