use crate::database::Database;
use crate::error::ApplicationError;

pub struct Entity;

impl Entity {
    pub fn register(login: &str, token: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("SADD")
            .arg(format!("token:{}", login))
            .arg(token)
            .query(&mut conn)?;
        Ok(())
    }

    pub fn revoke_token(login: &str, token: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("SREM")
            .arg(format!("token:{}", login))
            .arg(token)
            .query(&mut conn)?;
        Ok(())
    }

    pub fn revoke_all(login: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("DEL")
            .arg(format!("token:{}", login))
            .query(&mut conn)?;
        Ok(())
    }

    pub fn verify(login: &str, token: &str) -> Result<bool, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let result: bool = redis::cmd("SISMEMBER")
            .arg(format!("token:{}", login))
            .arg(token)
            .query(&mut conn)?;
        Ok(result)
    }
}
