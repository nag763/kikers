use ffb_structs::error::ApplicationError as StructError;
use std::fmt;

#[derive(Debug)]
pub enum ApplicationError {
    IllegalToken,
    StructError(String),
    NotFound,
    UserNotAuthorized(String),
    InternalError,
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let reason : String = match &*self {
            Self::IllegalToken => "Your authentication token is not correct, please reconnect in order to regenarate it".into(),
            Self::NotFound => "The user hasn't been found".into(),
            Self::StructError(err) => err.into(),
            Self::UserNotAuthorized(login) => format!("The user : {}'s access has either been revoked or not granted", login).into(),
            Self::InternalError => "An internal error happened".into(),
        };
        write!(f, "{}", reason)
    }
}

impl From<jwt::Error> for ApplicationError {
    fn from(jwt_err: jwt::Error) -> Self {
        error!("A jwt error happened : {}", jwt_err.to_string());
        ApplicationError::IllegalToken
    }
}

impl From<std::env::VarError> for ApplicationError {
    fn from(var_error: std::env::VarError) -> Self {
        error!("A var error happened : {}", var_error.to_string());
        ApplicationError::InternalError
    }
}

impl From<hmac::digest::InvalidLength> for ApplicationError {
    fn from(digest_err: hmac::digest::InvalidLength) -> Self {
        error!("An error with the jwt digest happened : {}", digest_err);
        ApplicationError::InternalError
    }
}

impl From<StructError> for ApplicationError {
    fn from(struct_error: StructError) -> Self {
        ApplicationError::StructError(struct_error.to_string())
    }
}
