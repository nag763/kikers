use crate::error::ApplicationError;
use crate::ApplicationData;
use actix_web::cookie::Cookie;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

#[get("/cookies_approved")]
pub async fn cookies_approved(
    req: HttpRequest,
    app_data: web::Data<ApplicationData>,
) -> Result<impl Responder, ApplicationError> {
    info!("Peer {:?} approved cookie usage", req.peer_addr());
    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .cookie(Cookie::new(app_data.get_cookie_approval_path(), "1"))
        .finish())
}
