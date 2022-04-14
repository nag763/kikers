use crate::auth::JwtUser;
use crate::database::Database;
use ffb_structs::entities::user;
use ffb_structs::entities::user::Model as User;
use ffb_structs::entities::user_league;
use ffb_structs::entities::user_league::Model as UserLeague;
use crate::error::ApplicationError;
use actix_web::http::Cookie;
use actix_web::HttpMessage;
use actix_web::{post, HttpRequest, HttpResponse, Responder};
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::ModelTrait;
use sea_orm::QueryFilter;
use sea_orm::Set;

#[derive(serde::Deserialize, validator::Validate)]
pub struct UserActivation {
    #[validate(range(min = 0))]
    id: i32,
    #[validate(range(min = 0, max = 1))]
    value: i8,
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
    let conn = Database::acquire_sql_connection().await?;
    let user_to_update: User = user::Entity::find_by_id(user_activation_form.id)
        .filter(Condition::all().add(user::Column::Role.lt(jwt_user.role)))
        .one(&conn)
        .await?
        .ok_or(ApplicationError::NotFound)?;

    let mut user_to_update: user::ActiveModel = user_to_update.into();
    user_to_update.is_authorized = Set(user_activation_form.value);
    user_to_update.update(&conn).await?;

    if user_activation_form.value == 0 {
        let mut redis_conn = Database::acquire_redis_connection()?;
        redis::cmd("DEL")
            .arg(format!("token:{}", user_activation_form.login.as_str()))
            .query(&mut redis_conn)?;
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
    id: i32,
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
    let conn = Database::acquire_sql_connection().await?;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let user_to_delete: User = user::Entity::find_by_id(user_deletion_form.id)
        .filter(Condition::all().add(user::Column::Role.lt(jwt_user.role)))
        .one(&conn)
        .await?
        .ok_or(ApplicationError::NotFound)?;
    user_to_delete.delete(&conn).await?;
    let mut redis_conn = Database::acquire_redis_connection()?;
    redis::cmd("DEL")
        .arg(format!("token:{}", user_deletion_form.login.as_str()))
        .query(&mut redis_conn)?;
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
    id: i32,
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
    req: HttpRequest,
    user_modification_form: actix_web_validator::Form<UserModification>,
) -> Result<impl Responder, ApplicationError> {
    let conn = Database::acquire_sql_connection().await?;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let user: User = user::Entity::find_by_id(user_modification_form.id)
        .filter(Condition::all().add(user::Column::Role.lt(jwt_user.role)))
        .one(&conn)
        .await?
        .ok_or(ApplicationError::NotFound)?;
    let mut user: user::ActiveModel = user.into();
    user.name = Set(user_modification_form.name.clone());
    user.is_authorized = Set(user_modification_form.is_authorized.is_some() as i8);
    user.update(&conn).await?;
    let mut redis_conn = Database::acquire_redis_connection()?;
    redis::cmd("DEL")
        .arg(format!("token:{}", user_modification_form.login.as_str()))
        .query(&mut redis_conn)?;
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
    #[validate(range(min = 0))]
    league_id: i32,
    #[validate(range(min = 0))]
    user_id: i32,
    code: Option<String>,
    action: String,
    name: String,
}

#[post("/profile/leagues")]
pub async fn user_change_leagues(
    req: HttpRequest,
    user_change_league_form: actix_web_validator::Form<UserChangeLeague>,
) -> Result<impl Responder, ApplicationError> {
    let conn = Database::acquire_sql_connection().await?;
    let res_msg: String = match user_change_league_form.action.as_str() {
        "add" => {
            let user_league = user_league::ActiveModel {
                league_id: Set(user_change_league_form.league_id),
                user_id: Set(user_change_league_form.user_id),
                ..Default::default()
            };
            user_league.insert(&conn).await?;
            format!(
                "{} has been added as favorite",
                user_change_league_form.name
            )
        }
        "remove" => {
            let user_league_to_delete: UserLeague = user_league::Entity::find()
                .filter(
                    Condition::all()
                        .add(user_league::Column::UserId.eq(user_change_league_form.user_id))
                        .add(user_league::Column::LeagueId.eq(user_change_league_form.league_id)),
                )
                .one(&conn)
                .await?
                .ok_or(ApplicationError::NotFound)?;
            user_league_to_delete.delete(&conn).await?;
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
    req: HttpRequest,
    user_search_form: actix_web_validator::Form<UserSearch>,
) -> Result<impl Responder, ApplicationError> {
    let conn = Database::acquire_sql_connection().await?;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let user: Option<User> = user::Entity::find()
        .filter(
            Condition::all()
                .add(user::Column::Role.lt(jwt_user.role))
                .add(user::Column::Login.eq(user_search_form.login.clone())),
        )
        .one(&conn)
        .await?;
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
