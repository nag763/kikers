use super::entities::user::Model as User;
use crate::entities::sea_orm_active_enums::Role;
use crate::entities::user;
use crate::error::ApplicationError;
use actix_web::dev::ServiceRequest;
use actix_web::HttpMessage;
use hmac::Hmac;
use hmac::Mac;
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use sea_orm::ActiveEnum;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct JwtUser {
    pub id: i32,
    pub login: String,
    pub name: String,
    pub is_authorized: i8,
    pub role: String,
}

impl JwtUser {
    pub async fn emit(login: &str, password: &str) -> Result<Option<String>, ApplicationError> {
        let db_url = std::env::var("DATABASE_URL")?;
        let conn = sea_orm::Database::connect(&db_url).await?;
        let user_unwrapped: Option<User> = user::Entity::find()
            .filter(
                Condition::all()
                    .add(user::Column::Login.eq(login))
                    .add(user::Column::Password.eq(password))
                    .add(user::Column::IsAuthorized.eq(1)),
            )
            .one(&conn)
            .await?;

        match user_unwrapped {
            Some(user) => {
                let jwt_key: String = std::env::var("JWT_KEY")?;
                let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes())?;

                let header: Header = Default::default();
                let unsigned_token = Token::new(
                    header,
                    JwtUser {
                        id: user.id,
                        login: user.login,
                        name: user.name,
                        is_authorized: user.is_authorized,
                        role: user.role.to_value(),
                    },
                );
                let signed_token = unsigned_token.sign_with_key(&key)?;
                Ok(Some(signed_token.into()))
            }
            None => {
                let user_unwrapped: Option<User> = user::Entity::find()
                    .filter(
                        Condition::all()
                            .add(user::Column::Login.eq(login))
                            .add(user::Column::Password.eq(password))
                            .add(user::Column::IsAuthorized.eq(0)),
                    )
                    .one(&conn)
                    .await?;
                if user_unwrapped.is_some() {
                    Err(ApplicationError::UserNotAuthorized(login.to_string()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn check_token(token: &str) -> Result<JwtUser, ApplicationError> {
        let jwt_key: String = std::env::var("JWT_KEY")?;
        let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes())?;
        let token: Token<Header, JwtUser, _> = VerifyWithKey::verify_with_key(token, &key)?;
        let (_, jwt_user) = token.into();
        Ok(jwt_user)
    }
}

pub async fn extract(req: &mut ServiceRequest) -> Result<Vec<Role>, actix_web::Error> {
    match req.cookie(super::constants::JWT_TOKEN_PATH) {
        Some(token) => {
            let user: JwtUser = JwtUser::check_token(token.value())?;
            let role: Role = Role::try_from_value(&user.role).unwrap();
            Ok(vec![role])
        }
        None => {
            error!("no jwt");
            Ok(vec![])
        }
    }
}
