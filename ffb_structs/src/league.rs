use crate::common_api_structs::League;
use crate::country::Model as Country;
use crate::database::Database;
use crate::error::ApplicationError;
use crate::{ASSETS_BASE_PATH, RE_HOST_REPLACER};
use bson::Bson;
use futures::StreamExt;
use futures::TryStreamExt;
use mongodb::bson::doc;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    pub league: League,
    pub country: Country,
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

    pub async fn get_all_leagues_logo() -> Result<Vec<String>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let mut results = database
            .collection::<Model>("league")
            .aggregate(
                vec![doc! {"$replaceRoot": { "newRoot": {"logo": "$league.logo"} }}],
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
                .replace(&model.league.logo, assets_base_path)
                .into();
            database
                .collection::<Model>("league")
                .update_one(
                    doc! {"league.id": model.league.id},
                    doc! {"$set": {"league.localLogo": replaced_path}},
                    None,
                )
                .await?;
        }
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
                    doc! {"league.id": model.league.id},
                    doc! {"$set": bson::to_bson(&model)?},
                    update_options.clone(),
                )
                .await?;
        }
        Ok(())
    }
}
