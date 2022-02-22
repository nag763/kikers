use askama::Template;

use crate::auth::JwtUser;
use crate::error::ApplicationError;
use actix_web::web;
use actix_web::HttpMessage;
use actix_web::{get, HttpRequest, HttpResponse};

#[derive(serde::Deserialize)]
pub struct ContextQuery {
    info: Option<String>,
    error: Option<String>,
}

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
    match req.cookie(super::constants::JWT_TOKEN_PATH) {
        Some(token) => {
            let jwt_user = JwtUser::check_token(token.value())?;
            index = Index {
                title: format!("Weclome back {0}", jwt_user.name),
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
    if req.cookie(super::constants::JWT_TOKEN_PATH).is_none() {
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
