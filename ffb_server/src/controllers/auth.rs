use ffb_auth::JwtUser;

use crate::error::ApplicationError;
use crate::ApplicationData;
use actix_web::cookie::{time::Duration, Cookie};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use ffb_structs::user;

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
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    match JwtUser::emit(login_form.login.as_str(), login_form.password.as_str()).await? {
        Some(token) => {
            let cookie_path: &str = app_data.get_jwt_path();
            let cookie: Cookie = Cookie::build(cookie_path, &token)
                .path("/")
                .http_only(true)
                .secure(true)
                .max_age(Duration::days(7))
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
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
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
pub async fn logout(
    req: HttpRequest,
    app_data: web::Data<ApplicationData>,
) -> Result<impl Responder, ApplicationError> {
    if let Some(mut jwt_cookie) = req.cookie(app_data.get_jwt_path()) {
        if let Ok(jwt_user) = JwtUser::from_request(req) {
            JwtUser::revoke_session(&jwt_user.login, jwt_cookie.value())?;
        }
        jwt_cookie.make_removal();
        Ok(HttpResponse::Found()
            .append_header(("Location", "/?info=You have been logged out successfully"))
            .cookie(jwt_cookie)
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish())
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
    locale_id: u32,
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

    let result: bool = user::Entity::insert_user(
        &sign_up_form.login,
        &sign_up_form.name,
        sign_up_form.locale_id,
        &JwtUser::encrypt_key(&sign_up_form.password)?,
    )
    .await?
    .into();
    let response: String = match result {
        true => {
            info!(
                "User {} has been created, access hasn't been granted yet",
                sign_up_form.login
            );
            format!("/?info=User {} has been created, you will need to wait for approval before being able to use this site's functionnalities.", sign_up_form.login)
        }
        false =>"/signup/?error=An error happened while trying to create your user".to_string(),
    };
    Ok(HttpResponse::Found()
        .append_header(("Location", response))
        .finish())
}
