use ffb_structs::error::ApplicationError as StructError;

#[derive(Debug, Display)]
pub enum ApplicationError {
    #[display(
        fmt = "Your authentication token is not correct, please reconnect in order to regenarate it"
    )]
    IllegalToken,
    #[display(fmt = "{}", _0)]
    StructError(String),
    #[display(fmt = "The user hasn't been found")]
    NotFound,
    #[display(
        fmt = "The user : {}'s access has either been revoked or not granted",
        _0
    )]
    UserNotAuthorized(String),
    #[display(fmt = "An internal error happened")]
    InternalError,
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
