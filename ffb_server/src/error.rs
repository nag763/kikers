use actix_web::{http::header, http::StatusCode, HttpResponse, HttpResponseBuilder};
use askama::Template;

use serde::ser::StdError;
use ffb_auth::error::ApplicationError as AuthApplicationError;
use ffb_structs::error::ApplicationError as StructApplicationError;

#[derive(Debug, Display)]
pub enum ApplicationError {
    #[display(
        fmt = "An internal error happened, it has been reported and will be resolved as soon as possible"
    )]
    InternalError,
    #[display(
        fmt = "One of the requested item hasn't been found, please ensure your request is correct"
    )]
    NotFound,
    #[display(
        fmt = "Your authentication token is not correct, please reconnect in order to regenarate it"
    )]
    IllegalToken,
    #[display(
        fmt = "You don't have access to this ressource, or the way you are trying to access it is wrong"
    )]
    BadRequest,
    #[display(fmt = "An error happened when trying to display, this error has been reported")]
    TemplateError,
    #[display(
        fmt = "You haven't approved cookies yet, approve them prior any usage of the application"
    )]
    CookiesUnapproved,
    #[display(
        fmt = "Following too many bad requests on your side, your IP {} has been banned. If you believe it is an error, please contact the administrator of the site.",
        _0
    )]
    PeerBanned(String),
    #[display(fmt = "{}", _0)]
    StructError(String),
    #[display(fmt = "{}", _0)]
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
            Self::AuthError(_) | Self::IllegalToken => Some("/logout".into()),
            Self::CookiesUnapproved => Some("/cookies".into()),
            Self::PeerBanned(_) => Some("https://google.com/".into()),
            _ => None,
        }
    }
}

impl StdError for ApplicationError {

}

impl actix_web::error::ResponseError for ApplicationError {
    fn error_response(&self) -> HttpResponse {
        let error_page = Error {
            status_code: self.status_code().to_string(),
            error_desc: self.to_string(),
            redirect_url: self.redirect_url(),
        };
        let builder = HttpResponseBuilder::new(self.status_code())
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .body(error_page.render().unwrap());
        builder
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ApplicationError::InternalError
            | ApplicationError::TemplateError
            | ApplicationError::StructError(_)
            | ApplicationError::AuthError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::IllegalToken
            | ApplicationError::CookiesUnapproved
            | ApplicationError::BadRequest => StatusCode::BAD_REQUEST,
            ApplicationError::NotFound => StatusCode::NOT_FOUND,
            ApplicationError::PeerBanned(_) => StatusCode::FORBIDDEN,
        }
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

impl From<actix_web::http::header::ToStrError> for ApplicationError {
    fn from(to_str_err: actix_web::http::header::ToStrError) -> Self {
        error!(
            "A header deserialization error happened : {}",
            to_str_err.to_string()
        );
        Self::InternalError
    }
}

impl From<actix_web::http::uri::InvalidUri> for ApplicationError {
    fn from(invalid_uri_err: actix_web::http::uri::InvalidUri) -> Self {
        error!(
            "An invalid uri has been given : {}",
            invalid_uri_err.to_string()
        );
        Self::InternalError
    }
}
