pub mod admin;
pub mod unauth;

use crate::auth::JwtUser;
use crate::ApplicationError;
use actix_web::HttpMessage;
use actix_web::HttpRequest;

fn get_jwt_user(req: HttpRequest) -> Result<JwtUser, ApplicationError> {
    match req.cookie(std::env::var("JWT_TOKEN_PATH")?.as_str()) {
        Some(token) => Ok(JwtUser::check_token(token.value())?),
        None => Err(ApplicationError::InternalError),
    }
}

#[derive(serde::Deserialize)]
pub struct ContextQuery {
    info: Option<String>,
    error: Option<String>,
}
