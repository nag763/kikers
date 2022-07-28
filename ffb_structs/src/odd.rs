//! An odd is an independant MongoDB entity from the fixtures that stores the
//! odds for each games.
//!
//! Odds are stored so that they can be processed later and added to the 
//! fixtures so that user bets on them.
//!
//! They should be fetched once per day and refreshed for the day to come if
//! appliable.

use crate::bookmaker::Model as Bookmaker;
use crate::database::Database;
use crate::error::ApplicationError;
use crate::{game, game::Model as Game};
use futures::TryStreamExt;
use mongodb::bson::doc;

/// Simplified fixture structure so that we don't fetch all the fields.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct SimplifiedFixture {
    id: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct Model {
    /// The fixture to add an odd on.
    fixture: SimplifiedFixture,
    /// The list of bookmakers that have a bet available for the game.
    bookmakers: Vec<Bookmaker>,
    /// Whether the odd has been processed or not.
    processed: Option<bool>,
}

pub struct Entity;

impl Entity {

    /// Stores the serialized structure within the database.
    pub async fn store(value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        debug!("Starting to store the odds within the database");
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
        debug!("Odds stored in the database");
        Ok(())
    }

    /// Index the odds so that they are linked with their actual games.
    pub async fn index() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        debug!("Starting to index the odds");
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
        database
            .collection::<Model>("odd")
            .update_many(doc! {}, doc! {"$set": {"processed": true}}, None)
            .await?;
        game::Entity::clear_cache()?;
        debug!("Odds have been processed with success");
        Ok(())
    }
}
