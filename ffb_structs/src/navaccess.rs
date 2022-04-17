use crate::database::Database;
use crate::error::ApplicationError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    pub id: u32,
    pub label: String,
    pub href: String,
    pub position: Option<u32>,
}

pub struct Entity;

impl Entity {
    pub async fn get_navaccess_for_role_id(id: u32) -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let models : Vec<Model> = sqlx::query_as("SELECT * FROM NAVACCESS na INNER JOIN ROLE_NAVACCESS rna ON na.id = rna.navaccess_id WHERE rna.role_id=? ORDER BY na.position")
            .bind(&id)
            .fetch_all(&mut conn)
            .await?;
        Ok(models)
    }
}
