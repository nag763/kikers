use crate::database::Database;
use crate::error::ApplicationError;
use futures::StreamExt;
use mongodb::bson::doc;
use futures::TryStreamExt;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Model {
    pub id: u32,
    pub name: String,
    pub logo: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct ModelLogo {
    logo: Option<String>,
}

pub struct Entity;

impl Entity {

    pub async fn find_all() -> Result<Vec<Model>, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models : Vec<Model> = database.collection::<Model>("club")
            .find(doc!{}, None)
            .await?
            .try_collect()
            .await?;
        Ok(models)
    }

    pub async fn store() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let updatable = database.collection::<Model>("club");
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        let home_teams = database
            .collection::<Model>("fixture")
            .aggregate([doc! {"$replaceRoot": {"newRoot": "$teams.home"}}], None)
            .await?;
        let away_teams = database
            .collection::<Model>("fixture")
            .aggregate([doc! {"$replaceRoot": {"newRoot": "$teams.away"}}], None)
            .await?;

        Self::upsert_list_of_docs(home_teams, updatable.clone(), update_options.clone()).await?;
        Self::upsert_list_of_docs(away_teams, updatable.clone(), update_options.clone()).await?;
        Ok(())
    }

    async fn upsert_list_of_docs(mut docs: mongodb::Cursor<bson::Document>, updatable: mongodb::Collection<Model>, update_options: mongodb::options::UpdateOptions) -> Result<(), ApplicationError> {
        while let Some(result) = docs.next().await { 
            let doc: Model = bson::from_document(result?)?;
            updatable.update_one(               
                doc!{"id":doc.id},
                doc!{"$set": bson::to_bson(&doc)?},
                Some(update_options.clone())
            ).await?;
        }
        Ok(())
    }
}
