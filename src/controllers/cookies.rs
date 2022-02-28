use crate::error::ApplicationError;
use actix_web::http::Cookie;
use actix_web::{get, HttpResponse, Responder, HttpRequest};

#[get("/cookies_approved")]
pub async fn cookies_approved(req: HttpRequest) -> Result<impl Responder, ApplicationError> {
    info!("Peer {:?} approved cookie usage", req.peer_addr());
    Ok(HttpResponse::Found()
        .header("Location", "/")
        .cookie(Cookie::new(std::env::var("COOKIE_APPROVAL_PATH")?, "1"))
        .finish())
}
