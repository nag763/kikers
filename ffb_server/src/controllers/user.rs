use crate::auth::JwtUser;
use crate::database::Database;
use crate::error::ApplicationError;
use actix_web::{post, HttpRequest, HttpResponse, Responder};
use ffb_structs::sql_entities::{user, user::Model as User};
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
