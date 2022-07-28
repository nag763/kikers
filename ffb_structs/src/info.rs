//! Redis stored set of latest informations.
//!
//! These informations are stored directly from r/soccer, which is the best
//! way to get the latest transfer news or review just scored goals.
//!
//! These informations are downloaded regulary through the cli. A good frequency
//! should be every minute. Understand that storing again the news will 
//! overwrite the former stored.

use crate::database::Database;
use crate::error::ApplicationError;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Model {
    pub title: String,
    pub href: String,
}

pub struct Entity;

impl Entity {

    /// Stores the set of structs in the redis database.
    ///
    /// # Arguments
    ///
    /// - models : The list to store.
    pub fn store(models: Vec<Model>) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("SET")
            .arg("infos")
            .arg(serde_json::to_string(&models)?)
            .query(&mut conn)?;
        debug!("Latest news stored successfully into database");
        Ok(())
    }

    /// Retrieves all the latest news.
    ///
    /// **Warning** : Will panic if no news are stored.
    pub fn get_all() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let models_serialized: String = redis::cmd("GET").arg("infos").query(&mut conn)?;
        let models : Vec<Model> = serde_json::from_str(&models_serialized)?;
        debug!("The news have been retrieved with success");
        Ok(models)
    }
}
