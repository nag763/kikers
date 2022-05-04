use crate::database::Database;
use crate::error::ApplicationError;

const KEY: &str = "client_errors";

pub struct Entity;

impl Entity {
    pub fn register_client_error(ip: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("HINCRBY")
            .arg(KEY)
            .arg(ip)
            .arg(1)
            .query(&mut conn)?;

        Ok(())
    }

    pub fn is_ip_banned(ip: &str) -> Result<bool, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let client_errors: Option<u32> = redis::cmd("HGET").arg(KEY).arg(ip).query(&mut conn)?;
        match client_errors {
            Some(v) if 15 < v => Ok(true),
            _ => Ok(false),
        }
    }
}
