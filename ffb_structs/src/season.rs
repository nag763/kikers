//! A season is a MySQL entity that contains a list of bets that user have 
//! made during a given period of time.

use crate::database::Database;
use crate::error::ApplicationError;
use crate::transaction_result::TransactionResult;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use sqlx::{QueryBuilder, FromRow, mysql::MySqlRow};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow, Eq, Hash)]
pub struct Model {
    /// The MySQL ID.
    pub id: u32,
    /// The name of the season.
    pub name: String,
    /// Whether the season is the main one or not.
    ///
    /// The main season is the one that will be used when a bet is registered.
    pub is_main: bool,
    /// Whether the season is closed or not.
    ///
    /// A closed season can't contains more bet than it already has.
    pub is_closed: bool,
}

pub struct Entity;

impl Entity {

    /// Find a season by its id.
    ///
    /// # Arguments
    ///
    /// - id : the season id.
    pub async fn find_by_id(id: u32) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let model: Option<Model> = sqlx::query_as("SELECT * FROM SEASON WHERE id=?")
            .bind(id)
            .fetch_optional(&mut conn)
            .await?;
        Ok(model)
    }

    /// Get the current season's id.
    ///
    /// This method is cached within the redis cache.
    pub async fn get_current_season_id() -> Result<u32, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let cache_result: Option<u32> = redis::cmd("GETEX")
            .arg("current_season_id")
            .arg("EX")
            .arg("300")
            .query(&mut redis_conn)?;
        if let Some(cache_result) = cache_result {
            debug!("The main season id has been gotten from the cache");
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
            debug!("The main season's id has been stored in cache");
            Ok(row.0)
        }
    }

    /// Clears the main season's id cache.
    fn clear_cache_main() -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("DEL")
            .arg("current_season_id")
            .query(&mut conn)?;
        Ok(())
    }

    /// Add a new season within the database.
    ///
    /// # Arguments
    ///
    /// * name : Name of the new season, be aware that the season name has to be
    /// unique within the table.
    pub async fn add_new(name: &str) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query("INSERT INTO SEASON(name) VALUES (?)")
            .bind(&name)
            .execute(&mut conn)
            .await?;
        Self::clear_cache()?;
        info!("The season {} has been successfully added within the databaae", name);
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Close a season.
    ///
    /// Once a season is closed, no bets can be made for that season.
    ///
    /// # Arguments
    ///
    /// - id : the MySQL id of the season to close.
    pub async fn close(id: u32) -> Result<TransactionResult, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query("UPDATE SEASON SET is_closed=1 WHERE id=? AND is_main=0")
            .bind(&id)
            .execute(&mut conn)
            .await?;
        Self::clear_cache()?;
        info!("Season #{} has just been closed", id);
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }

    /// Changes the main season.
    ///
    /// Only one main season can be set within the database.
    ///
    /// # Arguments
    ///
    /// - id : the MySQL id of the season to set as main.
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

    /// Clears the cache.
    pub(crate) fn clear_cache() -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg(r#"seasons::*"#).query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }
        debug!("The cache for the seasons has been cleared");
        Ok(())
    }
}

#[derive(Default, Hash, Debug)]
pub struct EntityBuilder {
    /// Whether to include or exclude the main season.
    is_main: Option<bool>,
    /// Whether to include or exclude the closed seasons.
    is_closed: Option<bool>,
}

impl EntityBuilder {
    pub fn build() -> EntityBuilder {
        Self::default()
    }

    /// Whether to include or exclude the main season.
    pub fn is_main(&mut self, is_main: Option<bool>) -> &mut Self {
        self.is_main = is_main;
        self
    }

    /// Whether to include or exclude the closed seasons.
    pub fn is_closed(&mut self, is_closed: Option<bool>) -> &mut Self {
        self.is_closed = is_closed;
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
            debug!("The season has been found in cache and will be returned from it");
            Ok(serde_json::from_str(&cache_result)?)
        } else {
            debug!("The season hasn't been found in cache and will be queried");
            let mut conn = Database::acquire_sql_connection().await?;
            let mut query_builder = QueryBuilder::new("SELECT * FROM SEASON");
            if self.is_closed.is_some() || self.is_main.is_some() {
                query_builder.push("\nWHERE");
                if let Some(is_closed) = self.is_closed {
                    query_builder.push("\n\tis_closed=")
                        .push_bind(is_closed);
                }
                if let Some(is_main) = self.is_main {
                    query_builder.push("\n\tis_main=")
                        .push_bind(is_main);
                }
            }
            query_builder.push("\nRDER BY is_main DESC, name DESC");
            let rows: Vec<MySqlRow> = query_builder.build().fetch_all(&mut conn).await?;
            let mut models: Vec<Model> = Vec::with_capacity(rows.len());
            for row in rows {
                models.push(Model::from_row(&row)?);
            }
            redis::cmd("SET")
                .arg(&redis_key)
                .arg(serde_json::to_string(&models)?)
                .arg("EX")
                .arg("300")
                .query(&mut redis_conn)?;
            debug!("The seasons have been found with success and stored within the cache");
            Ok(models)
        }
    }
}
