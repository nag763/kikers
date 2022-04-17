use crate::error::ApplicationError;
use crate::magic_crypt::MagicCryptTrait;
use actix_web::{HttpMessage, HttpRequest};
use ffb_structs::navaccess;
use ffb_structs::navaccess::Model as NavAccess;
use ffb_structs::user;
use ffb_structs::user::Model as User;
use hmac::{Hmac, Mac};
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub mod error;

#[macro_use]
extern crate magic_crypt;
#[macro_use]
extern crate log;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct JwtUser {
    pub id: u32,
    pub login: String,
    pub name: String,
    pub nav: Vec<NavAccess>,
    pub fav_leagues: Vec<u32>,
    pub is_authorized: bool,
    pub role: u32,
}

impl JwtUser {
    pub fn encrypt_key(key: &str) -> Result<String, ApplicationError> {
        let mc = new_magic_crypt!(std::env::var("ENCRYPT_KEY")?, 256);
        Ok(mc.encrypt_str_to_base64(key))
    }

    async fn gen_token(user: User) -> Result<String, ApplicationError> {
        let nav: Vec<NavAccess> =
            navaccess::Entity::get_navaccess_for_role_id(user.role_id).await?;
        let fav_leagues: Vec<u32> = user::Entity::get_favorite_leagues_id(user.id).await?;

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
                role: user.role_id,
                fav_leagues,
            },
        );
        info!("Token for {} has been emitted", user.login);
        let signed_token = unsigned_token.sign_with_key(&key)?;
        Ok(signed_token.into())
    }

    pub async fn emit(login: &str, password: &str) -> Result<Option<String>, ApplicationError> {
        let encrypted_password: String = Self::encrypt_key(password)?;
        let user: Option<User> = user::Entity::get_user_by_credentials(
            login.to_string(),
            encrypted_password.to_string(),
        )
        .await?;

        match user {
            Some(user) => match user.is_authorized {
                true => Ok(Some(Self::gen_token(user).await?)),
                false => Err(ApplicationError::UserNotAuthorized(login.to_string())),
            },
            None => Ok(None),
        }
    }

    pub fn check_token(token: &str) -> Result<JwtUser, ApplicationError> {
        let jwt_key: String = std::env::var("JWT_KEY")?;
        let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes())?;
        let token: Token<Header, JwtUser, _> = VerifyWithKey::verify_with_key(token, &key)?;
        let (_, jwt_user) = token.into();
        Ok(jwt_user)
    }

    pub async fn refresh_token(token: &str) -> Result<String, ApplicationError> {
        let jwt_user = JwtUser::check_token(token)?;
        let user: User = user::Entity::find_by_id(jwt_user.id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        let new_token = Self::gen_token(user).await?;
        Ok(new_token)
    }

    pub fn from_request(req: HttpRequest) -> Result<JwtUser, ApplicationError> {
        match req.cookie(std::env::var("JWT_TOKEN_PATH")?.as_str()) {
            Some(token) => Ok(JwtUser::check_token(token.value())?),
            None => Err(ApplicationError::InternalError),
        }
    }
}
