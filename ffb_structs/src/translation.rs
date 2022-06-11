use crate::database::Database;
use crate::error::ApplicationError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    pub label_id: u32,
    pub label_name: String,
    pub locale_id: u32,
    pub translation: String,
}

pub struct Entity;

impl Entity {
    pub async fn get_all() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_sql_connection().await?;
        let models : Vec<Model> = sqlx::query_as("SELECT loc.id as 'locale_id',  lbl.id as 'label_id', lbl.name as 'label_name', IF(tra.translation IS NULL, lbl.default_translation, tra.translation) AS 'translation'
FROM LOCALE loc
JOIN LABEL lbl
LEFT OUTER JOIN TRANSLATION tra
ON tra.label_id = lbl.id AND loc.id = tra.locale_id")
            .fetch_all(&mut conn)
            .await?;
        Ok(models)
    }
}
