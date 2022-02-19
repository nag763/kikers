use super::entities::user::Model as User;
use crate::entities::user;
use hmac::Hmac;
use hmac::Mac;
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct JwtUser {
    id: i32,
    pub login: String,
    pub name: String,
    pub is_admin: i8,
}

impl JwtUser {
    pub async fn emit(login: &str, password: &str) -> Option<String> {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
        let conn = sea_orm::Database::connect(&db_url).await.unwrap();
        let user: User = user::Entity::find()
            .filter(
                Condition::all()
                    .add(user::Column::Login.eq(login))
                    .add(user::Column::Password.eq(password))
                    .add(user::Column::IsAuthorized.eq(1)),
            )
            .one(&conn)
            .await
            .expect("Error during db connect")?;

        let jwt_key: String = std::env::var("JWT_KEY").expect("No jwt key configured");
        let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes()).unwrap();

        let header: Header = Default::default();
        let unsigned_token = Token::new(
            header,
            JwtUser {
                id: user.id,
                login: user.login,
                name: user.name,
                is_admin: user.is_admin,
            },
        );
        let signed_token = unsigned_token.sign_with_key(&key).unwrap();

        Some(signed_token.into())
    }

    pub fn check_token(token: &str) -> Result<JwtUser, Box<dyn std::error::Error>> {
        let jwt_key: String = std::env::var("JWT_KEY").expect("No jwt key configured");
        let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes()).unwrap();
        let token: Token<Header, JwtUser, _> = VerifyWithKey::verify_with_key(token, &key)?;
        let (_, jwt_user) = token.into();
        Ok(jwt_user)
    }
}
