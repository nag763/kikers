use crate::auth::JwtUser;
use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

#[derive(Template)]
#[template(path = "users/user.html")]
struct UserTemplate {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
}

#[get("/profile")]
pub async fn profile(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let index = UserTemplate {
        title: "Your informations".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
