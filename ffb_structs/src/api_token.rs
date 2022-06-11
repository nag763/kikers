use crate::database::Database;
use crate::error::ApplicationError;

pub struct Entity;

impl Entity {
    pub fn register(token: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("ZADD")
            .arg("api_token")
            .arg(100)
            .arg(token)
            .query(&mut conn)?;
        Ok(())
    }

    pub fn get_token() -> Result<String, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let result: Vec<String> = redis::cmd("ZRANGE")
            .arg("api_token")
            .arg(-1)
            .arg(-1)
            .query(&mut conn)?;
        let result: String = result
            .get(0)
            .ok_or(ApplicationError::NoTokenStored)?
            .clone();
        Ok(result)
    }

    pub fn update_threshold(token: &str, threshold: i32) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("ZREM")
            .arg("api_token")
            .arg(token)
            .query(&mut conn)?;
        redis::cmd("ZADD")
            .arg("api_token")
            .arg(threshold)
            .arg(token)
            .query(&mut conn)?;
        Ok(())
    }
}
