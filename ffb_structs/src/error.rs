use std::fmt;

#[derive(Debug)]
pub enum ApplicationError {
    DatabaseError(String),
    RedisError(String),
    SerialError,
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let reason : String = match &*self {
            Self::DatabaseError(db_err) => format!("A database error happened, it has been reported and will be resolved as soon as possible : {} ", db_err) ,
            Self::RedisError(redis_err) => format!("A redis error happened, it has been reported and will be resolved as soon as possible : {} ", redis_err) ,
            Self::SerialError => format!("A serial error happened"),
        };
        write!(f, "{}", reason)
    }
}

impl From<serde_json::Error> for ApplicationError {
    fn from(serde_err: serde_json::Error) -> Self {
        error!("A serde error happened : {}", serde_err);
        ApplicationError::SerialError
    }
}

impl From<sqlx::Error> for ApplicationError {
    fn from(sqlx_error: sqlx::Error) -> Self {
        error!("A sqlx error happened : {}", sqlx_error);
        ApplicationError::DatabaseError(sqlx_error.to_string())
    }
}

impl From<redis::RedisError> for ApplicationError {
    fn from(redis_error: redis::RedisError) -> Self {
        error!("A redis error happened : {}", redis_error);
        ApplicationError::RedisError(redis_error.to_string())
    }
}
