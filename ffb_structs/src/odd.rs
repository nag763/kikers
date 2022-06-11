use crate::bookmaker::Model as Bookmaker;
use crate::database::Database;
use crate::error::ApplicationError;
use mongodb::bson::doc;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct SimplifiedFixture {
    id: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct Model {
    fixture: SimplifiedFixture,
    bookmakers: Vec<Bookmaker>,
}

pub struct Entity;

impl Entity {
    pub async fn store(value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        let models: Vec<Model> = serde_json::from_str(value)?;
        for model in models {
            database
                .collection::<Model>("odd")
                .update_one(
                    doc! {"fixture_id": model.fixture.id},
                    doc! {"$set": bson::to_bson(&model)?},
                    update_options.clone(),
                )
                .await?;
        }
        Ok(())
    }
}
