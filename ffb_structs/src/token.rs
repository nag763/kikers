//! This struture gathers in the Redis cache all the JWT used to authenticate 
//! the users.
//!
//! These tokens expire a week after being emitted, and should either be 
//! revockable has a whole or alone given the use case.
//!
//! They are checked by the middleware before a request is processed, and this
//! entity acts as a white list.
//!
//! A user can have several jwt associed at once, since he can have multiple 
//! sessions, for instance the website being opened on his phone and his 
//! laptop.
//!
//! Each user has a set of unique tokens that belongs to him, this entity 
//! understands that two users can't share the same token and the verifications
//! of token should consider that.

use crate::database::Database;
use crate::error::ApplicationError;

/// Used to have a constant expiracy time.
const ONE_WEEK_IN_SECONDS: u32 = 604_800;

pub struct Entity;

impl Entity {

    /// Registers a token within the whitelist for the given login.
    ///
    /// Once the token is registered, it is stored for a week before being 
    /// cleaned, and thus revocked, by the cache manager.
    ///
    /// # Arguments
    ///
    /// * login : The login assoccied with the token to register.
    /// * token : The token to register along the login. It has to conform the
    /// jwt format agreed within the application.
    pub fn register(login: &str, token: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("SET")
            .arg(format!("token::{}::{}", login, token))
            .arg(true)
            .arg("EX")
            .arg(ONE_WEEK_IN_SECONDS)
            .query(&mut conn)?;
        debug!("A new token for {} has just been registered", login);
        Ok(())
    }

    /// Revokes one token for the given login.
    ///
    /// This can be used for instance to refresh the JWT of the user.
    ///
    /// # Arguments
    ///
    /// * login : The login assoccied with the token to revoke.
    /// * token : The token to revoke.
    pub fn revoke_token(login: &str, token: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("DEL")
            .arg(format!("token::{}::{}", login, token))
            .arg(token)
            .query(&mut conn)?;
        debug!("One token for user {} has just been revoked", login);
        Ok(())
    }

    /// Revokes all the token for a user.
    ///
    /// This can be done in order to force the update globally of a user,
    /// or to simply deauth him.
    ///
    /// # Arguments
    ///
    /// - login : the login of the user whose token needs to be registered.
    pub fn revoke_all(login: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys: String = format!(r#"token::{}:*"#, login);
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg(&keys).query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }
        info!("All the tokens for {} have just been revoked", login);
        Ok(())
    }

    /// Verifies that the token is valid for the given user.
    ///
    /// This can be used by a middleware in order to check that the user is 
    /// a legit user. This doesn't verify the format of the jwt itself.
    ///
    /// # Arguments
    ///
    /// - login : The login of the user.
    /// - token : The token associed  to the user.
    pub fn verify(login: &str, token: &str) -> Result<bool, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let result: bool = redis::cmd("GET")
            .arg(format!("token::{}::{}", login, token))
            .query(&mut conn)?;
        debug!("Token of {} has just been verified", login);
        Ok(result)
    }
}
