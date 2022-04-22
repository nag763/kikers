use crate::database::Database;
use crate::error::ApplicationError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display, sqlx::FromRow)]
#[display(fmt = "{}=>{}(positioned at {:?}", label, href, position)]
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

    pub async fn get_role_navaccess_mapping() -> Result<HashMap<u32, Vec<Model>>, ApplicationError>
    {
        let mut conn = Database::acquire_sql_connection().await?;
        let roles: Vec<(u32,)> = sqlx::query_as("SELECT id FROM ROLE")
            .fetch_all(&mut conn)
            .await?;
        let mut role_navaccess = HashMap::new();
        for role in roles {
            let models : Vec<Model> = sqlx::query_as("SELECT * FROM NAVACCESS na INNER JOIN ROLE_NAVACCESS rna ON na.id = rna.navaccess_id WHERE rna.role_id=? ORDER BY na.position")
                .bind(&role.0)
                .fetch_all(&mut conn)
                .await?;
            role_navaccess.insert(role.0, models);
        }
        Ok(role_navaccess)
    }
}
