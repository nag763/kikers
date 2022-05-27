use crate::database::Database;
use crate::error::ApplicationError;
use crate::{ASSETS_BASE_PATH, RE_HOST_REPLACER};
use bson::oid::ObjectId;
use futures::StreamExt;
use futures::TryStreamExt;
use mongodb::bson::doc;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    #[serde(rename = "_id", skip_serializing)]
    pub id: Option<ObjectId>,
    pub name: String,
    pub code: Option<String>,
    pub flag: Option<String>,
    #[serde(rename = "localFlag")]
    pub local_flag: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct ModelLogo {
    flag: Option<String>,
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

    pub async fn get_all_countries_logo() -> Result<Vec<String>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let mut results = database
            .collection::<Model>("country")
            .aggregate(
                vec![doc! {"$replaceRoot": { "newRoot": {"flag": "$flag"} }}],
                None,
            )
            .await?;
        let mut logos: Vec<String> = Vec::new();
        while let Some(result) = results.next().await {
            let doc: ModelLogo = bson::from_document(result?)?;
            if let Some(flag) = doc.flag {
                logos.push(flag);
            }
        }
        Ok(logos)
    }

    pub async fn replace_all_country_logo() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let assets_base_path: &str = &ASSETS_BASE_PATH;
        let models: Vec<Model> = database
            .collection::<Model>("country")
            .find(doc! {}, None)
            .await?
            .try_collect()
            .await?;
        for model in models {
            if let (Some(id), Some(flag)) = (model.id, model.flag) {
                let replaced_path: String =
                    RE_HOST_REPLACER.replace(&flag, assets_base_path).into();
                database
                    .collection::<Model>("country")
                    .update_one(
                        doc! {"_id": id},
                        doc! {"$set": {"localFlag": replaced_path}},
                        None,
                    )
                    .await?;
            }
        }
        Ok(())
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
