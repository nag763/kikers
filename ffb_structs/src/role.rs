use crate::database::Database;
use crate::error::ApplicationError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow, Eq, Hash)]
pub struct Model {
    pub id: u32,
    pub name: String,
}

pub struct Entity;

impl Entity {
    pub async fn get_roles() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let models : Vec<Model> = sqlx::query_as("SELECT * FROM ROLE")
            .fetch_all(&mut conn)
            .await?;
        Ok(models)
    }
}
