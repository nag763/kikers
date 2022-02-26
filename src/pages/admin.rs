use crate::auth::JwtUser;
use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use crate::pages::get_jwt_user;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

#[derive(Template, Debug)]
#[template(path = "admin.html")]
struct Admin {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
}

#[get("/admin")]
pub async fn admin_dashboard(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let index = Admin {
        title: "Admin board".into(),
        user: Some(get_jwt_user(req)?),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
