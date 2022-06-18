use crate::error::ApplicationError;
use crate::uri_builder::{MessageType, UriBuilder};
use actix_web::http::Uri;
use actix_web::{post, HttpRequest, HttpResponse};
use ffb_structs::game;

#[derive(serde::Deserialize, validator::Validate)]
pub struct ChangeGameBetStatus {
    id: u32,
    value: bool,
}

#[post("/games/update/status")]
pub async fn update_game_status(
    req: HttpRequest,
    game_status: actix_web_validator::Form<ChangeGameBetStatus>,
) -> Result<HttpResponse, ApplicationError> {
    let result: bool = game::Entity::change_is_bet_status(game_status.id, game_status.value)
        .await?
        .into();
    let referer: &str = req
        .headers()
        .get("referer")
        .ok_or(ApplicationError::InternalError)?
        .to_str()?;
    let mut uri_builder: UriBuilder = UriBuilder::from_existing_uri(referer.parse::<Uri>()?);
    if result {
        let message = match game_status.value {
            true => "The game has been added to the bets",
            false => "The game has been dropped from the bets",
        };
        uri_builder.append_msg(MessageType::INFO, message);
    } else {
        uri_builder.append_msg(MessageType::ERROR,"The game couldn't have been added to the bets, please retry or contact the administrator");
    }
    Ok(HttpResponse::Found()
        .append_header(("Location", uri_builder.build()))
        .finish())
}
