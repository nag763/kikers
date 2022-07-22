//! A game is a Mongo stored structure that represent an opposition between
//! two clubs.
//!
//! This structure is the core of the application. A game is what the users
//! will bet on. This structure is fetched regulary given the need from the
//! API provider. For instance, between 13 and 23 CET, the games should be
//! fetched between every 1 and 20 minutes, to keep the scores alive.
//!
//! It is very important to cache the requests since the queries are very
//! ressource consuming. The cache is cleared everytime :
//! * A bet is made by a user.
//! * The game is refreshed.
//! * Odds are fetched.

use crate::bet::GameResult;
#[cfg(feature = "server")]
use crate::common_api_structs::ShortStatus;
use crate::common_api_structs::{Better, Fixture, Goals, Odds, Score, Teams};
use crate::database::Database;
use crate::error::ApplicationError;
use crate::league::Model as League;
#[cfg(feature = "server")]
use crate::transaction_result::TransactionResult;
use bson::oid::ObjectId;
#[cfg(feature = "server")]
use futures::TryStreamExt;
use mongodb::bson::doc;
#[cfg(feature = "server")]
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
#[cfg(feature = "server")]
use std::hash::{Hash, Hasher};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// The ID in the internal mongodatabase.
    ///
    /// Since the mongo ID is automaticly indexed, it is prefered to use it
    /// rather than the remote API id.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// The internal season id associed to the game.
    ///
    /// Given user "A" adds the game to the bets, the fixture will be associed
    /// to the current season.
    ///
    /// This also indicates whether the game is in the bets or not, since if
    /// the field is "None", this means that the game isn't a bet as it is
    /// added to the structure during the process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub season_id: Option<u32>,
    /// Informs if the game given it is a bet has been processed.
    ///
    /// A processed game is a game whose bets have already been validated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed_as: Option<GameResult>,
    /// The odds associed to the bets.
    ///
    /// They should contain the odds for home, draw and away team winning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odds: Option<Odds>,
    /// The list of users who have bet on the game.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub betters: Option<HashSet<Better>>,
    /// The local league logo.
    ///
    /// Must correspond to a local asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub league_local_logo: Option<String>,
    /// The local home team logo.
    ///
    /// Must correspond to a local asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_local_logo: Option<String>,
    /// The local away team logo.
    ///
    /// Must correspond to a local asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub away_local_logo: Option<String>,
    /// The result of the game.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<GameResult>,
    /// The details about the game.
    pub fixture: Fixture,
    /// The associed league to the game.
    pub league: League,
    /// The teams that competed in this game.
    pub teams: Teams,
    /// The goals scored during the game.
    pub goals: Goals,
    /// The final score.
    pub score: Score,
}

#[cfg(feature = "server")]
impl Model {
    /// Whether the game is started or not.
    ///
    /// If the game has been cancelled or postponed, it will return false.
    pub fn is_started(&self) -> bool {
        !matches!(
            self.fixture.status.short,
            ShortStatus::Ns | ShortStatus::Tbd
        )
    }

    /// Whether the game is finished or not.
    ///
    /// If the game has been cancelled or postponed, it will return false.
    pub fn is_finished(&self) -> bool {
        matches!(
            self.fixture.status.short,
            ShortStatus::Ft | ShortStatus::Aet | ShortStatus::Pen
        )
    }

    /// Get the bet associed to the game for the given user id.
    ///
    /// # Argument
    ///
    /// - user_id : The MySQL user ID.
    pub fn get_bet_for_user(&self, user_id: &u32) -> Option<GameResult> {
        if let Some(betters) = &self.betters {
            let filtered_bets: Vec<GameResult> = betters
                .iter()
                .filter(|bet| &bet.user_id == user_id)
                .map(|bet| bet.game_result)
                .collect();
            filtered_bets.get(0).copied()
        } else {
            None
        }
    }
}

pub struct Entity;

impl Entity {
    /// Store the games in the mongo database.
    ///
    /// # Arguments :
    ///
    /// - date : the date of the game.
    /// - value : the struct serialized.
    #[cfg(feature = "cli")]
    pub async fn store(date: &str, value: &str) -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let models: Vec<Model> = serde_json::from_str(value)?;
        debug!("Games have been serialized with success");
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        for model in models {
            database
                .collection::<Model>("fixture")
                .update_one(
                    doc! {"fixture.id":model.fixture.id},
                    doc! {"$set": bson::to_bson(&model)?},
                    Some(update_options.clone()),
                )
                .await?;
        }
        debug!("Games have been upserted successfully");
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("HSET")
            .arg("fixtures_fetch_date")
            .arg(date)
            .arg(chrono::Utc::now().to_rfc3339())
            .query(&mut conn)?;
        debug!("The fetched date has been updated");
        Self::clear_cache()?;
        Ok(())
    }

    /// Indicates that the game is now a bet.
    ///
    /// When the game is a bet, the user can do bets on it.
    ///
    /// # Arguments :
    ///
    /// - id : game id, this corresponds to the fixture's id.
    /// - value : the season id, if none is bet, remove the game from the
    /// bets.
    #[cfg(feature = "server")]
    pub async fn change_is_bet_status(
        id: u32,
        value: Option<u32>,
    ) -> Result<TransactionResult, ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let result = database
            .collection::<Model>("fixture")
            .update_one(
                doc! {"fixture.id": id},
                doc! {"$set":{"season_id":value}},
                None,
            )
            .await?;
        debug!(
            "Game {} 's bet status has succesfully been modified to be with season {:?}",
            id, value
        );
        Self::clear_cache()?;
        Ok(TransactionResult::expect_single_result(
            result.modified_count,
        ))
    }

    /// Get the last time the games have been fetched for the given date.
    ///
    /// # Arguments
    /// - date : date as YYYY-MM-DD format.
    #[cfg(feature = "server")]
    pub fn get_last_fetched_timestamp_for_date(
        date: &str,
    ) -> Result<Option<String>, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let result: Option<String> = redis::cmd("HGET")
            .arg("fixtures_fetch_date")
            .arg(date)
            .query(&mut conn)?;
        Ok(result)
    }

    pub(crate) fn clear_cache() -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg(r#"games:*"#).query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }
        debug!("Games cache has been cleared successfully");
        Ok(())
    }
}

#[cfg(feature = "server")]
#[derive(Default, Hash, Debug)]
pub struct EntityBuilder {
    /// Look up for a specific date.
    ///
    /// Has to be passed as YYYY-MM-DD.
    date: Option<String>,
    /// Restrict results to leagues whose id is contained in this field.
    leagues: Option<Vec<u32>>,
    /// Restrict the results to the games who are contained in this field.
    clubs: Option<Vec<u32>>,
    /// Look up only for bets.
    ///
    /// If true, only bets will be fetched, otherwise no restriction will be
    /// applied.
    bets: bool,
    /// Look up only for potential bets.
    ///
    /// If true, only potential bets will be fetched, otherwise no restriction
    /// will be applied.
    ///
    /// Will also include the bets.
    potential_bets: bool,
    /// Limit the results returned.
    limit: Option<i64>,
}

#[cfg(feature = "server")]
impl EntityBuilder {
    /// Create a new entity builder.
    pub fn build() -> EntityBuilder {
        Self::default()
    }

    /// Look up for a specific date.
    ///
    /// Has to be passed as YYYY-MM-DD.
    pub fn date(&mut self, date: &str) -> &mut Self {
        self.date = Some(date.into());
        self
    }

    /// Restrict results to leagues whose id is contained in this field.
    pub fn leagues(&mut self, leagues: Vec<u32>) -> &mut Self {
        self.leagues = Some(leagues);
        self
    }

    /// Restrict the results to the games who are contained in this field.
    pub fn clubs(&mut self, clubs: Vec<u32>) -> &mut Self {
        self.clubs = Some(clubs);
        self
    }

    /// Look up only for bets.
    ///
    /// If true, only bets will be fetched, otherwise no restriction will be
    /// applied.
    pub fn bets(&mut self, bets: bool) -> &mut Self {
        self.bets = bets;
        self
    }

    /// Look up only for potential bets.
    ///
    /// If true, only potential bets will be fetched, otherwise no restriction
    /// will be applied.
    ///
    /// Will also include the bets.
    pub fn potential_bets(&mut self, potential_bets: bool) -> &mut Self {
        self.potential_bets = potential_bets;
        self
    }

    /// Limit the results.
    pub fn limit(&mut self, limit: i64) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    pub async fn finish(&self) -> Result<Vec<Model>, ApplicationError> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let redis_key: String = format!("games::{:x}", hasher.finish());
        debug!("Lookup {:#?}", &self);
        let mut conn = Database::acquire_redis_connection()?;
        let cached_struct: Option<String> = redis::cmd("GETEX")
            .arg(redis_key.as_str())
            .arg("EX")
            .arg(200)
            .query(&mut conn)?;
        if let Some(cached_struct) = cached_struct {
            let deserialized_struct: Vec<Model> = serde_json::from_str(cached_struct.as_str())?;
            debug!("Model game has been found from cache for the given lookup");
            Ok(deserialized_struct)
        } else {
            debug!("Model game hasn't been found in cache for the given lookup");
            let database = Database::acquire_mongo_connection().await?;
            let options: Option<mongodb::options::FindOptions> = self.limit.map(|v| {
                mongodb::options::FindOptions::builder()
                    .limit(Some(v))
                    .build()
            });
            let mut key: bson::Document = bson::Document::new();
            let mut query_selector: Vec<bson::Document> = vec![];
            if let Some(leagues) = &self.leagues {
                query_selector.push(doc! {"league.id": {"$in": leagues}});
            }
            if let Some(clubs) = &self.clubs {
                query_selector.push(doc! {"teams.home.id" : {"$in" : &clubs}});
                query_selector.push(doc! {"teams.away.id" : {"$in" : &clubs}});
            }
            if self.bets {
                query_selector.push(doc! {"season_id": {"$ne": null}});
            }
            if self.potential_bets {
                query_selector.push(doc! {"odds": {"$ne": null}});
            }
            if let Some(date) = &self.date {
                key.insert("fixture.date", doc! {"$regex" : date});
            }
            if !query_selector.is_empty() {
                key.insert("$or", query_selector);
            }
            let model: Vec<Model> = database
                .collection::<Model>("fixture")
                .find(key, options)
                .await?
                .try_collect()
                .await?;
            redis::cmd("SET")
                .arg(redis_key.as_str())
                .arg(serde_json::to_string(&model)?)
                .arg("EX")
                .arg(100)
                .query(&mut conn)?;
            debug!("The list of models fetched with the entity builder has been successfully returned and stored in cache");
            Ok(model)
        }
    }
}
