use crate::database::Database;
use crate::error::ApplicationError;
use crate::game;
use crate::{ASSETS_BASE_PATH, RE_HOST_REPLACER};
use elasticsearch::http::request::JsonBody;
use elasticsearch::{BulkParts, SearchParts};
use futures::StreamExt;
use futures::TryStreamExt;
use mongodb::bson::doc;
use serde_json::{json, Value};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    pub id: u32,
    pub name: String,
    #[serde(rename = "localLogo", skip_serializing_if = "Option::is_none")]
    pub local_logo: Option<String>,
    pub country: Option<String>,
    pub logo: String,
    pub flag: Option<String>,
    pub round: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct ModelLogo {
    logo: String,
}

pub struct Entity;

impl Entity {
    pub async fn get_fav_leagues_of_user(
        fav_leagues_id: Vec<u32>,
    ) -> Result<Vec<Model>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models: Vec<Model> = database
            .collection::<Model>("league")
            .find(doc! { "id" : { "$in" : fav_leagues_id }}, None)
            .await?
            .try_collect()
            .await?;
        Ok(models)
    }

    async fn find_all() -> Result<Vec<Model>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models = database
            .collection::<Model>("league")
            .find(doc! {}, None)
            .await?
            .try_collect()
            .await?;
        Ok(models)
    }

    pub async fn get_all_leagues_logo() -> Result<Vec<String>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let mut results = database
            .collection::<Model>("league")
            .aggregate(
                vec![
                    doc! {"$match" : {"localLogo":null}},
                    doc! {"$replaceRoot": { "newRoot": {"logo": "$logo"} }},
                ],
                None,
            )
            .await?;
        let mut logos: Vec<String> = Vec::new();
        while let Some(result) = results.next().await {
            let doc: ModelLogo = bson::from_document(result?)?;
            logos.push(doc.logo);
        }
        Ok(logos)
    }

    pub async fn replace_all_league_logo() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let assets_base_path: &str = &ASSETS_BASE_PATH;
        let models: Vec<Model> = database
            .collection::<Model>("league")
            .find(doc! {}, None)
            .await?
            .try_collect()
            .await?;
        for model in models {
            let replaced_path: String = RE_HOST_REPLACER
                .replace(&model.logo, assets_base_path)
                .into();
            database
                .collection::<Model>("league")
                .update_one(
                    doc! {"id": model.id},
                    doc! {"$set": {"localLogo": &replaced_path}},
                    None,
                )
                .await?;
            database
                .collection::<Model>("fixture")
                .update_many(
                    doc! {"league.id": model.id},
                    doc! {"$set": {"localLeagueLogo": &replaced_path}},
                    None,
                )
                .await?;
        }
        game::Entity::clear_cache()?;
        Ok(())
    }

    pub async fn store(value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await.unwrap();
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        let models: Vec<Model> = serde_json::from_str(value)?;
        for model in models {
            database
                .collection::<Model>("league")
                .update_one(
                    doc! {"id": model.id},
                    doc! {"$set": bson::to_bson(&model)?},
                    update_options.clone(),
                )
                .await?;
        }
        Ok(())
    }

    pub async fn index() -> Result<(), ApplicationError> {
        let client = Database::acquire_elastic_connection().await?;
        let models: Vec<Model> = Self::find_all().await?;
        let mut body: Vec<JsonBody<_>> = Vec::with_capacity(models.len() * 2);
        for model in models {
            body.push(json!({"index": {"_id":model.id}}).into());
            body.push(json!(model).into())
        }
        let response = client
            .bulk(BulkParts::Index("league"))
            .body(body)
            .send()
            .await?;
        if response.status_code().is_success() {
            Ok(())
        } else {
            Err(ApplicationError::ElasticError(format!(
                "Error while joining the elastics daemon : {:?}",
                response
            )))
        }
    }

    pub async fn search_name(name: &str) -> Result<Vec<Model>, ApplicationError> {
        let client = Database::acquire_elastic_connection().await?;
        let response = client
            .search(SearchParts::Index(&["league"]))
            .from(0)
            .size(10)
            .body(json!(
                    {
                        "query": {
                            "match": {
                                "name": name
                            }
                        }
                    }
            ))
            .send()
            .await?;
        let response_body = response.json::<Value>().await?;
        let mut models: Vec<Model> = Vec::with_capacity(10);
        for object in response_body["hits"]["hits"].as_array().ok_or_else(|| {
            ApplicationError::ElasticError("Elasticsearch result is in bad format".into())
        })? {
            models.push(serde_json::from_value(object["_source"].clone())?);
        }
        Ok(models)
    }
}
