use crate::database::Database;
use crate::error::ApplicationError;
use mongodb::bson::doc;
use futures::TryStreamExt;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    pub name: String,
    pub code: Option<String>,
    pub flag: Option<String>,
}

pub struct Entity;

impl Entity {
    pub async fn find_all(
    ) -> Result<Vec<Model>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models: Vec<Model> = database
            .collection::<Model>("country")
            .find(doc! {}, None)
            .await?
            .try_collect()
            .await?;
        Ok(models)
    }

    pub async fn store(value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await.unwrap();
        let models: Vec<Model> = serde_json::from_str(value)?;
        database
            .collection::<Model>("country")
            .delete_many(doc!{}, None)
            .await?;
        database
            .collection::<Model>("country")
            .insert_many(models, None)
            .await?;
        Ok(())
    }
}
