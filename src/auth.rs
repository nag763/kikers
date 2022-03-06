use super::entities::navaccess::Model as NavAccess;
use super::entities::user::Model as User;
use crate::entities::navaccess;
use crate::entities::role_navaccess;
use crate::entities::user;
use crate::error::ApplicationError;
use crate::database::Database;
use actix_web::{HttpMessage, HttpRequest};
use hmac::{Hmac, Mac};
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct JwtUser {
    pub id: i32,
    pub login: String,
    pub name: String,
    pub nav: Vec<NavAccess>,
    pub is_authorized: i8,
    pub role: i32,
    pub to_refresh_on: i64,
}

impl JwtUser {
    pub async fn emit(login: &str, password: &str) -> Result<Option<String>, ApplicationError> {
        let conn = Database::acquire_connection().await?;
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
                let nav: Vec<NavAccess> = navaccess::Entity::find()
                    .join(
                        JoinType::InnerJoin,
                        navaccess::Relation::RoleNavaccess.def(),
                    )
                    .filter(Condition::all().add(role_navaccess::Column::RoleId.eq(user.role)))
                    .order_by_asc(navaccess::Column::Position)
                    .all(&conn)
                    .await?;
                let to_refresh_on: i64 =
                    (time::OffsetDateTime::now_utc() + time::Duration::minutes(1)).unix_timestamp();

                let jwt_key: String = std::env::var("JWT_KEY")?;
                let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes())?;

                let header: Header = Default::default();
                let unsigned_token = Token::new(
                    header,
                    JwtUser {
                        id: user.id,
                        login: user.login.clone(),
                        name: user.name,
                        nav,
                        is_authorized: user.is_authorized,
                        role: user.role,
                        to_refresh_on,
                    },
                );
                let signed_token = unsigned_token.sign_with_key(&key)?;
                info!("Token for {} has been emitted", login);
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
                    warn!("User {} tried to connect but isn't authorized yet", login);
                    Err(ApplicationError::UserNotAuthorized(login.to_string()))
                } else {
                    warn!("User {} tried to connect but either his credentials are incorrect or he doesn't exist", login);
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

    pub fn from_request(req: HttpRequest) -> Result<JwtUser, ApplicationError> {
        match req.cookie(std::env::var("JWT_TOKEN_PATH")?.as_str()) {
            Some(token) => Ok(JwtUser::check_token(token.value())?),
            None => Err(ApplicationError::InternalError),
        }
    }
}
