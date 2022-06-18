use crate::bookmaker::Model as Bookmaker;
use crate::database::Database;
use crate::error::ApplicationError;
use crate::{game, game::Model as Game};
use futures::TryStreamExt;
use mongodb::bson::doc;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct SimplifiedFixture {
    id: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct Model {
    fixture: SimplifiedFixture,
    bookmakers: Vec<Bookmaker>,
    processed: Option<bool>,
}

pub struct Entity;

impl Entity {
    pub async fn store(value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        let models: Vec<Model> = serde_json::from_str(value)?;
        for model in models {
            database
                .collection::<Model>("odd")
                .update_one(
                    doc! {"fixture_id": model.fixture.id},
                    doc! {"$set": bson::to_bson(&model)?},
                    update_options.clone(),
                )
                .await?;
        }
        Ok(())
    }

    pub async fn index() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models: Vec<Model> = database
            .collection::<Model>("odd")
            .find(doc! {"processed": {"$ne": true}}, None)
            .await?
            .try_collect()
            .await?;
        for model in models {
            if let Some(bets) = model.bookmakers[0].bets.clone() {
                if bets[0].values.len() == 3 {
                    let home_odd: f32 = bets[0].values[0].odd.parse()?;
                    let draw_odd: f32 = bets[0].values[1].odd.parse()?;
                    let away_odd: f32 = bets[0].values[2].odd.parse()?;
                    database
                .collection::<Game>("fixture")
                .update_one(
                    doc! {"fixture.id": model.fixture.id},
                    doc! {"$set": {"odds": {"home":home_odd, "draw": draw_odd, "away":away_odd}}},
                    None,
                )
                .await?;
                }
            }
        }
        game::Entity::clear_cache()?;
        database
            .collection::<Model>("odd")
            .update_many(doc! {}, doc! {"$set": {"processed": true}}, None)
            .await?;
        Ok(())
    }
}
