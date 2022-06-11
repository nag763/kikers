use crate::common_api_structs::{Fixture, Goals, Score, Teams};
use crate::database::Database;
use crate::error::ApplicationError;
use crate::league::Model as League;
use crate::transaction_result::TransactionResult;
use futures::TryStreamExt;
use mongodb::bson::doc;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Model {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bet: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_odds: Option<bool>,
    #[serde(rename = "localLeagueLogo", skip_serializing_if = "Option::is_none")]
    pub league_local_logo : Option<String>,
    #[serde(rename = "localHomeLogo", skip_serializing_if = "Option::is_none")]
    pub home_local_logo: Option<String>,
    #[serde(rename = "localAwayLogo", skip_serializing_if = "Option::is_none")]
    pub away_local_logo: Option<String>,
    pub fixture: Fixture,
    pub league: League,
    pub teams: Teams,
    pub goals: Goals,
    pub score: Score,
}

pub struct Entity;

impl Entity {
    pub async fn find_all_for_date(
        date: &str,
        fav_leagues: Option<Vec<u32>>,
        fav_clubs: Option<Vec<u32>>,
        limit: Option<i64>,
    ) -> Result<Vec<Model>, ApplicationError> {
        let redis_key: String = format!(
            "games:{}::{:?}::{:?}::{:?}",
            date, fav_leagues, fav_clubs, limit
        );
        let mut conn = Database::acquire_redis_connection()?;
        let cached_struct: Option<String> =
            redis::cmd("GET").arg(redis_key.as_str()).query(&mut conn)?;
        if let Some(cached_struct) = cached_struct {
            let deserialized_struct: Vec<Model> = serde_json::from_str(cached_struct.as_str())?;
            redis::cmd("EXPIRE")
                .arg(redis_key.as_str())
                .arg(200)
                .query(&mut conn)?;
            return Ok(deserialized_struct);
        } else {
            let database = Database::acquire_mongo_connection().await?;
            let options: Option<mongodb::options::FindOptions> = match limit {
                Some(v) => Some(
                    mongodb::options::FindOptions::builder()
                        .limit(Some(v))
                        .build(),
                ),
                None => None,
            };
            let mut key: bson::Document = bson::Document::new();
            key.insert("fixture.date", doc! {"$regex" : date});
            match (fav_leagues, fav_clubs) {
                (Some(leagues), Some(clubs)) => {
                    key.insert(
                        "$or",
                        vec![
                            doc! {"teams.home.id" : {"$in" : &clubs}},
                            doc! {"teams.away.id" : {"$in" : &clubs}},
                            doc! {"league.id": doc! {"$in": leagues}},
                        ],
                    );
                }
                (None, Some(clubs)) => {
                    key.insert(
                        "$or",
                        vec![
                            doc! {"teams.home.id" : {"$in" : &clubs}},
                            doc! {"teams.away.id" : {"$in" : &clubs}},
                        ],
                    );
                }
                (Some(leagues), None) => {
                    key.insert("league.id", doc! {"$in": leagues});
                }
                (None, None) => {}
            }
            let model: Vec<Model> = database
                .collection::<Model>("fixture")
                .find(key, options)
                .await?
                .try_collect()
                .await?;
            redis::cmd("SET")
                .arg(redis_key.as_str())
                .arg(serde_json::to_string(&model)?)
                .query(&mut conn)?;
            redis::cmd("EXPIRE")
                .arg(redis_key.as_str())
                .arg(100)
                .query(&mut conn)?;
            Ok(model)
        }
    }

    pub async fn store(date: &str, value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await.unwrap();
        let models: Vec<Model> = serde_json::from_str(value)?;
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(false)
            .build();
        for model in models {
            database
                .collection::<Model>("fixture")
                .update_one(
                    doc! {"fixture.id":model.fixture.id},
                    doc! {"$set": bson::to_bson(&model)?},
                    Some(update_options.clone()),
                )
                .await?;
        }
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("HSET")
            .arg("fixtures_fetch_date")
            .arg(date)
            .arg(chrono::Utc::now().to_rfc3339())
            .query(&mut conn)?;
        Self::clear_cache_for_date(date)?;

        Ok(())
    }

    pub async fn change_is_bet_status(
        id: u32,
        value: bool,
        date: &str,
    ) -> Result<TransactionResult, ApplicationError> {
        let database = Database::acquire_mongo_connection().await.unwrap();
        let result = database
            .collection::<Model>("fixture")
            .update_one(
                doc! {"fixture.id": id},
                doc! {"$set":{"is_bet":value}},
                None,
            )
            .await?;
        Self::clear_cache_for_date(date)?;
        Ok(TransactionResult::expect_single_result(
            result.modified_count,
        ))
    }

    pub fn get_last_fetched_timestamp_for_date(
        date: &str,
    ) -> Result<Option<String>, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let result: Option<String> = redis::cmd("HGET")
            .arg("fixtures_fetch_date")
            .arg(date)
            .query(&mut conn)?;
        Ok(result)
    }

    pub(crate) fn clear_cache() -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg(r#"games:*"#).query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }

        Ok(())
    }

    pub(crate) fn clear_cache_for_date(date: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys_to_del: Vec<String> = redis::cmd("KEYS")
            .arg(format!(r#"games:{}::*"#, date))
            .query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }

        Ok(())
    }
}
