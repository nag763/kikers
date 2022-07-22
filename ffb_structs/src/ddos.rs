//! Redis entity to limit DDOS.
//!
//! This entity's purpose is to mitigate the possibility of a DDoS. For that,
//! every client that generates an error (4XX) got his IP registered and then
//! banned once it exceeds the threshold.

use crate::database::Database;
use crate::error::ApplicationError;

const KEY: &str = "client_errors";

pub struct Entity;

impl Entity {
    /// Register a client error.
    ///
    /// The IP is stored within an HSET that associates the IP with a number
    /// of client errros.
    ///
    /// **Warning** : Since the app is ran behind a reverse proxy, the real
    /// ip transmitted by the proxy is used.
    ///
    /// # Arguments
    ///
    /// - ip : The IP which has to register an error
    pub fn register_client_error(ip: &str) -> Result<(), ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        redis::cmd("HINCRBY")
            .arg(KEY)
            .arg(ip)
            .arg(1)
            .query(&mut conn)?;
        debug!("A client error has been registered for {}", ip);
        Ok(())
    }

    /// Looks up whether an IP is already banned from the system.
    ///
    /// The threshold is defined to 15 client errors before getting banned.
    ///
    /// # Arguments
    ///
    /// - ip : The ip to check whether it is banned or not.
    pub fn is_ip_banned(ip: &str) -> Result<bool, ApplicationError> {
        let mut conn = Database::acquire_redis_connection()?;
        let client_errors: Option<u32> = redis::cmd("HGET").arg(KEY).arg(ip).query(&mut conn)?;
        match client_errors {
            Some(v) if 15 < v => Ok(true),
            _ => Ok(false),
        }
    }
}
