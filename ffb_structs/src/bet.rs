//! A bet is a MySQL structure that represents a user's stake on a bet.
//!
//! The common logic behind this struct is that it is stored mainly in MySQL
//! with a replication in Mongo so that once the game result is known, the
//! user either earns 100 times is stake given he had the right results, 0
//! points otherwise.

use crate::database::Database;
use crate::error::ApplicationError;
use crate::game;
use crate::game::Model as Game;
use crate::scoreboard;
use crate::transaction_result::TransactionResult;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

/// A way to modelize the output of a game.
///
/// A game has three possible outcomes given it has finished :
///
/// * Win : the home team has a superior score to the away one.
/// * Loss : the away team has a superior score to the home one.
/// * Draw : both teams scored the same number of goals.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash, sqlx::Type, Display)]
#[repr(u32)]
pub enum GameResult {
    /// Home team has a higher score than the away one.
    Win = 1,
    /// No team scored more goals than the other one.
    Draw = 2,
    /// Away team has a higher score than the home one.
    Loss = 3,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    /// ID of the user who made the bet.
    pub user_id: u32,
    /// The fixture on which the bet stands.
    pub fixture_id: u32,
    /// The user's bet.
    pub result_id: GameResult,
    /// The associated season id.
    pub season_id: u32,
    /// The stake.
    pub stake: f32,
    /// Outcome of the bet.
    ///
    /// If result_id = the real result at the time the user bet is fetched,
    pub outcome: Option<u32>,
}

pub struct Entity;

impl Entity {
    /// Validate the user bets.
    ///
    /// This method is taking the result from the games that haven't been
    /// processed by a subsequent call, and then will add the points to the
    /// users who won the bet.
    ///
    /// The information that this method has been executed is stored within
    /// the fixture structure as [crate::game::Model::processed_as].
    pub async fn validate_bets() -> Result<(), ApplicationError> {
        let database = Database::acquire_mongo_connection().await?;
        let mut conn = Database::acquire_sql_connection().await?;
        let mut total_number_of_rows_updated: u64 = 0;
        // First step : We fitler the fixtures that haven't been processed
        // yet.
        let games: Vec<Game> = database
            .collection::<Game>("fixture")
            .find(
                doc! {
                    "processed_as" : null,
                    "season_id": {"$ne": null},
                    "fixture.status.short" :
                    {
                        "$in":
                            ["FT", "PEN", "AET"]
                    }
                },
                None,
            )
            .await?
            .try_collect()
            .await?;
        // Second step : We iterate over the results.
        for game in games {
            // Third step : We check whether the score at the fulltime is
            // existing or not, and store it within variables.
            if let (Some(game_id), Some(score)) = (game.id, game.score.fulltime) {
                if let (Some(home), Some(away)) = (score.home, score.away) {
                    let result: GameResult = match home - away {
                        v if 0 < v => GameResult::Win,
                        v if v < 0 => GameResult::Loss,
                        _ => GameResult::Draw,
                    };
                    // Fourth step : now that the result is known, we update
                    // the user bets.
                    let update_result = sqlx::query("UPDATE USER_BET SET outcome=IF(result_id=?, stake*100, 0) WHERE fixture_id=?")
                        .bind(result)
                        .bind(game.fixture.id)
                        .execute(&mut conn)
                        .await?;
                    let number_of_rows_updated: u64 = update_result.rows_affected();
                    total_number_of_rows_updated += number_of_rows_updated;
                    // Fifth step : We report the modification within the
                    // mongodb that the game has been processed.
                    database
                        .collection::<Game>("fixture")
                        .update_one(
                            doc! {"_id": game_id},
                            doc! {"$set": {"processed_as": bson::to_bson(&result)?}},
                            None,
                        )
                        .await?;
                    info!("Game id {} has been processed with success with the result {} and {} user bet rows updated", game_id, result, number_of_rows_updated);
                }
            }
        }
        debug!("The bet validaiton process has completed with success");
        debug!("Number of rows updated : {}", total_number_of_rows_updated);
        // Sixth step if appliable : we clear the cache of the leaderboard
        // given the bets have been updated.
        if total_number_of_rows_updated != 0 {
            scoreboard::Entity::clear_cache()?;
        }
        Ok(())
    }

    /// Upsert a bet within the database.
    ///
    /// When a user wants to bet on a fixture that has odds stored and is
    /// indicated as a bet, the user can bet on the game and its outcome
    /// through this method.
    ///
    /// Once this method is called, the result is stored within the SQL DB
    /// besides of being replicated in Mongo.
    ///
    /// # Arguments
    ///
    /// - user_id : the id of the user who makes the bet.
    /// - fixture_id : the id of the fixture the user bets on.
    /// - season_id : the id of the current season, used to know on which
    /// season the outcome of the game will be rattached.
    /// - game_result : the bet of the user on the fixture.
    /// - stake : the original odds on the game.
    pub async fn upsert_bet(
        user_id: u32,
        fixture_id: u32,
        season_id: u32,
        game_result: GameResult,
        stake: f32,
    ) -> Result<TransactionResult, ApplicationError> {
        let now: DateTime<Utc> = Utc::now();
        let database = Database::acquire_mongo_connection().await?;
        // We store the result of the update request since we update a bet
        // a game only and only if it hasn't started. So if a request to make
        // a bet is done further the kickoff, no mongo entity will be updated.
        let mongo_result = database
            .collection::<Model>("fixture")
            .update_one(
                doc! {
                    "fixture.id" : fixture_id,
                    "betters.user_id": user_id,
                    "fixture.timestamp": {
                        "$gte":now.timestamp()
                    }
                },
                doc! {
                    "$set": {
                        "betters.$.game_result": bson::to_bson(&game_result)?
                    }
                },
                None,
            )
            .await?;
        // If no update has been made, we check whether the form was outdated
        // at the time of the request or not.
        if mongo_result.modified_count == 0 {
            let mongo_result = database
                .collection::<Model>("fixture")
                .update_one(
                    doc! {
                        "fixture.id":fixture_id,
                        "fixture.timestamp": {
                            "$gte":now.timestamp()
                        }
                    },
                    doc! {
                        "$addToSet": {
                            "betters" :{
                                "user_id": user_id,
                                "game_result": bson::to_bson(&game_result)?
                            }
                        }
                    },
                    None,
                )
                .await?;
            if mongo_result.matched_count == 0 {
                warn!(
                    "User {} has tried to update a bet after the kickoff",
                    user_id
                );
                return Err(ApplicationError::FormOutdated);
            }
        }
        let mut conn = Database::acquire_sql_connection().await?;
        let result = sqlx::query(
            "INSERT INTO USER_BET(user_id, fixture_id, result_id, season_id, stake) VALUES(?,?,?,?,?) ON DUPLICATE KEY UPDATE result_id=?, stake=?",
        )
        .bind(user_id)
        .bind(fixture_id)
        .bind(&game_result)
        .bind(season_id)
        .bind(stake)
        .bind(&game_result)
        .bind(stake)
        .execute(&mut conn)
        .await?;
        debug!(
            "The user {} has bet on game {} with output {}",
            user_id, fixture_id, game_result
        );
        game::Entity::clear_cache()?;
        Ok(TransactionResult::expect_single_result(
            result.rows_affected(),
        ))
    }
}
