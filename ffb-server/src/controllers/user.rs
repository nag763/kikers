use crate::auth::JwtUser;
use crate::database::Database;
use crate::entities::user;
use crate::entities::user::Model as User;
use crate::error::ApplicationError;
use actix_web::web::Form;
use actix_web::{post, HttpRequest, HttpResponse, Responder};
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::ModelTrait;
use sea_orm::QueryFilter;
use sea_orm::Set;

#[derive(serde::Deserialize)]
pub struct UserActivation {
    id: i32,
    value: i8,
    page: i32,
    per_page: i32,
    login: String,
}

#[post("/user/activation")]
pub async fn user_activation(
    req: HttpRequest,
    user_activation_form: Form<UserActivation>,
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

#[derive(serde::Deserialize)]
pub struct UserDeletion {
    id: i32,
    login: String,
    page: i32,
    per_page: i32,
}

#[post("/user/deletion")]
pub async fn user_deletion(
    req: HttpRequest,
    user_deletion_form: Form<UserDeletion>,
) -> Result<impl Responder, ApplicationError> {
    let conn = Database::acquire_sql_connection().await?;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let user_to_delete: User = user::Entity::find_by_id(user_deletion_form.id)
        .filter(Condition::all().add(user::Column::Role.lt(jwt_user.role)))
        .one(&conn)
        .await?
        .ok_or(ApplicationError::NotFound)?;
    user_to_delete.delete(&conn).await?;
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

#[derive(serde::Deserialize)]
pub struct UserModification {
    id: i32,
    login: String,
    name: String,
    is_authorized: Option<String>,
    page: i32,
    per_page: i32,
}

#[post("/user/modification")]
pub async fn user_modification(
    req: HttpRequest,
    user_modification_form: Form<UserModification>,
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

#[derive(serde::Deserialize)]
pub struct UserSearch {
    login: String,
    page: i32,
    per_page: i32,
}

#[post("/user/search")]
pub async fn user_search(
    req: HttpRequest,
    user_search_form: Form<UserSearch>,
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
