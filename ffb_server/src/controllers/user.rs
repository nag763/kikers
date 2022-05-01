use ffb_auth::JwtUser;

use crate::error::ApplicationError;
use actix_web::{post, HttpRequest, HttpResponse, Responder};
use ffb_structs::user;
use ffb_structs::user::Model as User;

#[derive(serde::Deserialize, validator::Validate)]
pub struct UserActivation {
    uuid: String,
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
    let result: bool = user::Entity::change_activation_status_with_role_check(
        &user_activation_form.uuid,
        user_activation_form.value,
        jwt_user.role,
    )
    .await?
    .into();

    let result: String = match result {
        true => {
            if !user_activation_form.value {
                JwtUser::revoke_all_session(&user_activation_form.login)?;
            }
            format!(
                "info=User {}'s access has been modified",
                &user_activation_form.login
            )
        }
        false => format!(
            "error=An error happened while modifying user {}'s acces",
            &user_activation_form.login
        ),
    };
    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!(
                "/admin?{}&page={}&per_page={}",
                result, user_activation_form.page, user_activation_form.per_page
            ),
        ))
        .finish())
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct UserDeletion {
    uuid: String,
    #[validate(length(min = 2))]
    login: String,
    #[validate(range(min = 0))]
    page: i32,
    #[validate(range(min = 0))]
    per_page: i32,
}

#[post("/user/deletion")]
pub async fn user_deletion(
    req: HttpRequest,
    user_deletion_form: actix_web_validator::Form<UserDeletion>,
) -> Result<impl Responder, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let result: bool =
        user::Entity::delete_user_uuid_with_role_check(&user_deletion_form.uuid, jwt_user.role)
            .await?
            .into();

    let result: String = match result {
        true => {
            JwtUser::revoke_all_session(&user_deletion_form.login)?;
            format!("info=User {} has been deleted", &user_deletion_form.login)
        }
        false => format!(
            "error=User {} hasn't been deleted",
            &user_deletion_form.login
        ),
    };
    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!(
                "/admin?{}&page={}&per_page={}",
                &result, user_deletion_form.page, user_deletion_form.per_page
            ),
        ))
        .finish())
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct UserModification {
    id: u32,
    uuid: String,
    #[validate(length(min = 2))]
    login: String,
    #[validate(length(min = 2))]
    name: String,
    is_authorized: Option<String>,
    #[validate(range(min = 0))]
    page: i32,
    #[validate(range(min = 0))]
    per_page: i32,
    role: u32,
}

#[post("/user/modification")]
pub async fn user_modification(
    user_modification_form: actix_web_validator::Form<UserModification>,
    req: HttpRequest,
) -> Result<impl Responder, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let mut user: User = user::Entity::find_by_uuid(&user_modification_form.uuid)
        .await?
        .ok_or(ApplicationError::NotFound)?;
    user.name = user_modification_form.name.clone();
    user.role_id = user_modification_form.role;
    user.is_authorized = user_modification_form.is_authorized.is_some();
    let result: bool = user::Entity::update_with_role_check(user, jwt_user.role)
        .await?
        .into();
    let result: String = match result {
        true => {
            JwtUser::revoke_all_session(&user_modification_form.login)?;
            format!(
                "info=User {} has been modified",
                &user_modification_form.login
            )
        }
        false => format!(
            "error=User {} hasn't been modified, an error happened",
            &user_modification_form.login
        ),
    };
    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!(
                "/admin?{}&page={}&per_page={}&id={}",
                &result,
                user_modification_form.page,
                user_modification_form.per_page,
                user_modification_form.id
            ),
        ))
        .finish())
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct UserSelfModification {
    uuid: String,
    #[validate(length(min = 2))]
    login: String,
    #[validate(length(min = 2))]
    name: String,
    password: Option<String>,
}

#[post("/profile/edit")]
pub async fn user_self_modification(
    user_modification_form: actix_web_validator::Form<UserSelfModification>,
    req: HttpRequest,
) -> Result<impl Responder, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let mut user: User = user::Entity::find_by_uuid(&user_modification_form.uuid)
        .await?
        .ok_or(ApplicationError::NotFound)?;
    if user.id != jwt_user.id {
        return Err(ApplicationError::BadRequest);
    }
    user.name = user_modification_form.name.clone();
    if let Some(password) = &user_modification_form.password {
        user.password = JwtUser::encrypt_key(&password)?
    }
    let result: bool = user::Entity::update_self(user).await?.into();
    if result {
            JwtUser::revoke_all_session(&user_modification_form.login)?;
    }
    Ok(HttpResponse::Found()
        .append_header(("Location", "/logout"))
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
    user_change_league_form: actix_web_validator::Form<UserChangeLeague>,
    req: HttpRequest,
) -> Result<impl Responder, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    if jwt_user.id != user_change_league_form.user_id {
        return Err(ApplicationError::BadRequest);
    }
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

    let code_redirect: String = match &user_change_league_form.code {
        Some(v) => format!("&code={}", v),
        None => String::new(),
    };
    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!("/profile/leagues?info={}{}", res_msg, code_redirect),
        ))
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
    req: HttpRequest,
) -> Result<impl Responder, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let user: Option<User> =
        user::Entity::get_user_by_login_with_role_check(&user_search_form.login, jwt_user.role)
            .await?;
    let result: String = match user {
        Some(v) => format!(
            "/admin?page={}&per_page={}&id={}",
            user_search_form.page, user_search_form.per_page, v.id
        ),
        None => {
            format!(
                "/admin?error=User {} has either not been found or you don't have rights on him&page={}&per_page={}",
                user_search_form.login, user_search_form.page, user_search_form.per_page,
            )
        }
    };
    Ok(HttpResponse::Found()
        .append_header(("Location", result))
        .finish())
}
