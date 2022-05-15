use crate::error::ApplicationError;
use actix_web::{post, HttpRequest, HttpResponse};
use ffb_structs::game;

#[derive(serde::Deserialize, validator::Validate)]
pub struct ChangeGameBetStatus {
    id: u32,
    value: bool,
    date: String,
}

#[post("/games/update/status")]
pub async fn update_game_status(
    req: HttpRequest,
    game_status: actix_web_validator::Form<ChangeGameBetStatus>,
) -> Result<HttpResponse, ApplicationError> {
    let result: bool =
        game::Entity::change_is_bet_status(game_status.id, game_status.value, &game_status.date)
            .await?
            .into();
    let referer: &str = req
        .headers()
        .get("referer")
        .ok_or(ApplicationError::InternalError)?
        .to_str()?;
    Ok(HttpResponse::Found()
        .append_header(("Location", referer))
        .finish())
}
