use crate::error::ApplicationError;
use crate::uri_builder::{MessageType, UriBuilder};
use actix_web::http::Uri;
use actix_web::{post, HttpRequest, HttpResponse};
use ffb_structs::bookmaker;

#[derive(serde::Deserialize, validator::Validate)]
pub struct MainBookmakerUpdate {
    id: u32,
}

#[post("/admin/bookmakers")]
pub async fn admin_bookmakers(
    req: HttpRequest,
    bookmarker_update_form: actix_web_validator::Form<MainBookmakerUpdate>,
) -> Result<HttpResponse, ApplicationError> {
    let referer: &str = req
        .headers()
        .get("referer")
        .ok_or(ApplicationError::InternalError)?
        .to_str()?;
    let mut uri_builder: UriBuilder = UriBuilder::from_existing_uri(referer.parse::<Uri>()?);
    let result = bookmaker::Entity::set_main_bookmaker(bookmarker_update_form.id).await?;
    if result.into() {
        uri_builder.append_msg(MessageType::INFO, "The default bookmaker has been updated");
    } else {
        uri_builder.append_msg(
            MessageType::ERROR,
            "An error happened during the update, the bookmaker hasn't been updated",
        );
    }
    Ok(HttpResponse::Found()
        .append_header(("Location", uri_builder.build()))
        .finish())
}
