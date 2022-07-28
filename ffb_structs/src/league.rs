//! A league is a Mongo struct gathering a set of games between clubs.
//!
//! A league is mainly defined by its name and a logo. Leagues can have several
//! seasons that might define a winner. 
//!
//! This structure has to be called once in a while from the API provider,
//! same goes for its logo.

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
use std::hash::{Hash, Hasher};
use bson::Document;
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Remote API's league ID.
    pub id: u32,
    /// The league's name
    pub name: String,
    /// Its local logo, the one that should be used while using the app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_logo: Option<String>,
    /// The country name associed with the league.
    pub country: Option<String>,
    /// The league's remote logo, shouldn't be used.
    pub logo: String,
    /// The country's flag.
    ///
    /// The country "World" doesn't have a flag.
    pub flag: Option<String>,
    /// The current round of the league.
    ///
    /// ie. 16th match day, semi-final, ...
    pub round: Option<String>,
}

/// The league's model logo, used to fetch the remote logos.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct ModelLogo {
    logo: String,
}

pub struct Entity;

#[cfg (feature = "cli")]
impl Entity {

    /// Clear the entity redis cache.
    ///
    /// This has to be called whenever the entities are modified.
    pub(crate) fn clear_cache() -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg(r#"leagues::*"#).query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }
        debug!("Cache cleaned for league entity");
        Ok(())
    }

    /// Returns all the remote logo of the stored leagues.
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
        debug!("Leagues logo successfully fetched");
        Ok(logos)
    }

    pub async fn replace_all_league_logo() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let assets_base_path: &str = &ASSETS_BASE_PATH;
        debug!("Starting the replacement of unfetched leagues logo");
        let models: Vec<Model> = database
            .collection::<Model>("league")
            .find(doc! {}, None)
            .await?
            .try_collect()
            .await?;
        // Since there is no bulk insert nor bulk modify available, we have
        // to update the logos one by one.
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
            // The logos within the fixtures also have to be updated
            database
                .collection::<Model>("fixture")
                .update_many(
                    doc! {"league.id": model.id},
                    doc! {"$set": {"localLeagueLogo": &replaced_path}},
                    None,
                )
                .await?;
        }
        Self::clear_cache()?;
        game::Entity::clear_cache()?;
        debug!("All the leagues logo have been successfully replaced");
        Ok(())
    }

    /// Stores the serialized struct within the mongo database.
    pub async fn store(value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await.unwrap();
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        let models: Vec<Model> = serde_json::from_str(value)?;
        debug!("The leagues have successfully been deserialized");
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
        Self::clear_cache()?;
        debug!("The leagues have successfully been upserted");
        Ok(())
    }

    /// Index the current models within the ES engine.
    ///
    /// They are searchable by name following the indexation.
    pub async fn index() -> Result<(), ApplicationError> {
        let client = Database::acquire_elastic_connection().await?;
        let models: Vec<Model> = EntityBuilder::build().finish().await?;
        debug!("ES engine is about to be built");
        let mut body: Vec<JsonBody<_>> = Vec::with_capacity(models.len() * 2);
        for model in models {
            body.push(json!({"index": {"_id":model.id}}).into());
            body.push(json!(model).into())
        }
        debug!("Body is ready to be submitted to ES engine");
        let response = client
            .bulk(BulkParts::Index("league"))
            .body(body)
            .send()
            .await?;
        if response.status_code().is_success() {
            Self::clear_cache()?;
            info!("Models have successfully been indexed within the ES engine");
            Ok(())
        } else {
            Err(ApplicationError::ElasticError(format!(
                "Error while joining the elastics daemon : {:?}",
                response
            )))
        }
    }
}

#[derive(Default, Hash, Debug)]
pub struct EntityBuilder {
    /// The list of ids to query.
    ///
    /// **Warning :** Giving a value to this argument will reset name to
    /// [Option::None].
    ids: Option<Vec<u32>>,
    /// The name to search for.
    ///
    /// This is using elastic search.
    ///
    /// **Warning :** Giving a value to this argument will reset ids to
    /// [Option::None].
    name: Option<String>,
}

impl EntityBuilder {
    /// Create the builder.
    pub fn build() -> EntityBuilder {
        Self::default()
    }

    /// Set the ids property.
    ///
    /// **Warning :** Giving a value to this argument will reset name to
    /// [Option::None].
    pub fn ids(&mut self, ids: Option<Vec<u32>>) -> &mut Self {
        self.ids = ids;
        if self.ids.is_some() && self.name.is_some() {
            self.name = None;
        }
        self
    }

    /// Set the name property.
    ///
    /// **Warning :** Giving a value to this argument will reset ids to
    /// [Option::None].
    pub fn name(&mut self, name: Option<String>) -> &mut Self {
        self.name = name;
        if self.name.is_some() && self.ids.is_some() {
            self.ids = None;
        }
        self
    }

    /// Returns the list of models.
    pub async fn finish(&self) -> Result<Vec<Model>, ApplicationError> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let redis_key: String = format!("leagues::{:x}", hasher.finish());
        debug!("Lookup key for leagues : {:#?}", &self);
        let mut conn = Database::acquire_redis_connection()?;
        let cached_struct: Option<String> = redis::cmd("GETEX")
            .arg(redis_key.as_str())
            .arg("EX")
            .arg(200)
            .query(&mut conn)?;
        if let Some(cached_struct) = cached_struct {
            let deserialized_struct: Vec<Model> = serde_json::from_str(cached_struct.as_str())?;
            debug!("Model leagues has been found from cache for the given lookup");
            Ok(deserialized_struct)
        } else {
            debug!("Model leagues has not been found from cache for the given lookup, the entity builder lookup is starting.");
            let models: Vec<Model> = if let Some(name) = &self.name {
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
                models
            } else {
                let database = Database::acquire_mongo_connection().await?;
                let mut search_criteria = Document::new();
                if let Some(ids) = &self.ids {
                    search_criteria.insert("id", doc! {"$in": ids});
                }
                let models: Vec<Model> = database
                    .collection::<Model>("league")
                    .find(search_criteria, None)
                    .await?
                    .try_collect()
                    .await?;
                models
            };
            redis::cmd("SET")
                .arg(redis_key.as_str())
                .arg(serde_json::to_string(&models)?)
                .arg("EX")
                .arg(200)
                .query(&mut conn)?;
            debug!(
                "The league entity builder query finished with success and has been stored in cache"
            );
            Ok(models)
        }
    }
}
