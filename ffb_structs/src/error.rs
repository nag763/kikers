use std::fmt;

#[derive(Debug)]
pub enum ApplicationError {
    DatabaseError(String),
    RedisError(String),
    MongoError(String),
    ElasticError(String),
    TranslationError(String, u32),
    SerialError,
    NoTokenStored,
    ParseError(String),
    FormOutdated,
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let reason : String = match &*self {
//TODO:            impl format derive more here
            Self::DatabaseError(db_err) => format!("A database error happened, it has been reported and will be resolved as soon as possible : {} ", db_err) ,
            Self::MongoError(db_err) => format!("A mongo error happened, it has been reported and will be resolved as soon as possible : {} ", db_err) ,
            Self::RedisError(redis_err) => format!("A redis error happened, it has been reported and will be resolved as soon as possible : {} ", redis_err) ,
            Self::ElasticError(elastic_err) => format!("An elasticsearch error happened, it has been reported and will be resolved as soon as possible : {} ", elastic_err),
            Self::TranslationError(label_name, locale_id) => format!("A translatione error happened : the label {} has been request for locale {} but this mapping doesn't exist.", label_name, locale_id),
            Self::SerialError => "A serial error happened".into(),
            Self::NoTokenStored => "There are no tokens stored to call the remote API endpoint".into(),
            Self::ParseError(err)=> format!("A parse error happened : {}", err),
            Self::FormOutdated => "The request that has been submitted is most likely using expired parameters and is thus not valid, please refresh your browser".into()
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
        error!("A redis error happened : {:?}", redis_error);
        ApplicationError::RedisError(redis_error.to_string())
    }
}

impl From<mongodb::error::Error> for ApplicationError {
    fn from(mongo_error: mongodb::error::Error) -> Self {
        error!("A mongo error happened : {}", mongo_error);
        ApplicationError::MongoError(mongo_error.to_string())
    }
}

impl From<bson::ser::Error> for ApplicationError {
    fn from(mongo_error: bson::ser::Error) -> Self {
        error!("A bson serialization error happened : {}", mongo_error);
        ApplicationError::MongoError(mongo_error.to_string())
    }
}

impl From<bson::de::Error> for ApplicationError {
    fn from(mongo_error: bson::de::Error) -> Self {
        error!("A bson deserialization error happened : {}", mongo_error);
        ApplicationError::MongoError(mongo_error.to_string())
    }
}

impl From<elasticsearch::Error> for ApplicationError {
    fn from(elastic_err: elasticsearch::Error) -> Self {
        error!("An elasticsearch error happened : {}", elastic_err);
        ApplicationError::ElasticError(elastic_err.to_string())
    }
}

impl From<std::num::ParseFloatError> for ApplicationError {
    fn from(parse_float_err: std::num::ParseFloatError) -> Self {
        error!("A float parsing error happened : {}", parse_float_err);
        ApplicationError::ParseError(parse_float_err.to_string())
    }
}
