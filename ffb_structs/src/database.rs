use elasticsearch::{http::transport::Transport, Elasticsearch};
use mongodb::{options::ClientOptions, Client};
use sqlx::Connection;

lazy_static! {
    static ref DB_URL: String = std::env::var("DATABASE_URL").unwrap();
    static ref REDIS_URL: String = std::env::var("REDIS_URL").unwrap();
    static ref MONGO_URL: String = std::env::var("MONGO_URL").unwrap();
    static ref MONGO_DBNAME: String = std::env::var("MONGO_DBNAME").unwrap();
    static ref ELASTIC_HOST: String = std::env::var("ELASTIC_HOST").unwrap();
}

pub(crate) struct Database;

impl Database {
    pub async fn acquire_sql_connection() -> Result<sqlx::mysql::MySqlConnection, sqlx::Error> {
        sqlx::MySqlConnection::connect(DB_URL.as_str()).await
    }

    pub fn acquire_redis_connection() -> Result<redis::Connection, redis::RedisError> {
        let client = redis::Client::open(REDIS_URL.as_str())?;
        client.get_connection()
    }

    pub async fn acquire_mongo_connection() -> Result<mongodb::Database, mongodb::error::Error> {
        let client_options = ClientOptions::parse(MONGO_URL.as_str()).await?;
        Ok(Client::with_options(client_options)?.database(MONGO_DBNAME.as_str()))
    }

    pub async fn acquire_elastic_connection() -> Result<Elasticsearch, elasticsearch::Error> {
        let transport = Transport::single_node(&ELASTIC_HOST)?;
        Ok(Elasticsearch::new(transport))
    }
}
