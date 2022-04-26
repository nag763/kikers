use crate::common_api_structs::League;
use crate::country::Model as Country;
use crate::database::Database;
use crate::error::ApplicationError;
use bson::Bson;
use futures::TryStreamExt;
use mongodb::bson::doc;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    pub league: League,
    pub country: Country,
}

pub struct Entity;

impl Entity {
    pub async fn get_fav_leagues_of_user(
        fav_leagues_id: Vec<u32>,
    ) -> Result<Vec<Model>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models: Vec<Model> = database
            .collection::<Model>("league")
            .find(doc! { "league.id" : { "$in" : fav_leagues_id }}, None)
            .await?
            .try_collect()
            .await?;
        Ok(models)
    }

    pub async fn get_leagues_for_country_code(
        country_code: &str,
    ) -> Result<Vec<Model>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let search_key: Bson = match country_code {
            v if !v.is_empty() => Bson::String(v.into()),
            _ => Bson::Null,
        };
        let models: Vec<Model> = database
            .collection::<Model>("league")
            .find(doc! { "country.code" : search_key}, None)
            .await?
            .try_collect()
            .await?;
        Ok(models)
    }

    pub async fn store(value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await.unwrap();
        let models: Vec<Model> = serde_json::from_str(value)?;
        database
            .collection::<Model>("league")
            .delete_many(doc! {}, None)
            .await?;
        database
            .collection::<Model>("league")
            .insert_many(models, None)
            .await?;
        Ok(())
    }
}
