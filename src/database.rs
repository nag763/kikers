lazy_static!(
     static ref DB_URL : String = std::env::var("DATABASE_URL").unwrap();
);

pub struct Database;

impl Database {
    pub async fn acquire_connection() -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
        sea_orm::Database::connect(DB_URL.as_str()).await
    }
}
