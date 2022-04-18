use crate::common_api_structs::Game;
use crate::database::Database;
use crate::error::ApplicationError;

#[derive(serde::Deserialize, Clone)]
pub struct Model {
    pub games: Vec<Game>,
    pub fetched_on: Option<String>,
}

pub struct Entity;

impl Entity {
    pub fn find_all_for_date(
        date: &str,
        fav_leagues: Option<Vec<u32>>,
    ) -> Result<Option<Model>, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let model_as_string: Option<String> = redis::cmd("HGET")
            .arg("fixtures")
            .arg(&date)
            .query(&mut conn)?;
        match model_as_string {
            Some(v) => {
                let mut model: Model = serde_json::from_str(v.as_str())?;
                if let Some(fav_leagues) = fav_leagues {
                    model.games = model
                        .games
                        .into_iter()
                        .filter(|game| fav_leagues.contains(&game.league.id))
                        .collect();
                }
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    pub fn find_all_for_date_truncate_games(
        date: &str,
        fav_leagues: Option<Vec<u32>>,
        limit: usize,
    ) -> Result<Option<Model>, ApplicationError> {
        let games = Self::find_all_for_date(date, fav_leagues)?;
        match games {
            Some(mut v) => {
                v.games.truncate(limit);
                Ok(Some(v))
            }
            None => Ok(None),
        }
    }

    pub fn store(date: &str, value: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("HSET")
            .arg("fixtures")
            .arg(date)
            .arg(value)
            .query(&mut conn)?;
        Ok(())
    }
}
