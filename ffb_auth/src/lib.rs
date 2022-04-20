use crate::error::ApplicationError;
use crate::magic_crypt::MagicCryptTrait;
use actix_web::HttpRequest;
use ffb_structs::navaccess;
use ffb_structs::navaccess::Model as NavAccess;
use ffb_structs::token;
use ffb_structs::user;
use ffb_structs::user::Model as User;
use hmac::{Hmac, Mac};
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub mod error;

#[macro_use]
extern crate derive_more;
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
    pub is_authorized: bool,
    pub role: u32,
    pub emited_on: i64,
}

impl JwtUser {
    pub fn encrypt_key(key: &str) -> Result<String, ApplicationError> {
        let mc = new_magic_crypt!(std::env::var("ENCRYPT_KEY")?, 256);
        Ok(mc.encrypt_str_to_base64(key))
    }

    async fn gen_token(user: User) -> Result<String, ApplicationError> {
        let nav: Vec<NavAccess> =
            navaccess::Entity::get_navaccess_for_role_id(user.role_id).await?;

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
                emited_on: time::OffsetDateTime::now_utc().unix_timestamp(),
            },
        );
        info!("Token for {} has been generated", user.login);
        let signed_token = unsigned_token.sign_with_key(&key)?;
        Ok(signed_token.into())
    }

    pub async fn emit(login: &str, password: &str) -> Result<Option<String>, ApplicationError> {
        let encrypted_password: String = Self::encrypt_key(password)?;
        let user: Option<User> =
            user::Entity::get_user_by_credentials(login, &encrypted_password).await?;

        match user {
            Some(user) => match user.is_authorized {
                true => {
                    let token = Self::gen_token(user).await?;
                    token::Entity::register(login, &token)?;
                    info!("Token for {} has been registered and emitted", &login);
                    Ok(Some(token))
                }
                false => {
                    warn!(
                        "Token for {} was ready but user isn't authorized",
                        &user.login
                    );
                    Err(ApplicationError::UserNotAuthorized(login.to_string()))
                }
            },
            None => Ok(None),
        }
    }

    pub fn check_token(token: &str) -> Result<(), ApplicationError> {
        let jwt_user: Self = Self::from_token(token)?;
        if !token::Entity::verify(&jwt_user.login, token)? {
            warn!(
                "Token for {} has been considered as invalid",
                &jwt_user.login
            );
            Err(ApplicationError::IllegalToken)
        } else {
            debug!("Token for {} has been checked", &jwt_user.login);
            Ok(())
        }
    }

    pub fn from_token(token: &str) -> Result<JwtUser, ApplicationError> {
        let jwt_key: String = std::env::var("JWT_KEY")?;
        let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes())?;
        let token: Token<Header, JwtUser, _> = VerifyWithKey::verify_with_key(token, &key)?;
        let (_, jwt_user) = token.into();
        Ok(jwt_user)
    }

    pub async fn refresh_token(token: &str) -> Result<String, ApplicationError> {
        let jwt_user = JwtUser::from_token(token)?;
        let user: User = user::Entity::find_by_id(jwt_user.id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        let new_token = Self::gen_token(user).await?;
        token::Entity::revoke_token(&jwt_user.login, &token)?;
        debug!("Token for {} has been refreshed", &jwt_user.login);
        token::Entity::register(&jwt_user.login, &new_token)?;
        debug!("Token for {} has been registered", &jwt_user.login);
        info!("Token for {} has been refreshed", &jwt_user.login);
        Ok(new_token)
    }

    pub fn revoke_session(login: &str, token: &str) -> Result<(), ApplicationError> {
        token::Entity::revoke_token(login, token)?;
        info!("Token for {} has been revoked", login);
        Ok(())
    }

    pub fn revoke_all_session(login: &str) -> Result<(), ApplicationError> {
        token::Entity::revoke_all(login)?;
        info!("Sessions of {} have been discarded", login);
        Ok(())
    }

    pub fn from_request(req: HttpRequest) -> Result<JwtUser, ApplicationError> {
        match req.cookie(std::env::var("JWT_TOKEN_PATH")?.as_str()) {
            Some(token) => Ok(JwtUser::from_token(token.value())?),
            None => Err(ApplicationError::InternalError),
        }
    }
}
