use std::fmt;

#[derive(Debug)]
pub enum CliError {
    VarError(String),
    RequestError(String),
    StructError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &*self {
                CliError::VarError(reason) => reason,
                CliError::RequestError(reason) => reason,
                CliError::StructError(reason) => reason,
            }
        )
    }
}

impl std::error::Error for CliError {}

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

impl From<ffb_structs::error::ApplicationError> for CliError {
    fn from(struct_err: ffb_structs::error::ApplicationError) -> Self {
        Self::StructError(struct_err.to_string())
    }
}
