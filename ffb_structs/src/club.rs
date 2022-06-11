use crate::database::Database;
use crate::error::ApplicationError;
use crate::game;
use crate::{ASSETS_BASE_PATH, RE_HOST_REPLACER};
use elasticsearch::http::request::JsonBody;
use elasticsearch::BulkParts;
use elasticsearch::SearchParts;
use futures::StreamExt;
use futures::TryStreamExt;
use mongodb::bson::doc;
use serde_json::json;
use serde_json::Value;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    pub id: u32,
    pub name: String,
    pub logo: Option<String>,
    #[serde(rename = "localLogo")]
    pub local_logo: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct ModelLogo {
    logo: Option<String>,
}

pub struct Entity;

impl Entity {
    async fn find_all() -> Result<Vec<Model>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models: Vec<Model> = database
            .collection::<Model>("club")
            .find(doc! {}, None)
            .await?
            .try_collect()
            .await?;
        Ok(models)
    }

    pub async fn get_fav_clubs_of_user(
        fav_clubs_id: Vec<u32>,
    ) -> Result<Vec<Model>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models: Vec<Model> = database
            .collection::<Model>("club")
            .find(doc! { "id" : { "$in" : fav_clubs_id }}, None)
            .await?
            .try_collect()
            .await?;
        Ok(models)
    }

    pub async fn get_logos() -> Result<Vec<String>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let mut results = database
            .collection::<Model>("club")
            .aggregate(
                vec![
                    doc! {"$match": {"localLogo":null}},
                    doc! {"$replaceRoot": { "newRoot": {"logo": "$logo"} }},
                ],
                None,
            )
            .await?;
        let mut logos: Vec<String> = Vec::new();
        while let Some(result) = results.next().await {
            let doc: ModelLogo = bson::from_document(result?)?;
            if let Some(logo) = doc.logo {
                logos.push(logo);
            }
        }
        Ok(logos)
    }

    pub async fn replace_all_club_logo() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let assets_base_path: &str = &ASSETS_BASE_PATH;
        let models: Vec<Model> = database
            .collection::<Model>("club")
            .find(doc! {}, None)
            .await?
            .try_collect()
            .await?;
        for model in models {
            if let Some(logo) = model.logo {
                let replaced_path: String =
                    RE_HOST_REPLACER.replace(&logo, assets_base_path).into();
                database
                    .collection::<Model>("club")
                    .update_one(
                        doc! {"id": model.id},
                        doc! {"$set": {"localLogo": &replaced_path}},
                        None,
                    )
                    .await?;
                database
                    .collection::<Model>("fixture")
                    .update_many(
                        doc! {"teams.home.id": model.id},
                        doc! {"$set": {"teams.home.localLogo": &replaced_path}},
                        None,
                    )
                    .await?;
                database
                    .collection::<Model>("fixture")
                    .update_many(
                        doc! {"teams.away.id": model.id},
                        doc! {"$set": {"teams.away.localLogo": &replaced_path}},
                        None,
                    )
                    .await?;
            }
        }
        game::Entity::clear_cache()?;
        Ok(())
    }

    pub async fn store() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let updatable = database.collection::<Model>("club");
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        let home_teams = database
            .collection::<Model>("fixture")
            .aggregate([doc! {"$replaceRoot": {"newRoot": "$teams.home"}}], None)
            .await?;
        let away_teams = database
            .collection::<Model>("fixture")
            .aggregate([doc! {"$replaceRoot": {"newRoot": "$teams.away"}}], None)
            .await?;

        Self::upsert_list_of_docs(home_teams, updatable.clone(), update_options.clone()).await?;
        Self::upsert_list_of_docs(away_teams, updatable.clone(), update_options.clone()).await?;
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
            .bulk(BulkParts::Index("club"))
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
            .search(SearchParts::Index(&["club"]))
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
        for object in
            response_body["hits"]["hits"]
                .as_array()
                .ok_or(ApplicationError::ElasticError(
                    "Elasticsearch result is in bad format".into(),
                ))?
        {
            models.push(serde_json::from_value(object["_source"].clone())?);
        }
        Ok(models)
    }

    async fn upsert_list_of_docs(
        mut docs: mongodb::Cursor<bson::Document>,
        updatable: mongodb::Collection<Model>,
        update_options: mongodb::options::UpdateOptions,
    ) -> Result<(), ApplicationError> {
        while let Some(result) = docs.next().await {
            let doc: Model = bson::from_document(result?)?;
            updatable
                .update_one(
                    doc! {"id":doc.id},
                    doc! {"$set": bson::to_bson(&doc)?},
                    Some(update_options.clone()),
                )
                .await?;
        }
        Ok(())
    }
}
