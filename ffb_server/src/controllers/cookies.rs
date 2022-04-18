use crate::error::ApplicationError;
use actix_web::cookie::Cookie;
use actix_web::{get, HttpRequest, HttpResponse, Responder};

#[get("/cookies_approved")]
pub async fn cookies_approved(req: HttpRequest) -> Result<impl Responder, ApplicationError> {
    info!("Peer {:?} approved cookie usage", req.peer_addr());
    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .cookie(Cookie::new(std::env::var("COOKIE_APPROVAL_PATH")?, "1"))
        .finish())
}
