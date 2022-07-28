//! Role define a set of privileges and actions that a user can execute.
//!
//! A user has a unique role that is upgraded only by a user who has a 
//! higher role.

use crate::database::Database;
use crate::error::ApplicationError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow, Eq, Hash)]
pub struct Model {
    /// The role's MySQL id.
    pub id: u32,
    /// The role's name.
    pub name: String,
}

pub struct Entity;

impl Entity {

    /// Get the different roles available within the application.
    pub async fn get_roles() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let models: Vec<Model> = sqlx::query_as("SELECT * FROM ROLE")
            .fetch_all(&mut conn)
            .await?;
        Ok(models)
    }
}
