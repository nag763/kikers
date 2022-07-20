use crate::error::ApplicationError;
use crate::uri_builder::{MessageType, UriBuilder};
use actix_web::http::Uri;
use actix_web::{post, HttpRequest, HttpResponse};
use ffb_auth::JwtUser;
use ffb_structs::user;

#[derive(serde::Deserialize, validator::Validate)]
pub struct ClubStatusUpdate {
    name: String,
    club_id: u32,
    user_id: u32,
    action: String,
}

#[post("/profile/clubs")]
pub async fn update_club_status(
    req: HttpRequest,
    club_status: actix_web_validator::Form<ClubStatusUpdate>,
) -> Result<HttpResponse, ApplicationError> {
    let referer: &str = req
        .headers()
        .get("referer")
        .ok_or(ApplicationError::InternalError)?
        .to_str()?;
    let mut uri_builder: UriBuilder = UriBuilder::from_existing_uri(referer.parse::<Uri>()?);
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    if jwt_user.id != club_status.user_id {
        return Err(ApplicationError::BadRequest);
    }
    match club_status.action.as_str() {
        "add" => {
            let result: bool =
                user::Entity::add_club_as_favorite(club_status.user_id, club_status.club_id)
                    .await?
                    .into();
            if result {
                uri_builder.append_msg(
                    MessageType::Info,
                    &format!("Club {} has been added to favorites", &club_status.name),
                );
            } else {
                uri_builder.append_msg(
                    MessageType::Error,
                    "An error happened while updating the club status",
                );
            }
        }
        "remove" => {
            let result: bool =
                user::Entity::remove_club_as_favorite(club_status.user_id, club_status.club_id)
                    .await?
                    .into();

            if result {
                uri_builder.append_msg(
                    MessageType::Info,
                    &format!("Club {} has been removed from favorites", &club_status.name),
                );
            } else {
                uri_builder.append_msg(
                    MessageType::Error,
                    "An error happened while updating the club status",
                );
            }
        }
        _ => return Err(ApplicationError::BadRequest),
    };
    Ok(HttpResponse::Found()
        .append_header(("Location", uri_builder.build()))
        .finish())
}
