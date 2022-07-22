//! A bookmaker is a Mongo entity odd provider for games.
//!
//! The bookmaker is mainly used to fetch the remote odds. The application
//! works so that a main bookmaker is set by the admin, and this bookmaker will
//! be used to fetch the remote odds.

use crate::common_api_structs::Bet;
use crate::database::Database;
use crate::error::ApplicationError;
#[cfg(feature = "server")]
use crate::transaction_result::TransactionResult;
#[cfg(feature = "server")]
use futures::TryStreamExt;
use mongodb::bson::doc;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    /// The remote API struct's ID.
    pub id: u32,
    /// Name of the provider (ie. Unibet).
    pub name: String,
    /// The main provider is the bookmaker used to fetch the odds.
    ///
    /// Only one bookmaker within the DB should have this attribute set to true.
    #[serde(skip_serializing_if = "Option::is_none")]
    /// This field is only used when fetching the odds.
    ///
    /// It is neither set within the MongoDB.
    pub is_main_bookmaker: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bets: Option<Vec<Bet>>,
}

pub struct Entity;

impl Entity {
    /// Get all the bookmakers stored in the database.
    #[cfg(feature = "server")]
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

    /// Returns the ID of the main bookmaker.
    ///
    /// If no main bookmaker is set yet, None will be returned.
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

    /// Changes the main bookmaker of the application.
    ///
    /// The application can only have a unique bookmaker, if the main bookmaker
    /// is changed, the current bookmaker's is_main_bookmaker value will be set
    /// to false following this call.
    ///
    /// This application throws an error if the new main bookmaker is the same
    /// as the previous one.
    #[cfg(feature = "server")]
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
            .update_one(
                doc! {"id": bookmaker_id},
                doc! {"$set": {"is_main_bookmaker": true}},
                None,
            )
            .await?;
        Ok(TransactionResult::expect_single_result(
            result.modified_count,
        ))
    }

    /// Store the serialized value of the model into the mongo database.
    ///
    /// # Arguments
    ///
    /// - value : The serialized value of the struct.
    #[cfg(feature = "cli")]
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
