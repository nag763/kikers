use crate::auth::JwtUser;
use actix_web::http::Cookie;
use actix_web::web::Form;
use sea_orm::Set;
use sea_orm::ActiveModelTrait;
use crate::entities::user;
use actix_web::{get, post, HttpResponse, Responder, HttpRequest, HttpMessage};
use std::thread;
use magic_crypt::MagicCryptTrait;

#[derive(serde::Deserialize)]
pub struct LoginForm {
    login: String,
    password: String,
}

#[post("/login")]
pub async fn login(login_form: Form<LoginForm>) -> impl Responder {
    
    let mc = new_magic_crypt!(std::env::var("ENCRYPT_KEY").unwrap(), 256);
    let encrypted_password : String = mc.encrypt_str_to_base64(login_form.password.as_str());
    match JwtUser::emit(login_form.login.as_str(), encrypted_password.as_str()).await {
        Some(token) => {
            HttpResponse::Found()
                .header("Location", "/")
                .cookie(Cookie::new(super::constants::JWT_TOKEN_PATH, token))
                .finish()
        }
        None => {
            thread::sleep(std::time::Duration::from_secs(3));
            HttpResponse::Found().header("Location", "/").finish()
        }
    }
}

#[get("/logout")]
pub async fn logout(req : HttpRequest) -> impl Responder {
    let cookie: Cookie = req.cookie(super::constants::JWT_TOKEN_PATH).unwrap();
    HttpResponse::Found().header("Location", "/").del_cookie(&cookie).finish()
}

#[derive(serde::Deserialize)]
pub struct SignUpForm {
    login: String,
    name: String,
    password: String
}

#[post("/signup")]
pub async fn register_user(sign_up_form : Form<SignUpForm>) -> impl Responder {
    if minreq::get(format!("https://ws2.kik.com/user/{}", sign_up_form.login)).send().unwrap().status_code != 200 {
        return HttpResponse::NotFound().finish()
    }
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let conn = sea_orm::Database::connect(&db_url).await.unwrap();
    let mc = new_magic_crypt!(std::env::var("ENCRYPT_KEY").unwrap(), 256);
    let encrypted_password : String = mc.encrypt_str_to_base64(sign_up_form.password.as_str());
    let new_user = user::ActiveModel {
        login: Set(sign_up_form.login.to_owned()),
        name: Set(sign_up_form.name.to_owned()),
        password: Set(encrypted_password.to_owned()),
        ..Default::default() // all other attributes are `NotSet`
    };
    new_user.insert(&conn).await.expect("An error happened while persisting new user");
    HttpResponse::Found().header("Location", "/").finish()
}
