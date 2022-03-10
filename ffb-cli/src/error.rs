use std::fmt;

#[derive(Debug)]
pub enum CliError {
    RedisError(String),
    VarError(String),
    RequestError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &*self {
                CliError::RedisError(reason) => reason,
                CliError::VarError(reason) => reason,
                CliError::RequestError(reason) => reason,
            }
        )
    }
}

impl std::error::Error for CliError {}

impl From<redis::RedisError> for CliError {
    fn from(err: redis::RedisError) -> Self {
        Self::RedisError(err.to_string())
    }
}

impl From<std::env::VarError> for CliError {
    fn from(err: std::env::VarError) -> Self {
        Self::VarError(err.to_string())
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestError(err.to_string())
    }
}
