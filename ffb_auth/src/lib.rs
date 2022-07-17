use crate::error::ApplicationError;
use actix_web::HttpRequest;
use ffb_structs::{token, user, user::Model as User};
use hmac::{Hmac, Mac};
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use magic_crypt::{MagicCrypt256, MagicCryptTrait};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use time::{Duration, OffsetDateTime};

/// Module to handles the common errors that can
/// be thrown by this crate.
pub mod error;

#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate magic_crypt;
#[macro_use]
extern crate log;

/**
 * A jwt user is a user from the base whose
 * informations have been stored within a secure
 * token.
 */
#[derive(Debug, Default, Deserialize, Serialize, Clone, Display)]
#[display(fmt = "{login} ({name}) with id {id} and role {role}\n
        Emitted on {emited_on}, tbr on {refresh_after}, expires on {expiracy_date}")]
pub struct JwtUser {
    /// Id in base of the user
    pub id: u32,
    /// UUID used for more delicate operations
    pub uuid: String,
    /// His login
    pub login: String,
    /// The username, up to his choice
    pub name: String,
    /// Whether the user is authorized or not
    pub is_authorized: bool,
    /// The user role, admin, simple user, or manager
    pub role: u32,
    /// His locale, used to display the translated application
    pub locale_id: u32,
    /// When the jwt_token has been emited
    pub emited_on: i64,
    /// When the jwt token will have to be refreshed
    pub refresh_after: i64,
    /// The expiracy date of the jwt, it shouldn't be usable following this
    /// date
    pub expiracy_date: i64,
}

impl JwtUser {
    /**
     * Encrypts a key with the secret
     *
     * * Arguments :
     *
     * - key : The key to encrypt
     */
    pub fn encrypt_key(key: &str) -> Result<String, ApplicationError> {
        let mc: MagicCrypt256 = new_magic_crypt!(std::env::var("ENCRYPT_KEY")?, 256);
        let encrypted_key: String = mc.encrypt_str_to_base64(key);
        Ok(encrypted_key)
    }

    /**
     * Generates a token
     *
     * * Arguments :
     * - user : The base user that we want to tokenize
     */
    async fn gen_token(user: User) -> Result<String, ApplicationError> {
        let jwt_key: String = std::env::var("JWT_KEY")?;
        let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes())?;
        let header: Header = Default::default();
        let emited_on: OffsetDateTime = OffsetDateTime::now_utc();
        // The token has to be refreshed every 15 minutes
        let refresh_after: OffsetDateTime = emited_on + Duration::minutes(15);
        // The token expires after one week
        let expiracy_date: OffsetDateTime = emited_on + Duration::weeks(1);
        let unsigned_token = Token::new(
            header,
            JwtUser {
                id: user.id,
                uuid: user.uuid,
                login: user.login.clone(),
                name: user.name,
                is_authorized: user.is_authorized,
                role: user.role_id,
                locale_id: user.locale_id,
                emited_on: emited_on.unix_timestamp(),
                refresh_after: refresh_after.unix_timestamp(),
                expiracy_date: expiracy_date.unix_timestamp(),
            },
        );
        info!("Token for {} has been generated", user.login);
        let signed_token = unsigned_token.sign_with_key(&key)?;
        Ok(signed_token.into())
    }

    /**
     * Method to know if the jwt has to be refreshed.
     */
    pub fn has_to_be_refreshed(&self) -> bool {
        let now: i64 = OffsetDateTime::now_utc().unix_timestamp();
        self.refresh_after < now
    }

    /**
     * Whether the session is still valid or not.
     */
    pub fn has_session_expired(&self) -> bool {
        let now: i64 = OffsetDateTime::now_utc().unix_timestamp();
        self.expiracy_date < now
    }

    /**
     * Emit a new token.
     *
     * The pair login, password is verified beforehand to check
     * that the user is indeed an existing user besides of being authorized.
     *
     * None is returned when the user doesn't exist in the database. An error is thrown when the
     * user isn't authorized and thus a token can't be emitted.
     *
     * * Arguments :
     *
     * - login : User's login
     * - Password : Associed password in base. Be aware that the raw password should be passed to
     * this method and not the hashed one
     */
    pub async fn emit(login: &str, password: &str) -> Result<Option<String>, ApplicationError> {
        let encrypted_password: String = Self::encrypt_key(password)?;
        let user: Option<User> =
            user::Entity::get_user_by_credentials(login, &encrypted_password).await?;

        match user {
            // If the user exists, we check whether he is authorized or not
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
            // If he doesn't exist, None is returned
            None => Ok(None),
        }
    }

    /**
     * Check whether the token is valid for the given login.
     *
     * # Arguments
     *
     * - token : The token to verify
     * - login : The login to verify
     */
    pub fn check_token_of_login(token: &str, login: &str) -> Result<(), ApplicationError> {
        if !token::Entity::verify(login, token)? {
            warn!("Token for {} has been considered as invalid", &login);
            Err(ApplicationError::IllegalToken)
        } else {
            debug!("Token for {} has been checked", &login);
            Ok(())
        }
    }

    /**
     * Returns the JWT User structure from an existing JWT.
     *
     * # Arguments :
     *
     * - token : The token to get the user from.
     */
    pub fn from_token(token: &str) -> Result<JwtUser, ApplicationError> {
        let jwt_key: String = std::env::var("JWT_KEY")?;
        let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_key.as_bytes())?;
        let token: Token<Header, JwtUser, _> = VerifyWithKey::verify_with_key(token, &key)?;
        let (_, jwt_user) = token.into();
        Ok(jwt_user)
    }

    /**
     * Refresh an existing token and invalidate the previous one.
     *
     * # Argument :
     *
     * - token : The token to refresh
     */
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

    /**
     * Revoke completely one session (or token).
     *
     * # Arguments :
     *
     * - login : the login associed with the token.
     * - token : the token to revoke.
     */
    pub fn revoke_session(login: &str, token: &str) -> Result<(), ApplicationError> {
        token::Entity::revoke_token(login, token)?;
        info!("Token for {} has been revoked", login);
        Ok(())
    }

    /**
     * Revoke completely every user session
     *
     * # Arguments :
     *
     * - login : the login associed with the token.
     */
    pub fn revoke_all_session(login: &str) -> Result<(), ApplicationError> {
        token::Entity::revoke_all(login)?;
        info!("Sessions of {} have been discarded", login);
        Ok(())
    }

    /**
     * Gets the jwt user directly from the HTTP request
     *
     * # Arguments :
     *
     * - req : The HTTP request
     */
    pub fn from_request(req: HttpRequest) -> Result<JwtUser, ApplicationError> {
        match req.cookie(std::env::var("JWT_TOKEN_PATH")?.as_str()) {
            Some(token) => Ok(JwtUser::from_token(token.value())?),
            None => Err(ApplicationError::InternalError),
        }
    }
}
