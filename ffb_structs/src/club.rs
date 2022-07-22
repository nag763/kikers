//! A club is a MongoDB struct that represents the entity of the same name.
//!
//! It gathers some informations such as the name of the club and its logo.
//!
//! The particularity of this struct is that it is directly fetched from the
//! fixtures to economize the API calls. So only the clubs that are already in
//! the fixtures will be accessible within the application.
//!
//! The clubs are searchable by their names thanks to the ES engine.

use crate::database::Database;
use crate::error::ApplicationError;
#[cfg(feature = "cli")]
use crate::game;
#[cfg(feature = "cli")]
use crate::{ASSETS_BASE_PATH, RE_HOST_REPLACER};
#[cfg(feature = "cli")]
use elasticsearch::{http::request::JsonBody, BulkParts}; 
use elasticsearch::SearchParts;
#[cfg(feature = "cli")]
use futures::StreamExt;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// ID of the model within the distant API.
    pub id: u32,
    /// The club's name.
    pub name: String,
    /// Its logo.
    ///
    /// This variable is set as the value of the remote logo. In the end, only
    /// the local_logo field will be used within the application.
    pub logo: Option<String>,
    /// The local logo.
    ///
    /// The value of this should point to a local URI accesible to the client,
    /// this logo **should** be used rather than the logo variable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_logo: Option<String>,
}

/// This struct is used merely to pass awway a list of logo to the cli while
/// fetching the logos locally.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct ModelLogo {
    /// The remote logo URL, to be downloaded during the process.
    logo: Option<String>,
}

#[cfg(feature = "cli")]
pub struct Entity;

#[cfg(feature = "cli")]
impl Entity {
    

    /// Clear the entity redis cache.
    ///
    /// This has to be called whenever the entities are modified.
    pub(crate) fn clear_cache() -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg(r#"clubs::*"#).query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }
        debug!("Cache cleaned for club entity");
        Ok(())
    }
    
    /// Get the logos of the entity.
    ///
    /// This will return only the logos for the given entity, making it easy to
    /// download them in bulk.
    pub async fn get_logos() -> Result<Vec<String>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        // We replace the root document by its logo, and rename it.
        // We only fetch the clubs that don't already have an existing local 
        // logo.
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
        debug!("Logos successfully fetched from mongo");
        let mut logos: Vec<String> = Vec::new();
        while let Some(result) = results.next().await {
            let doc: ModelLogo = bson::from_document(result?)?;
            if let Some(logo) = doc.logo {
                logos.push(logo);
            }
        }
        debug!("Logos successfully binded to the Model Logo, {} unbinded logo found", logos.len());
        Ok(logos)
    }

    /// Replace all the existing club logos by their local equivalent.
    ///
    /// This has to be called after fetching all the remote logos
    pub async fn replace_all_club_logo() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let assets_base_path: &str = &ASSETS_BASE_PATH;
        let models: Vec<Model> = database
            .collection::<Model>("club")
            .find(doc! {}, None)
            .await?
            .try_collect()
            .await?;
        // Since there is no bulk insert for [mongo] nor bulk `modify 
        // substring` for mongodb engine, we have to modify each model one by 
        // one.
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
                // Besides of the club models, the fixtures are also containing
                // the club's logo.
                database
                    .collection::<Model>("fixture")
                    .update_many(
                        doc! {"teams.home.id": model.id},
                        doc! {"$set": {"localHomeLogo": &replaced_path}},
                        None,
                    )
                    .await?;
                database
                    .collection::<Model>("fixture")
                    .update_many(
                        doc! {"teams.away.id": model.id},
                        doc! {"$set": {"localAwayLogo": &replaced_path}},
                        None,
                    )
                    .await?;
            }
        }
        // Both the clubs and game caches have to be cleared following this.
        Self::clear_cache()?;
        game::Entity::clear_cache()?;
        Ok(())
    }

    /// Store the clubs.
    ///
    /// Unlike the structs that are fetched from the API provider, this 
    /// function uses the local data from MongoDB to store the games.
    pub async fn store() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let updatable = database.collection::<Model>("club");
        // We don't insert clubs that are already existing, but we can consider
        // updating them.
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        // We query the fixture model by using only the teams that are in.
        let home_teams = database
            .collection::<Model>("fixture")
            .aggregate([doc! {"$replaceRoot": {"newRoot": "$teams.home"}}], None)
            .await?;
        let away_teams = database
            .collection::<Model>("fixture")
            .aggregate([doc! {"$replaceRoot": {"newRoot": "$teams.away"}}], None)
            .await?;

        // Once the structs are fetched, we upsert them within the club model.
        Self::upsert_list_of_docs(home_teams, updatable.clone(), update_options.clone()).await?;
        Self::upsert_list_of_docs(away_teams, updatable.clone(), update_options.clone()).await?;
        Self::clear_cache()?;
        Ok(())
    }

    /// Index the clubs within the elastic search engine.
    ///
    /// The existing models within the engine will be overwritten.
    pub async fn index() -> Result<(), ApplicationError> {
        let models: Vec<Model> = EntityBuilder::build().finish().await?;
        let client = Database::acquire_elastic_connection().await?;
        debug!("Starting to build the body of the ES request");
        let mut body: Vec<JsonBody<_>> = Vec::with_capacity(models.len() * 2);
        for model in models {
            body.push(json!({"index": {"_id":model.id}}).into());
            body.push(json!(&model).into())
        }
        debug!("Body built successfully, request to update elastic search starting");
        let response = client
            .bulk(BulkParts::Index("club"))
            .body(body)
            .send()
            .await?;
        if response.status_code().is_success() {
            Ok(())
        } else {
            let err_msg : String = format!(
                "Error while joining the elastics database : {:?}",
                response
            );
            Err(ApplicationError::ElasticError(err_msg))
        }
    }
    

    /// Upsert a list of documents within the database.
    ///
    /// # Arguments
    ///
    /// - docs : The list of documents to insert.
    /// - updatable : The database that have to be updated.
    /// - update_options : The options to update within the mongodb.
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
        let redis_key: String = format!("clubs::{:x}", hasher.finish());
        debug!("Lookup key for clubs : {:#?}", &self);
        let mut conn = Database::acquire_redis_connection()?;
        let cached_struct: Option<String> = redis::cmd("GETEX")
            .arg(redis_key.as_str())
            .arg("EX")
            .arg(200)
            .query(&mut conn)?;
        if let Some(cached_struct) = cached_struct {
            let deserialized_struct: Vec<Model> = serde_json::from_str(cached_struct.as_str())?;
            debug!("Model clubs has been found from cache for the given lookup");
            Ok(deserialized_struct)
        } else {
            debug!("Model clubs has not been found from cache for the given lookup, the entity builder lookup is starting.");
            let models: Vec<Model> = if let Some(name) = &self.name {
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
                    .collection::<Model>("club")
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
            debug!("The club entity builder query finished with success and has been stored in cache");
            Ok(models)
        }
    }
}
