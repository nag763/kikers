//! A locale is a MySQL structure representing a language variant.
//!
//! For instance there are differences between the portuguese from Portugal and
//! Brazil. These differences have to be understood to provide the best UX 
//! possible.
//!
//! They are mainly used to translate the application and provide a better user
//! experience. 
//!
//! Every application label can have a different translation for each language.
//!
//! By default, if a locale doesn't have a translation for a given label, the
//! default translation will be used.

use crate::database::Database;
use crate::error::ApplicationError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    /// The inbase id.
    pub id: u32,
    /// The language from which the locale is from.
    ///
    /// For instance fr_FR has for main language French.
    pub language_id: u32,
    /// The short name.
    ///
    /// ie. fr_BE, fr_FR, fr_CA, ...
    pub short_name: String,
    /// Long name.
    ///
    /// ie. "français (Belgique)", "français (France)", "français (Canada)", 
    /// ...
    pub long_name: String,
}

pub struct Entity;

impl Entity {
    /// Get all the locales stored in database.
    pub async fn get_locales() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let models: Vec<Model> = sqlx::query_as("SELECT * FROM LOCALE ORDER BY long_name")
            .fetch_all(&mut conn)
            .await?;
        Ok(models)
    }
}
