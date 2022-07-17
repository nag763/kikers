use ffb_structs::error::ApplicationError as StructError;

/**
 * The common errors that can be thrown while generating a token.
 */
#[derive(Debug, Display)]
pub enum ApplicationError {
    #[display(
        fmt = "Your authentication token is not correct, please reconnect in order to regenarate it"
    )]
    /// The token is illegal or became illegal
    IllegalToken,
    #[display(fmt = "{}", _0)]
    /// A structure error
    StructError(String),
    #[display(fmt = "The user hasn't been found")]
    /// User not found
    NotFound,
    #[display(
        fmt = "The user : {}'s access has either been revoked or not granted",
        _0
    )]
    /// The user hasn't been authorized
    UserNotAuthorized(String),
    #[display(fmt = "An internal error happened")]
    /// Internal error
    InternalError,
}

impl ApplicationError {
    /**
     * Returns an http error code for the given enum.
     */
    pub fn http_error_code(&self) -> u16 {
        match *self {
            ApplicationError::UserNotAuthorized(_) => 403,
            ApplicationError::NotFound => 404,
            _ => 500,
        }
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
