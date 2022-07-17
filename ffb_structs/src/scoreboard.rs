use crate::database::Database;
use crate::error::ApplicationError;
use crate::{season, season::Model as Season, scoreboard_entry::Model as ScoreEntry};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    pub season: Option<Season>,
    pub score_entries: Vec<ScoreEntry>,
}

pub(crate) struct Entity;

impl Entity {
    pub(crate) fn clear_cache() -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys_to_del: Vec<String> =
            redis::cmd("KEYS").arg(r#"scoreboard:*"#).query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }

        Ok(())
    }
}

#[derive(Default, Hash, Debug)]
pub struct EntityBuilder {
    season_id: Option<u32>,
    all_time: bool,
}

impl EntityBuilder {
    pub fn build() -> EntityBuilder {
        Self::default()
    }

    pub fn season_id<'a>(&'a mut self, season_id: u32) -> &'a mut Self {
        self.season_id = Some(season_id);
        self.all_time = false;
        self
    }

    pub fn all_time<'a>(&'a mut self, all_time: bool) -> &'a mut Self {
        self.all_time = all_time;
        self.season_id = None;
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
            Ok(serde_json::from_str(&cache_result)?)
        } else {
            let mut conn = Database::acquire_sql_connection().await?;
            let season_id: Option<u32> = match (self.season_id, self.all_time) {
                (Some(v), false) => Some(v),
                (None, true) => None,
                _ => Some(season::Entity::get_current_season_id().await?),
            };
            let statement = match season_id {
                Some(v) => sqlx::query_as("SELECT ub.user_id, usr.name as `user_name`, IF(SUM(outcome) IS NULL, 0, SUM(outcome)) AS `points`, COUNT(*) AS `bets_made`, TRUNCATE(IF(SUM(OUTCOME) IS NULL, 0, SUM(OUTCOME))/COUNT(*),2) as `ppb`
FROM `USER_BET`ub INNER JOIN USER usr ON ub.user_id = usr.id 
WHERE season_id=?                                
GROUP BY user_id 
ORDER BY points DESC;").bind(v),
                None => sqlx::query_as("SELECT ub.user_id, usr.name AS `user_name`, IF(SUM(outcome) IS NULL, 0, SUM(outcome)) AS `points`, COUNT(*) AS `bets_made`, TRUNCATE(IF(SUM(OUTCOME) IS NULL, 0, SUM(OUTCOME))/COUNT(*),2) as `ppb`
FROM `USER_BET`ub INNER JOIN USER usr ON ub.user_id = usr.id 
GROUP BY user_id 
ORDER BY points DESC;")
            };
            let score_entries: Vec<ScoreEntry> = statement.fetch_all(&mut conn).await?;
            let season = match season_id {
                Some(v) => season::Entity::find_by_id(v).await?,
                None => None
            };
            let model : Model = Model { season, score_entries }; 

            redis::cmd("SET")
                .arg(&redis_key)
                .arg(serde_json::to_string(&model)?)
                .arg("EX")
                .arg("300")
                .query(&mut redis_conn)?;
            Ok(model)
        }
    }
}
