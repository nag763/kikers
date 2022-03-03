use crate::auth::JwtUser;
use crate::entities::user;
use crate::entities::user::Model as User;
use crate::error::ApplicationError;
use actix_web::http::Cookie;
use actix_web::web::Form;
use actix_web::{get, post, HttpMessage, HttpRequest, HttpResponse, Responder};
use magic_crypt::MagicCryptTrait;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::Set;
use std::thread;

#[derive(serde::Deserialize)]
pub struct LoginForm {
    login: String,
    password: String,
}

#[post("/login")]
pub async fn login(
    req: HttpRequest,
    login_form: Form<LoginForm>,
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
        Ok(HttpResponse::Found()
            .header("Location", "/?info=You have been logged out successfully")
            .del_cookie(&jwt_cookie)
            .finish())
    } else {
        Ok(HttpResponse::Found().header("Location", "/").finish())
    }
}

#[derive(serde::Deserialize)]
pub struct SignUpForm {
    login: String,
    name: String,
    password: String,
}

#[post("/signup")]
pub async fn register_user(
    req: HttpRequest,
    sign_up_form: Form<SignUpForm>,
) -> Result<impl Responder, ApplicationError> {
    if minreq::get(format!("https://ws2.kik.com/user/{}", sign_up_form.login))
        .send()?
        .status_code
        != 200
    {
        warn!(
            "User {} has requested an account creation but isn't an existing kik user",
            sign_up_form.login
        );
        return Ok(HttpResponse::Found().header("Location", "?error=The given login isn't associed to any kik login, please ensure you use an existing kik login").finish());
    }

    let db_url = std::env::var("DATABASE_URL")?;
    let conn = sea_orm::Database::connect(&db_url).await?;
    let mc = new_magic_crypt!(std::env::var("ENCRYPT_KEY")?, 256);
    let encrypted_password: String = mc.encrypt_str_to_base64(sign_up_form.password.as_str());

    let user_with_same_login: Option<User> = user::Entity::find()
        .filter(Condition::all().add(user::Column::Login.eq(sign_up_form.login.to_owned())))
        .one(&conn)
        .await?;
    if user_with_same_login.is_some() {
        warn!(
            "Peer {:?} tried to sign up but a user with the same username ({}) already exists",
            req.peer_addr(),
            sign_up_form.login
        );
        return Ok(HttpResponse::Found().header("Location", "?error=Someone with the same login already exists, please contact the administrator if you believe you are the owner of the account").finish());
    }

    let new_user = user::ActiveModel {
        login: Set(sign_up_form.login.to_owned()),
        name: Set(sign_up_form.name.to_owned()),
        password: Set(encrypted_password.to_owned()),
        ..Default::default()
    };
    new_user.insert(&conn).await?;
    info!(
        "User {} has been created, access hasn't been granted yet",
        sign_up_form.login
    );
    let info_msg : String = format!("User {} has been created, you will need to wait for approval before being able to use this site's functionnalities.", sign_up_form.login);
    Ok(HttpResponse::Found()
        .header("Location", format!("/?info={}", info_msg))
        .finish())
}
