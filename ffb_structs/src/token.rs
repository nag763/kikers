use crate::database::Database;
use crate::error::ApplicationError;

const ONE_WEEK_IN_SECONDS: u32 = 604_800;

pub struct Entity;

impl Entity {
    pub fn register(login: &str, token: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("SET")
            .arg(format!("token::{}::{}", login, token))
            .arg(true)
            .arg("EX")
            .arg(ONE_WEEK_IN_SECONDS)
            .query(&mut conn)?;
        Ok(())
    }

    pub fn revoke_token(login: &str, token: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("DEL")
            .arg(format!("token::{}::{}", login, token))
            .arg(token)
            .query(&mut conn)?;
        Ok(())
    }

    pub fn revoke_all(login: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let keys: String = format!(r#"token::{}:*"#, login);
        let keys_to_del: Vec<String> = redis::cmd("KEYS").arg(&keys).query(&mut conn)?;
        if !keys_to_del.is_empty() {
            redis::cmd("DEL").arg(keys_to_del).query(&mut conn)?;
        }
        Ok(())
    }

    pub fn verify(login: &str, token: &str) -> Result<bool, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let result: bool = redis::cmd("GET")
            .arg(format!("token::{}::{}", login, token))
            .query(&mut conn)?;
        Ok(result)
    }
}
