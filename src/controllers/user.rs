use crate::auth::JwtUser;
use crate::entities::user;
use crate::entities::user::Model as User;
use crate::error::ApplicationError;
use actix_web::web::Form;
use actix_web::{post, HttpRequest, HttpResponse, Responder};
use sea_orm::ActiveModelTrait;
use sea_orm::EntityTrait;
use sea_orm::Set;

#[derive(serde::Deserialize)]
pub struct UserActivation {
    id: i32,
    value: i8,
    offset: i32,
    login: String,
}

#[post("/user/activation")]
pub async fn user_activation(
    req: HttpRequest,
    user_activation_form: Form<UserActivation>,
) -> Result<impl Responder, ApplicationError> {
    let db_url = std::env::var("DATABASE_URL")?;
    let conn = sea_orm::Database::connect(&db_url).await?;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let user_to_update: User = match user::Entity::find_by_id(user_activation_form.id)
        .one(&conn)
        .await?
    {
        Some(user_to_update) => user_to_update,
        None => return Err(ApplicationError::NotFound),
    };

    if jwt_user.role < user_to_update.role {
        return Err(ApplicationError::BadRequest);
    }

    let mut user_to_update: user::ActiveModel = user_to_update.into();
    user_to_update.is_authorized = Set(user_activation_form.value);
    user_to_update.update(&conn).await?;

    Ok(HttpResponse::Found()
        .header(
            "Location",
            format!(
                "/admin?info=User {}'s access has been modified&offset={}",
                user_activation_form.login, user_activation_form.offset
            ),
        )
        .finish())
}
