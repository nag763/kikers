use crate::error::ApplicationError;
use crate::uri_builder::{MessageType, UriBuilder};
use actix_web::http::Uri;
use actix_web::{post, HttpRequest, HttpResponse};
use ffb_structs::{game, bet, bet::GameResult};

#[derive(serde::Deserialize, validator::Validate)]
pub struct ChangeGameGameResultStatus {
    id: u32,
    value: bool,
}

#[post("/games/update/status")]
pub async fn update_game_status(
    req: HttpRequest,
    game_status: actix_web_validator::Form<ChangeGameGameResultStatus>,
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

#[derive(serde::Deserialize, validator::Validate)]
pub struct GameResultOnGameForm {
    user_id: u32,
    fixture_id: u32,
    bet: GameResult,
    stake: f32,
}

#[post("/games/bet")]
pub async fn bet_on_game(
    req: HttpRequest,
    bet_form: actix_web_validator::Form<GameResultOnGameForm>,
) -> Result<HttpResponse, ApplicationError> {
    bet::Entity::upsert_bet(bet_form.user_id, bet_form.fixture_id, bet_form.bet, bet_form.stake).await?;
    let referer: &str = req
        .headers()
        .get("referer")
        .ok_or(ApplicationError::InternalError)?
        .to_str()?;
    let mut uri_builder: UriBuilder = UriBuilder::from_existing_uri(referer.parse::<Uri>()?);
    uri_builder.append_msg(MessageType::INFO, "Your bet has been successfully saved");
    Ok(HttpResponse::Found()
        .append_header(("Location", uri_builder.build()))
        .finish())
}
