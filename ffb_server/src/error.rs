use actix_web::{dev::HttpResponseBuilder, http::header, http::StatusCode, HttpResponse};
use askama::Template;

use ffb_auth::error::ApplicationError as AuthApplicationError;
use ffb_structs::error::ApplicationError as StructApplicationError;
use std::fmt;

#[derive(Debug)]
pub enum ApplicationError {
    InternalError,
    DatabaseError(String),
    NotFound,
    UserNotAuthorized(String),
    IllegalToken,
    BadRequest,
    TemplateError,
    CookiesUnapproved,
    StructError(String),
    AuthError(String),
}

#[derive(Template, Debug)]
#[template(path = "error.html")]
struct Error {
    status_code: String,
    error_desc: String,
    redirect_url: Option<String>,
}

impl ApplicationError {
    fn redirect_url(&self) -> Option<String> {
        match &*self {
            Self::IllegalToken => Some("/logout".into()),
            Self::CookiesUnapproved => Some("/cookies".into()),
            _ => None,
        }
    }
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let reason : String = match &*self {
            Self::InternalError => "An internal error happened, it has been reported and will be resolved as soon as possible".into(),
            Self::DatabaseError(db_err) => format!("A database error happened, it has been reported and will be resolved as soon as possible : {} ", db_err) ,
            Self::UserNotAuthorized(user) => format!("The following user's access has not been granted or has been revoked : {} ", user) ,
            Self::NotFound => "One of the requested item hasn't been found, please ensure your request is correct".into(),
            Self::BadRequest => "You don't have access to this ressource, or the way you are trying to access it is wrong.".into(),
            Self::IllegalToken => "Your authentication token is not correct, please reconnect in order to regenarate it".into(),
            Self::CookiesUnapproved => "You haven't approved cookies yet, approve them prior any usage of the application".into(),
            Self::TemplateError => "An error happened regarding the display, this error has been reported".into(),
            Self::StructError(err) => err.into(),
            Self::AuthError(err) => err.into(),
        };
        write!(f, "{}", reason)
    }
}

impl actix_web::error::ResponseError for ApplicationError {
    fn error_response(&self) -> HttpResponse {
        let error_page = Error {
            status_code: self.status_code().to_string(),
            error_desc: self.to_string(),
            redirect_url: self.redirect_url(),
        };
        let builder = HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(error_page.render().unwrap());
        builder
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ApplicationError::InternalError
            | ApplicationError::DatabaseError(_)
            | ApplicationError::TemplateError
            | ApplicationError::StructError(_)
            | ApplicationError::AuthError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::IllegalToken
            | ApplicationError::CookiesUnapproved
            | ApplicationError::BadRequest => StatusCode::BAD_REQUEST,
            ApplicationError::UserNotAuthorized(_) => StatusCode::FORBIDDEN,
            ApplicationError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<jwt::Error> for ApplicationError {
    fn from(jwt_err: jwt::Error) -> Self {
        error!("A jwt error happened : {}", jwt_err.to_string());
        ApplicationError::IllegalToken
    }
}

impl From<askama::Error> for ApplicationError {
    fn from(askama_err: askama::Error) -> Self {
        error!("A template error happened : {}", askama_err.to_string());
        ApplicationError::TemplateError
    }
}

impl From<std::env::VarError> for ApplicationError {
    fn from(var_error: std::env::VarError) -> Self {
        error!("A var error happened : {}", var_error.to_string());
        ApplicationError::InternalError
    }
}

impl From<StructApplicationError> for ApplicationError {
    fn from(struct_error: StructApplicationError) -> Self {
        Self::StructError(struct_error.to_string())
    }
}

impl From<AuthApplicationError> for ApplicationError {
    fn from(auth_error: AuthApplicationError) -> Self {
        Self::AuthError(auth_error.to_string())
    }
}
