use askama::Template;

use crate::auth::JwtUser;
use actix_web::{get, HttpResponse, Responder, HttpRequest};
use actix_web::HttpMessage;


#[derive(Template, Debug)]
#[template(path = "index.html")]
struct Index {
    title: String,
    user: Option<JwtUser>,
}

#[get("/")]
pub async fn index(req: HttpRequest) -> impl Responder {
    let index: Index;
    match req.cookie(super::constants::JWT_TOKEN_PATH) {
        Some(token) =>  match JwtUser::check_token(token.value()) {
            Ok(jwt_user) => { index = Index { title: format!("Weclome back {0}", jwt_user.name), user: Some(jwt_user)}; },
            Err(_) => return HttpResponse::Forbidden().finish()
        },
        None => { index = Index { title: "Login".to_string(), user: None }}
    };
    HttpResponse::Ok().body(index.render().unwrap())
}

#[derive(Template, Debug)]
#[template(path = "signup.html")]
struct SignUp {
    title: String,
    user: Option<JwtUser>,
}


#[get("/signup")]
pub async fn signup(req: HttpRequest) -> impl Responder {
    if req.cookie(super::constants::JWT_TOKEN_PATH).is_none() {
        let sign_up : SignUp = SignUp { title: "Sign up".to_string(), user: None};
        HttpResponse::Ok().body(sign_up.render().unwrap())
    } else {
        HttpResponse::Found().header("Location", "/").finish()
    }
}
