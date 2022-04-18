use crate::error::ApplicationError;

pub(crate) trait CacheManager<T> {

    fn from_cache(key:&str) -> Result<Option<T>, ApplicationError>;
    fn clean_cache(key:&str) -> Result<Option<T>, ApplicationError>;
    fn update_cache(key:&str, value:T) -> Result<Option<T>, ApplicationError>;
    
}
