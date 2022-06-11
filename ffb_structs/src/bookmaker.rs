use crate::common_api_structs::Bet;
use crate::database::Database;
use crate::error::ApplicationError;
use crate::transaction_result::TransactionResult;
use futures::TryStreamExt;
use mongodb::bson::doc;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_main_bookmaker: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bets: Option<Vec<Bet>>,
}

pub struct Entity;

impl Entity {
    pub async fn get_all() -> Result<Vec<Model>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models = database
            .collection::<Model>("bookmaker")
            .find(doc! {}, None)
            .await?
            .try_collect()
            .await?;
        Ok(models)
    }

    pub async fn get_main_bookmaker_id() -> Result<Option<u32>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let model: Option<Model> = database
            .collection::<Model>("bookmaker")
            .find_one(doc! {"is_main_bookmaker": true}, None)
            .await?;
        if let Some(model) = model {
            Ok(Some(model.id))
        } else {
            Ok(None)
        }
    }

    pub async fn set_main_bookmaker(
        bookmaker_id: u32,
    ) -> Result<TransactionResult, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        database
            .collection::<Model>("bookmaker")
            .update_many(
                doc! {"id": {"$ne": bookmaker_id}},
                doc! {"$set": {"is_main_bookmaker": false}},
                None,
            )
            .await?;
        let result = database
            .collection::<Model>("bookmaker")
            .update_many(
                doc! {"id": bookmaker_id},
                doc! {"$set": {"is_main_bookmaker": true}},
                None,
            )
            .await?;
        Ok(TransactionResult::expect_single_result(
            result.modified_count,
        ))
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
