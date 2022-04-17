use crate::database::Database;
use crate::error::ApplicationError;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Model {
    pub name: String,
    pub code: Option<String>,
    pub flag: Option<String>,
}

pub struct Entity;

impl Entity {
    pub fn find_all() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let model_as_string: String = redis::cmd("GET").arg("countries").query(&mut conn)?;
        let model: Vec<Model> = serde_json::from_str(model_as_string.as_str())?;
        Ok(model)
    }
    pub fn store(value: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("GET")
            .arg("countries")
            .arg(value)
            .query(&mut conn)?;
        Ok(())
    }
}
