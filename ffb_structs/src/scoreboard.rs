//! The scoreboard is the ranked list of users for a season.
//!
//! This list is defined by the bets they have made and how good they predicted
//! given the final results of the games.
//!
//! A scoreboard is associed to a season, or can be all time.
//!
//! Given the time complexity to display a scoreboard, it is important to cache
//! it.

use crate::database::Database;
use crate::error::ApplicationError;
use crate::{scoreboard_entry::Model as ScoreEntry, season, season::Model as Season};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, FromRow, QueryBuilder};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    /// The season associed to the scoreboard.
    pub season: Option<Season>,
    /// The entries of the scoreboard.
    pub score_entries: Vec<ScoreEntry>,
}

pub(crate) struct Entity;

impl Entity {
    /// Clears the cache.
    pub(crate) fn clear_cache() -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys_to_del: Vec<String> =
            redis::cmd("KEYS").arg(r#"scoreboard:*"#).query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }
        debug!("The scoreboard's cache has been cleared");

        Ok(())
    }
}

#[derive(Default, Hash, Debug)]
pub struct EntityBuilder {
    season_id: Option<u32>,
    all_time: bool,
    limit: Option<u32>,
}

impl EntityBuilder {
    pub fn build() -> EntityBuilder {
        Self::default()
    }

    /// The season id to look up for.
    ///
    /// If none is passed, all the season ids will be looked up for.
    pub fn season_id(&mut self, season_id: Option<u32>) -> &mut Self {
        self.season_id = season_id;
        self.all_time = false;
        self
    }

    /// Whether to look for all time or not.
    pub fn all_time(&mut self, all_time: bool) -> &mut Self {
        self.all_time = all_time;
        self.season_id = None;
        self
    }

    /// Limit the results to the 0..nth entries.
    pub fn limit(&mut self, limit: Option<u32>) -> &mut Self {
        self.limit = limit;
        self
    }

    pub async fn finish(&self) -> Result<Model, ApplicationError> {
        let mut redis_conn = Database::acquire_redis_connection()?;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let redis_key: String = format!("scoreboard::{:x}", hasher.finish());
        let cache_result: Option<String> = redis::cmd("GETEX")
            .arg(&redis_key)
            .arg("EX")
            .arg("300")
            .query(&mut redis_conn)?;
        if let Some(cache_result) = cache_result {
            debug!("The requested scoreboard has been found in the cache");
            Ok(serde_json::from_str(&cache_result)?)
        } else {
            debug!("The requested scoreboard hasn't been found in the cache, looking up in the database");
            let mut conn = Database::acquire_sql_connection().await?;
            let season_id: Option<u32> = match (self.season_id, self.all_time) {
                (Some(v), false) => Some(v),
                (None, true) => None,
                _ => Some(season::Entity::get_current_season_id().await?),
            };
            let mut query_builder = QueryBuilder::new("SELECT ub.user_id, usr.name as `user_name`, IF(SUM(outcome) IS NULL, 0, SUM(outcome)) AS `points`, COUNT(*) AS `bets_made`, TRUNCATE(IF(SUM(OUTCOME) IS NULL, 0, SUM(OUTCOME))/COUNT(*),2) as `ppb`");
            query_builder.push("\nFROM `USER_BET`ub INNER JOIN USER usr ON ub.user_id = usr.id");
            if let Some(season_id) = season_id {
                query_builder
                    .push("\nWHERE season_id=")
                    .push_bind(season_id);
            }
            query_builder.push("\nGROUP BY user_id");
            query_builder.push("\nORDER BY points DESC");
            if let Some(limit) = self.limit {
                query_builder.push("\nLIMIT ").push_bind(limit);
            }
            let rows: Vec<MySqlRow> = query_builder.build().fetch_all(&mut conn).await?;
            let mut score_entries: Vec<ScoreEntry> = Vec::with_capacity(rows.len());
            for row in rows {
                score_entries.push(ScoreEntry::from_row(&row)?);
            }
            let season = match season_id {
                Some(v) => season::Entity::find_by_id(v).await?,
                None => None,
            };
            let model: Model = Model {
                season,
                score_entries,
            };

            redis::cmd("SET")
                .arg(&redis_key)
                .arg(serde_json::to_string(&model)?)
                .arg("EX")
                .arg("300")
                .query(&mut redis_conn)?;
            debug!("The scoreboard has been cached within the redis cache");
            Ok(model)
        }
    }
}
