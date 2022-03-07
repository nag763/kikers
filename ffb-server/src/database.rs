lazy_static!(
     static ref DB_URL : String = std::env::var("DATABASE_URL").unwrap();
);

pub struct Database;

impl Database {
    
    pub async fn acquire_sql_connection() -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
        sea_orm::Database::connect(DB_URL.as_str()).await
    }

    pub fn acquire_redis_connection() -> Result<redis::Connection, redis::RedisError> {
        let client = redis::Client::open("redis://127.0.0.1/")?;
        Ok(client.get_connection()?)
    }
}
