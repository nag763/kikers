use crate::common_api_structs::League;
use crate::country::Model as Country;
use crate::database::Database;
use crate::error::ApplicationError;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Model {
    pub league: League,
    pub country: Country,
}

pub struct Entity;

impl Entity {
    fn get_all() -> Result<Vec<Model>, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let model_as_string: String = redis::cmd("GET").arg("leagues").query(&mut conn)?;
        let model = serde_json::from_str(model_as_string.as_str())?;
        Ok(model)
    }

    pub fn get_fav_leagues_of_user(
        fav_leagues_id: Vec<u32>,
    ) -> Result<Vec<Model>, ApplicationError> {
        let models: Vec<Model> = Self::get_all()?;
        let fav_leagues: Vec<Model> = models
            .into_iter()
            .filter(|model| fav_leagues_id.contains(&model.league.id))
            .collect();
        Ok(fav_leagues)
    }

    pub fn get_leagues_for_country_code(
        country_code: &str,
    ) -> Result<Vec<Model>, ApplicationError> {
        let models: Vec<Model> = Self::get_all()?;
        let leagues: Vec<Model> = match country_code.is_empty() {
            true => models
                .into_iter()
                .filter(|league| league.country.code.is_none())
                .collect(),
            false => models
                .into_iter()
                .filter(|league| {
                    if let Some(code) = &league.country.code {
                        code == country_code
                    } else {
                        false
                    }
                })
                .collect(),
        };
        Ok(leagues)
    }

    pub fn store(value: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("SET")
            .arg("leagues")
            .arg(value)
            .query(&mut conn)?;
        Ok(())
    }
}
