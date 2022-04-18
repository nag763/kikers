use ffb_auth::JwtUser;

use crate::error::ApplicationError;
use actix_web::cookie::Cookie;
use actix_web::{get, post, HttpRequest, HttpResponse, Responder};
use ffb_structs::user;
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
    match JwtUser::emit(login_form.login.as_str(), login_form.password.as_str()).await? {
        Some(token) => {
            let cookie_path: String = std::env::var("JWT_TOKEN_PATH")?;
            let cookie: Cookie = Cookie::build(cookie_path.as_str(), &token)
                .path("/")
                .http_only(true)
                .finish();
            Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .cookie(cookie)
                .finish())
        }
        None => {
            warn!(
                "{:?} tried to connect with login {} without success",
                req.peer_addr(),
                login_form.login
            );
            thread::sleep(std::time::Duration::from_secs(3));
            Ok(HttpResponse::Found()
                .append_header((
                    "Location",
                    "/?error=We couldn't connect you, please verify your credentials",
                ))
                .finish())
        }
    }
}

#[get("/logout")]
pub async fn logout(req: HttpRequest) -> Result<impl Responder, ApplicationError> {
    if let Some(mut jwt_cookie) = req.cookie(std::env::var("JWT_TOKEN_PATH")?.as_str()) {
        if let Ok(jwt_user) = JwtUser::from_request(req) {
            JwtUser::revoke_session(&jwt_user.login, jwt_cookie.value())?;
        }
        jwt_cookie.make_removal();
        Ok(HttpResponse::Found()
            .append_header(("Location", "/?info=You have been logged out successfully"))
            .cookie(jwt_cookie)
            .finish())
    } else {
        Ok(HttpResponse::Found().append_header(("Location", "/")).finish())
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
    let user_with_same_login: bool = user::Entity::login_exists(&sign_up_form.login).await?;
    if user_with_same_login {
        warn!(
            "Peer {:?} tried to sign up but a user with the same username ({}) already exists",
            req.peer_addr(),
            sign_up_form.login
        );
        return Ok(HttpResponse::Found().append_header(("Location", "?error=Someone with the same login already exists, please contact the administrator if you believe you are the owner of the account")).finish());
    }

    user::Entity::insert_user(
        &sign_up_form.login,
        &sign_up_form.name,
        &JwtUser::encrypt_key(&sign_up_form.password)?,
    )
    .await?;
    info!(
        "User {} has been created, access hasn't been granted yet",
        sign_up_form.login
    );
    let info_msg : String = format!("User {} has been created, you will need to wait for approval before being able to use this site's functionnalities.", sign_up_form.login);
    Ok(HttpResponse::Found()
        .append_header(("Location", format!("/?info={}", info_msg)))
        .finish())
}
