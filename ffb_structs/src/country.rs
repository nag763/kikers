use crate::database::Database;
use crate::error::ApplicationError;
use futures::TryStreamExt;
use mongodb::bson::doc;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    pub name: String,
    pub code: Option<String>,
    pub flag: Option<String>,
}

pub struct Entity;

impl Entity {
    pub async fn find_all() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let countries_as_string: Option<String> =
            redis::cmd("GET").arg("countries").query(&mut conn)?;
        if let Some(countries_as_string) = countries_as_string {
            let models: Vec<Model> = serde_json::from_str(&countries_as_string)?;
            Ok(models)
        } else {
            let database = Database::acquire_mongo_connection().await?;
            let models: Vec<Model> = database
                .collection::<Model>("country")
                .find(doc! {}, None)
                .await?
                .try_collect()
                .await?;
            redis::cmd("SET")
                .arg("countries")
                .arg(serde_json::to_string(&models)?)
                .arg("EX")
                .arg("500")
                .query(&mut conn)?;
            Ok(models)
        }
    }

    pub async fn store(value: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let database = Database::acquire_mongo_connection().await.unwrap();
        let models: Vec<Model> = serde_json::from_str(value)?;
        database
            .collection::<Model>("country")
            .delete_many(doc! {}, None)
            .await?;
        database
            .collection::<Model>("country")
            .insert_many(models, None)
            .await?;
        redis::cmd("DEL").arg("countries").query(&mut conn)?;
        Ok(())
    }
}
