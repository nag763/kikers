use crate::database::Database;
use crate::error::ApplicationError;
use futures::TryStreamExt;
use mongodb::bson::doc;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_main_bookmaker: Option<bool>,
}

pub struct Entity;

impl Entity {

    async fn find_all() -> Result<Vec<Model>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models = database
            .collection::<Model>("bookmaker")
            .find(doc! {}, None)
            .await?
            .try_collect()
            .await?;
        Ok(models)
    }

    pub async fn store(value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await.unwrap();
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        let models: Vec<Model> = serde_json::from_str(value)?;
        for model in models {
            database
                .collection::<Model>("bookmaker")
                .update_one(
                    doc! {"id": model.id},
                    doc! {"$set": bson::to_bson(&model)?},
                    update_options.clone(),
                )
                .await?;
        }
        Ok(())
    }

}
