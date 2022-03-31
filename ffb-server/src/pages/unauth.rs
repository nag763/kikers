use askama::Template;

use crate::auth::JwtUser;
use crate::error::ApplicationError;
use crate::pages::ContextQuery;
use actix_web::web;
use actix_web::HttpMessage;
use actix_web::{get, HttpRequest, HttpResponse};

#[derive(Template, Debug)]
#[template(path = "index.html")]
struct Index {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
}

#[get("/")]
pub async fn index(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let index: Index;
    match req.cookie(std::env::var("JWT_TOKEN_PATH")?.as_str()) {
        Some(token) => {
            let jwt_user = JwtUser::check_token(token.value())?;
            index = Index {
                title: format!("Welcome back {0}", jwt_user.name),
                user: Some(jwt_user),
                error: context_query.error.clone(),
                info: context_query.info.clone(),
            };
        }
        None => {
            index = Index {
                title: "Login".to_string(),
                user: None,
                error: context_query.error.clone(),
                info: context_query.info.clone(),
            }
        }
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}

#[derive(Template, Debug)]
#[template(path = "signup.html")]
struct SignUp {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
}

#[get("/signup")]
pub async fn signup(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    if req
        .cookie(std::env::var("JWT_TOKEN_PATH")?.as_str())
        .is_none()
    {
        let sign_up: SignUp = SignUp {
            title: "Sign up".to_string(),
            user: None,
            error: context_query.error.clone(),
            info: context_query.info.clone(),
        };
        Ok(HttpResponse::Ok().body(sign_up.render()?))
    } else {
        Ok(HttpResponse::Found().header("Location", "/").finish())
    }
}

#[derive(Template, Debug)]
#[template(path = "cookies.html")]
struct Cookies {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
}

#[get("/cookies")]
pub async fn cookies() -> Result<HttpResponse, ApplicationError> {
    let cookies: Cookies = Cookies {
        title: "Cookie approval".to_string(),
        user: None,
        error: None,
        info: None,
    };
    Ok(HttpResponse::Ok().body(cookies.render()?))
}
