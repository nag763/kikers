use ffb_auth::JwtUser;

use crate::error::ApplicationError;
use actix_web::http::Cookie;
use actix_web::HttpMessage;
use actix_web::{post, HttpRequest, HttpResponse, Responder};
use ffb_structs::user;
use ffb_structs::user::Model as User;

#[derive(serde::Deserialize, validator::Validate)]
pub struct UserActivation {
    #[validate(range(min = 0))]
    id: u32,
    value: bool,
    #[validate(range(min = 0))]
    page: i32,
    #[validate(range(min = 0))]
    per_page: i32,
    #[validate(length(min = 2))]
    login: String,
}

#[post("/user/activation")]
pub async fn user_activation(
    req: HttpRequest,
    user_activation_form: actix_web_validator::Form<UserActivation>,
) -> Result<impl Responder, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    user::Entity::change_activation_status(user_activation_form.id, user_activation_form.value)
        .await?;

    if !user_activation_form.value {
        JwtUser::revoke_all_session(&user_activation_form.login)?;
    }
    info!(
        "User {} updated activation status (to {}) of user (#{})",
        jwt_user.login, user_activation_form.value, user_activation_form.id
    );
    Ok(HttpResponse::Found()
        .header(
            "Location",
            format!(
                "/admin?info=User {}'s access has been modified&page={}&per_page={}",
                user_activation_form.login,
                user_activation_form.page,
                user_activation_form.per_page
            ),
        )
        .finish())
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct UserDeletion {
    #[validate(range(min = 0))]
    id: u32,
    #[validate(length(min = 2))]
    login: String,
    #[validate(range(min = 0))]
    page: i32,
    #[validate(range(min = 0))]
    per_page: i32,
}

#[post("/user/deletion")]
pub async fn user_deletion(
    user_deletion_form: actix_web_validator::Form<UserDeletion>,
) -> Result<impl Responder, ApplicationError> {
    user::Entity::delete_user_id(user_deletion_form.id).await?;
    JwtUser::revoke_all_session(&user_deletion_form.login)?;
    Ok(HttpResponse::Found()
        .header(
            "Location",
            format!(
                "/admin?info=User {} has been deleted&page={}&per_page={}",
                user_deletion_form.login, user_deletion_form.page, user_deletion_form.per_page
            ),
        )
        .finish())
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct UserModification {
    #[validate(range(min = 0))]
    id: u32,
    #[validate(length(min = 2))]
    login: String,
    #[validate(length(min = 2))]
    name: String,
    is_authorized: Option<String>,
    #[validate(range(min = 0))]
    page: i32,
    #[validate(range(min = 0))]
    per_page: i32,
}

#[post("/user/modification")]
pub async fn user_modification(
    user_modification_form: actix_web_validator::Form<UserModification>,
) -> Result<impl Responder, ApplicationError> {
    let mut user: User = user::Entity::find_by_id(user_modification_form.id)
        .await?
        .ok_or(ApplicationError::NotFound)?;
    user.name = user_modification_form.name.clone();
    user.is_authorized = user_modification_form.is_authorized.is_some();
    user::Entity::update(user).await?;
    JwtUser::revoke_all_session(&user_modification_form.login)?;
    Ok(HttpResponse::Found()
        .header(
            "Location",
            format!(
                "/admin?info=User {} has been modified&page={}&per_page={}&id={}",
                user_modification_form.login,
                user_modification_form.page,
                user_modification_form.per_page,
                user_modification_form.id
            ),
        )
        .finish())
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct UserChangeLeague {
    league_id: u32,
    user_id: u32,
    code: Option<String>,
    action: String,
    name: String,
}

#[post("/profile/leagues")]
pub async fn user_change_leagues(
    req: HttpRequest,
    user_change_league_form: actix_web_validator::Form<UserChangeLeague>,
) -> Result<impl Responder, ApplicationError> {
    let res_msg: String = match user_change_league_form.action.as_str() {
        "add" => {
            user::Entity::add_leagues_as_favorite(
                user_change_league_form.user_id,
                user_change_league_form.league_id,
            )
            .await?;
            format!(
                "{} has been added as favorite",
                user_change_league_form.name
            )
        }
        "remove" => {
            user::Entity::remove_leagues_as_favorite(
                user_change_league_form.user_id,
                user_change_league_form.league_id,
            )
            .await?;
            format!(
                "{} has been removed from the favorite list",
                user_change_league_form.name
            )
        }
        _ => return Err(ApplicationError::BadRequest),
    };

    let jwt_path: String = std::env::var("JWT_TOKEN_PATH")?;
    let current_token: Cookie = req
        .cookie(jwt_path.as_str())
        .ok_or(ApplicationError::IllegalToken)?;
    let mut refreshed_token: Cookie = Cookie::new(
        jwt_path.as_str(),
        JwtUser::refresh_token(current_token.value()).await?,
    );
    refreshed_token.set_path("/");
    let code_redirect: String = match &user_change_league_form.code {
        Some(v) => format!("&code={}", v),
        None => String::new(),
    };
    Ok(HttpResponse::Found()
        .del_cookie(&current_token)
        .cookie(refreshed_token)
        .header(
            "Location",
            format!("/profile/leagues?info={}{}", res_msg, code_redirect),
        )
        .finish())
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct UserSearch {
    #[validate(length(min = 1))]
    login: String,
    #[validate(range(min = 0))]
    page: i32,
    #[validate(range(min = 0))]
    per_page: i32,
}

#[post("/user/search")]
pub async fn user_search(
    user_search_form: actix_web_validator::Form<UserSearch>,
) -> Result<impl Responder, ApplicationError> {
    let user: Option<User> = user::Entity::get_user_by_login(&user_search_form.login).await?;
    let result: String = match user {
        Some(v) => format!(
            "/admin?page={}&per_page={}&id={}",
            user_search_form.page, user_search_form.per_page, v.id
        ),
        None => {
            format!(
                "/admin?error=User {} hasn't been found&page={}&per_page={}",
                user_search_form.login, user_search_form.page, user_search_form.per_page,
            )
        }
    };
    Ok(HttpResponse::Found().header("Location", result).finish())
}
