use crate::database::Database;
use crate::error::ApplicationError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    pub id: u32,
    pub language_id: u32,
    pub short_name: String,
    pub long_name: String,
}

pub struct Entity;

impl Entity {
    pub async fn get_locales() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let models: Vec<Model> = sqlx::query_as("SELECT * FROM LOCALE ORDER BY long_name")
            .fetch_all(&mut conn)
            .await?;
        Ok(models)
    }
}
