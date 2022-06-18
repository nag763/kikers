use crate::database::Database;
use crate::error::ApplicationError;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Model {
    pub title: String,
    pub href: String,
}

pub struct Entity;

impl Entity {
    pub fn store(models: Vec<Model>) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("SET")
            .arg("infos")
            .arg(serde_json::to_string(&models)?)
            .query(&mut conn)?;
        Ok(())
    }

    pub fn get_all() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let models: String = redis::cmd("GET").arg("infos").query(&mut conn)?;
        Ok(serde_json::from_str(&models)?)
    }
}
