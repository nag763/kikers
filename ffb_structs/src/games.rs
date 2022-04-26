use crate::common_api_structs::{Fixture, Goals, League, Teams};
use crate::database::Database;
use crate::error::ApplicationError;
use futures::TryStreamExt;
use mongodb::bson::doc;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Model {
    pub fixture: Fixture,
    pub league: League,
    pub teams: Teams,
    pub goals: Goals,
}

pub struct Entity;

impl Entity {
    pub async fn find_all_for_date(
        date: &str,
        fav_leagues: Option<Vec<u32>>,
        limit: Option<i64>,
    ) -> Result<Vec<Model>, ApplicationError> {
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
        if let Some(fav_leagues) = fav_leagues {
            key.insert("league.id", doc! {"$in": fav_leagues});
        }
        error!("Search key : {:#?}", key);
        error!("Options : {:#?}", options);
        let model: Vec<Model> = database
            .collection::<Model>("fixture")
            .find(key, options)
            .await?
            .try_collect()
            .await?;
        Ok(model)
    }

    pub async fn store(date: &str, value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await.unwrap();
        let models: Vec<Model> = serde_json::from_str(value)?;
        database
            .collection::<Model>("fixture")
            .delete_many(doc! {"fixture.date": {"$regex" : date}}, None)
            .await?;
        database
            .collection::<Model>("fixture")
            .insert_many(models, None)
            .await?;
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("HSET")
            .arg("fixtures_fetch_date")
            .arg(date)
            .arg(chrono::Utc::now().to_rfc3339())
            .query(&mut conn)?;
        Ok(())
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
}
