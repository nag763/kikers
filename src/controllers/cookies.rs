use crate::error::ApplicationError;
use actix_web::http::Cookie;
use actix_web::{get, HttpResponse, Responder};

#[get("/cookies_approved")]
pub async fn cookies_approved() -> Result<impl Responder, ApplicationError> {
    Ok(HttpResponse::Found()
        .header("Location", "/")
        .cookie(Cookie::new(std::env::var("COOKIE_APPROVAL_PATH")?, "1"))
        .finish())
}
