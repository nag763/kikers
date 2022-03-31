lazy_static! {
    static ref DB_URL: String = std::env::var("DATABASE_URL").unwrap();
    static ref REDIS_URL: String = std::env::var("REDIS_URL").unwrap();
}

pub struct Database;

impl Database {
    pub async fn acquire_sql_connection() -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
        sea_orm::Database::connect(DB_URL.as_str()).await
    }

    pub fn acquire_redis_connection() -> Result<redis::Connection, redis::RedisError> {
        let client = redis::Client::open(REDIS_URL.as_str())?;
        Ok(client.get_connection()?)
    }
}
