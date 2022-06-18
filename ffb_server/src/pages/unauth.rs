use askama::Template;

use crate::error::ApplicationError;
use crate::pages::ContextQuery;
use crate::ApplicationData;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};
use ffb_auth::JwtUser;
use ffb_structs::{info, info::Model as Info};

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    news: Option<Vec<Info>>,
    app_data: web::Data<ApplicationData>,
}

#[get("/")]
pub async fn index(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let index: Index;
    match req.cookie(app_data.get_jwt_path()) {
        Some(token) => {
            let jwt_user = JwtUser::from_token(token.value())?;
            index = Index {
                title: app_data
                    .translate("HOME_WELCOME_BACK", &jwt_user.locale_id)?
                    .to_string(),
                user: Some(jwt_user),
                error: context_query.error.clone(),
                info: context_query.info.clone(),
                news: Some(info::Entity::get_all()?),
                app_data,
            };
        }
        None => {
            index = Index {
                title: "Login".to_string(),
                user: None,
                error: context_query.error.clone(),
                info: context_query.info.clone(),
                news: None,
                app_data,
            }
        }
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}

#[derive(Template)]
#[template(path = "signup.html")]
struct SignUp {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    app_data: web::Data<ApplicationData>,
}

#[get("/signup")]
pub async fn signup(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    if req.cookie(app_data.get_jwt_path()).is_none() {
        let sign_up: SignUp = SignUp {
            title: "Sign up".to_string(),
            user: None,
            error: context_query.error.clone(),
            info: context_query.info.clone(),
            app_data,
        };
        Ok(HttpResponse::Ok().body(sign_up.render()?))
    } else {
        Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish())
    }
}

#[derive(Template)]
#[template(path = "cookies.html")]
struct Cookies {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    app_data: web::Data<ApplicationData>,
}

#[get("/cookies")]
pub async fn cookies(
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let cookies: Cookies = Cookies {
        title: "Cookie approval".to_string(),
        user: None,
        error: None,
        info: None,
        app_data,
    };
    Ok(HttpResponse::Ok().body(cookies.render()?))
}
