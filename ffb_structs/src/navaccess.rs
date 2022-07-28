//! A navaccess represents a navigable link accessible within the application.
//!
//! These are restricted inside the application. All users, given their profile
//! , don't have access to all the parts of the applications.
//!
//! For instance, an unauthenticated user shouldn't be able to access the admin
//! board.
//!
//! This entity should be fetched once at the application startup, if possible
//! refreshed regulary.

use crate::database::Database;
use crate::error::ApplicationError;
use crate::{role, role::Model as Role};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display, sqlx::FromRow, Hash)]
#[display(fmt = "{}=>{}(positioned at {:?}", label, href, position)]
pub struct Model {
    /// The model's ID within the DB.
    pub id: u32,
    /// The label of the navacess.
    ///
    /// The label is whether a reference to a label in the table label for 
    /// translation matters. Otherwise, the label is only here to indicate
    /// the purpose of the navaccess.
    pub label: String,
    /// The logo as a SVG path value.
    pub logo: Option<String>,
    /// The absolute hyper reference.
    ///
    /// Such as /my/link/path and not link/path
    pub href: String,
    /// The position within the navbar.
    pub position: Option<u32>,
}

pub struct Entity;

impl Entity {

    /// Returns the navaccesses linked with a role id.
    ///
    /// # Arguments
    ///
    /// * id : the role id we want to get the navaccess for.
    pub async fn get_navaccess_for_role_id(id: u32) -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let models: Vec<Model> = sqlx::query_as("SELECT * FROM NAVACCESS na INNER JOIN ROLE_NAVACCESS rna ON na.id = rna.navaccess_id WHERE rna.role_id=? ORDER BY na.position").bind(&id).fetch_all(&mut conn).await?;
        Ok(models)
    }

    /// Get the whole navacces mapping.
    ///
    /// Each role has a set of navaccess attributed. This method will return 
    /// the list of navaccess for each role.
    pub async fn get_role_navaccess_mapping() -> Result<HashMap<Role, Vec<Model>>, ApplicationError>
    {
        let roles: Vec<Role> = role::Entity::get_roles().await?;
        let mut role_navaccess = HashMap::new();

        for role in roles {
            let models = Self::get_navaccess_for_role_id(role.id).await?;
            role_navaccess.insert(role, models);
        }
        Ok(role_navaccess)
    }
}
