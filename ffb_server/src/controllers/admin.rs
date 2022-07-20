use crate::error::ApplicationError;
use crate::uri_builder::{MessageType, UriBuilder};
use actix_web::http::Uri;
use actix_web::{post, HttpRequest, HttpResponse};
use ffb_structs::{bookmaker, season};

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
        uri_builder.append_msg(MessageType::Info, "The default bookmaker has been updated");
    } else {
        uri_builder.append_msg(
            MessageType::Error,
            "An error happened during the update, the bookmaker hasn't been updated",
        );
    }
    Ok(HttpResponse::Found()
        .append_header(("Location", uri_builder.build()))
        .finish())
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct AddSeason {
    name: String,
}

#[post("/admin/season/add")]
pub async fn admin_season_add(
    req: HttpRequest,
    add_season: actix_web_validator::Form<AddSeason>,
) -> Result<HttpResponse, ApplicationError> {
    let referer: &str = req
        .headers()
        .get("referer")
        .ok_or(ApplicationError::InternalError)?
        .to_str()?;
    let mut uri_builder: UriBuilder = UriBuilder::from_existing_uri(referer.parse::<Uri>()?);
    let result = season::Entity::add_new(&add_season.name).await?;
    if result.into() {
        uri_builder.append_msg(
            MessageType::Info,
            "The season has been added to the list with success",
        );
    } else {
        uri_builder.append_msg(
            MessageType::Error,
            "An error happened during the update, the bookmaker hasn't been updated",
        );
    }
    Ok(HttpResponse::Found()
        .append_header(("Location", uri_builder.build()))
        .finish())
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct SeasonForm {
    id: u32,
}

#[post("/admin/season/set_main")]
pub async fn admin_season_set_main(
    req: HttpRequest,
    season_set_main: actix_web_validator::Form<SeasonForm>,
) -> Result<HttpResponse, ApplicationError> {
    let referer: &str = req
        .headers()
        .get("referer")
        .ok_or(ApplicationError::InternalError)?
        .to_str()?;
    let mut uri_builder: UriBuilder = UriBuilder::from_existing_uri(referer.parse::<Uri>()?);
    season::Entity::set_main(season_set_main.id).await?;
    uri_builder.append_msg(MessageType::Info, "The current season has been updated");
    Ok(HttpResponse::Found()
        .append_header(("Location", uri_builder.build()))
        .finish())
}

#[post("/admin/season/close")]
pub async fn admin_season_close(
    req: HttpRequest,
    season_set_main: actix_web_validator::Form<SeasonForm>,
) -> Result<HttpResponse, ApplicationError> {
    let referer: &str = req
        .headers()
        .get("referer")
        .ok_or(ApplicationError::InternalError)?
        .to_str()?;
    let mut uri_builder: UriBuilder = UriBuilder::from_existing_uri(referer.parse::<Uri>()?);
    let result = season::Entity::close(season_set_main.id).await?;
    if result.into() {
        uri_builder.append_msg(MessageType::Info, "The season has been closed");
    } else {
        uri_builder.append_msg(
            MessageType::Error,
            "An error happened during the update, ensure first that the season isn't the current one, then try again",
        );
    }
    Ok(HttpResponse::Found()
        .append_header(("Location", uri_builder.build()))
        .finish())
}
