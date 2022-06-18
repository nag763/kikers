use crate::common_api_structs::{Fixture, Goals, Odds, Score, Teams};
use crate::database::Database;
use crate::error::ApplicationError;
use crate::league::Model as League;
use crate::transaction_result::TransactionResult;
use futures::TryStreamExt;
use mongodb::bson::doc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Model {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bet: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odds: Option<Odds>,
    #[serde(rename = "localLeagueLogo", skip_serializing_if = "Option::is_none")]
    pub league_local_logo: Option<String>,
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
    pub async fn store(date: &str, value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await.unwrap();
        let models: Vec<Model> = serde_json::from_str(value)?;
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
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
        Self::clear_cache()?;

        Ok(())
    }

    pub async fn change_is_bet_status(
        id: u32,
        value: bool,
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
        Self::clear_cache()?;
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
}

#[derive(Default, Hash, Debug)]
pub struct EntityBuilder {
    date: Option<String>,
    leagues: Option<Vec<u32>>,
    clubs: Option<Vec<u32>>,
    bets: bool,
    potential_bets: bool,
    limit: Option<i64>,
}

impl EntityBuilder {
    pub fn build() -> EntityBuilder {
        Self::default()
    }

    pub fn date<'a>(&'a mut self, date: &str) -> &'a mut Self {
        self.date = Some(date.into());
        self
    }

    pub fn leagues<'a>(&'a mut self, leagues: Vec<u32>) -> &'a mut Self {
        self.leagues = Some(leagues);
        self
    }

    pub fn clubs<'a>(&'a mut self, clubs: Vec<u32>) -> &'a mut Self {
        self.clubs = Some(clubs);
        self
    }

    pub fn bets<'a>(&'a mut self, bets: bool) -> &'a mut Self {
        self.bets = bets;
        self
    }

    pub fn potential_bets<'a>(&'a mut self, potential_bets: bool) -> &'a mut Self {
        self.potential_bets = potential_bets;
        self
    }

    pub fn limit<'a>(&'a mut self, limit: i64) -> &'a mut Self {
        self.limit = Some(limit);
        self
    }

    pub async fn finish(&self) -> Result<Vec<Model>, ApplicationError> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let redis_key: String = format!("games::{:x}", hasher.finish());
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
            let options: Option<mongodb::options::FindOptions> = match self.limit {
                Some(v) => Some(
                    mongodb::options::FindOptions::builder()
                        .limit(Some(v))
                        .build(),
                ),
                None => None,
            };
            let mut key: bson::Document = bson::Document::new();
            let mut query_selector: Vec<bson::Document> = vec![];
            if let Some(leagues) = &self.leagues {
                query_selector.push(doc! {"league.id": {"$in": leagues}});
            }
            if let Some(clubs) = &self.clubs {
                query_selector.push(doc! {"teams.home.id" : {"$in" : &clubs}});
                query_selector.push(doc! {"teams.away.id" : {"$in" : &clubs}});
            }
            if self.bets {
                query_selector.push(doc! {"is_bet": true});
            }
            if self.potential_bets {
                query_selector.push(doc! {"odds": {"$ne": null}});
            }
            if let Some(date) = &self.date {
                key.insert("fixture.date", doc! {"$regex" : date});
            }
            if !query_selector.is_empty() {
                key.insert("$or", query_selector);
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
}
