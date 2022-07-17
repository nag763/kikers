use crate::database::Database;
use crate::error::ApplicationError;
use crate::transaction_result::TransactionResult;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow, Eq, Hash)]
pub struct Model {
    pub id: u32,
    pub name: String,
    pub is_main: bool,
    pub is_closed: bool,
}

pub struct Entity;

impl Entity {
    pub async fn find_by_id(id: u32) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model: Option<Model> = sqlx::query_as("SELECT * FROM SEASON WHERE id=?")
            .bind(id)
            .fetch_optional(&mut conn)
            .await?;
        Ok(model)
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

    fn clear_cache_main() -> Result<(), ApplicationError> {
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
        Self::clear_cache()?;
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
        Self::clear_cache()?;
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
        Self::clear_cache_main()?;
        Self::clear_cache()?;
        Ok(())
    }

    pub(crate) fn clear_cache() -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg(r#"seasons::*"#).query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }

        Ok(())
    }
}

#[derive(Default, Hash, Debug)]
pub struct EntityBuilder {
    open_only: bool,
}

impl EntityBuilder {
    pub fn build() -> EntityBuilder {
        Self::default()
    }

    pub fn open_only<'a>(&'a mut self, open_only: bool) -> &'a mut Self {
        self.open_only = open_only;
        self
    }

    pub async fn finish(&self) -> Result<Vec<Model>, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let redis_key: String = format!("seasons::{:x}", hasher.finish());
        let cache_result: Option<String> = redis::cmd("GETEX")
            .arg(&redis_key)
            .arg("EX")
            .arg("300")
            .query(&mut redis_conn)?;
        if let Some(cache_result) = cache_result {
            Ok(serde_json::from_str(&cache_result)?)
        } else {
            let mut conn = Database::acquire_sql_connection().await?;
            let statement = if self.open_only {
                sqlx::query_as(
                    "SELECT * FROM SEASON WHERE is_closed=0 ORDER BY is_main DESC, name DESC",
                )
            } else {
                sqlx::query_as("SELECT * FROM SEASON ORDER BY is_main DESC, name DESC")
            };
            let models: Vec<Model> = statement.fetch_all(&mut conn).await?;
            redis::cmd("SET")
                .arg(&redis_key)
                .arg(serde_json::to_string(&models)?)
                .arg("EX")
                .arg("300")
                .query(&mut redis_conn)?;
            Ok(models)
        }
    }
}
