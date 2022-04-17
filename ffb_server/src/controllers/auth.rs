use crate::auth::JwtUser;

use crate::error::ApplicationError;
use actix_web::http::Cookie;
use actix_web::{get, post, HttpMessage, HttpRequest, HttpResponse, Responder};
use ffb_structs::user;
use ffb_structs::user::Model as User;
use magic_crypt::MagicCryptTrait;
use std::thread;

lazy_static! {
    static ref RE_VALID_LOGIN: regex::Regex =
        regex::Regex::new(r####"^(?:[0-9a-zA-Z\-_]{3,32})$"####).unwrap();
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct LoginForm {
    #[validate(regex = "RE_VALID_LOGIN")]
    login: String,
    #[validate(length(min = 4, max = 128))]
    password: String,
}

#[post("/login")]
pub async fn login(
    req: HttpRequest,
    login_form: actix_web_validator::Form<LoginForm>,
) -> Result<HttpResponse, ApplicationError> {
    let mc = new_magic_crypt!(std::env::var("ENCRYPT_KEY")?, 256);
    let encrypted_password: String = mc.encrypt_str_to_base64(login_form.password.as_str());
    match JwtUser::emit(login_form.login.as_str(), encrypted_password.as_str()).await? {
        Some(token) => Ok(HttpResponse::Found()
            .header("Location", "/")
            .cookie(Cookie::new(
                std::env::var("JWT_TOKEN_PATH")?.as_str(),
                token,
            ))
            .finish()),
        None => {
            warn!(
                "{:?} tried to connect with login {} without success",
                req.peer_addr(),
                login_form.login
            );
            thread::sleep(std::time::Duration::from_secs(3));
            Ok(HttpResponse::Found()
                .header(
                    "Location",
                    "/?error=We couldn't connect you, please verify your credentials",
                )
                .finish())
        }
    }
}

#[get("/logout")]
pub async fn logout(req: HttpRequest) -> Result<impl Responder, ApplicationError> {
    if let Some(jwt_cookie) = req.cookie(std::env::var("JWT_TOKEN_PATH")?.as_str()) {
        let jwt_user = JwtUser::from_request(req)?;
        Ok(HttpResponse::Found()
            .header("Location", "/?info=You have been logged out successfully")
            .del_cookie(&jwt_cookie)
            .finish())
    } else {
        Ok(HttpResponse::Found().header("Location", "/").finish())
    }
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct SignUpForm {
    #[validate(regex = "RE_VALID_LOGIN")]
    login: String,
    #[validate(length(min = 4, max = 128))]
    password: String,
    #[validate(length(min = 2))]
    name: String,
}

#[post("/signup")]
pub async fn register_user(
    req: HttpRequest,
    sign_up_form: actix_web_validator::Form<SignUpForm>,
) -> Result<impl Responder, ApplicationError> {
    let mc = new_magic_crypt!(std::env::var("ENCRYPT_KEY")?, 256);
    let encrypted_password: String = mc.encrypt_str_to_base64(sign_up_form.password.as_str());
    let user_with_same_login: bool = user::Entity::login_exists(sign_up_form.login.clone()).await?;
    if user_with_same_login {
        warn!(
            "Peer {:?} tried to sign up but a user with the same username ({}) already exists",
            req.peer_addr(),
            sign_up_form.login
        );
        return Ok(HttpResponse::Found().header("Location", "?error=Someone with the same login already exists, please contact the administrator if you believe you are the owner of the account").finish());
    }

    user::Entity::insert_user(
        sign_up_form.login.clone(),
        sign_up_form.name.clone(),
        encrypted_password,
    )
    .await?;
    info!(
        "User {} has been created, access hasn't been granted yet",
        sign_up_form.login
    );
    let info_msg : String = format!("User {} has been created, you will need to wait for approval before being able to use this site's functionnalities.", sign_up_form.login);
    Ok(HttpResponse::Found()
        .header("Location", format!("/?info={}", info_msg))
        .finish())
}
