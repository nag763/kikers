//! Tokens are redis entities used to call the remote API provider.
//!
//! They are stored entirely within the redis database.
//!
//! Since the API provider is using a freemium model, this struct's entity
//! is rolling the tokens so that the one that still has the most calls is
//! used.
//!
//! The following process is then existing when a call to an API is done :
//! 1. A token is requested by the crate through [Entity::get_token].
//! 2. [Entity::get_token] retrieves which token has the most calls remaining.
//! 3. The token is returned to the crate
//! 4. The crate with its call update the number of call the token he used can
//! be still done with [Entity::update_threshold]

use crate::database::Database;
use crate::error::ApplicationError;

pub struct Entity;

impl Entity {
    /// Register a new token within the redis database.
    ///
    /// Every token is stored within a ZSET with an initial threshold of 100.
    ///
    /// # Arguments
    ///
    /// - token : The token to register.
    #[cfg(feature = "cli")]
    pub fn register(token: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("ZADD")
            .arg("api_token")
            .arg(100)
            .arg(token)
            .query(&mut conn)?;
        Ok(())
    }

    /// Retrieves the token with the most calls remaining.
    ///
    /// This method gets the most from the fact that the token is stored within
    /// a redis ZSET. With the help of the method ZRANGE, we can retrieve very
    /// quickly which token has the most calls remaining.
    #[cfg(feature = "cli")]
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

    /// Updates the number of calls remaining for a token before it exceeds
    /// its threshold.
    ///
    /// Given the API used by this app is in a freemium model, the threshold has
    /// to be updated pretty frequently.
    ///
    /// # Arguments
    ///
    /// - token : the token whose threshold has to be updated.
    /// - threshold : the new threshold.
    #[cfg(feature = "cli")]
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
        debug!("Threshold for token updated to {}", threshold);
        Ok(())
    }
}
