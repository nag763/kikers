use crate::auth::JwtUser;
use actix_web::http::Cookie;
use actix_web::web::Form;
use actix_web::{get, post, HttpResponse, Responder, HttpRequest, HttpMessage};
use std::thread;

#[derive(serde::Deserialize)]
pub struct LoginForm {
    login: String,
    password: String,
}

#[post("/login")]
pub async fn login(login_form: Form<LoginForm>) -> impl Responder {
    
    match JwtUser::emit(login_form.login.as_str(), login_form.password.as_str()).await {
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
