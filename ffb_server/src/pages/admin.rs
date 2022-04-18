use ffb_auth::JwtUser;

use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

use ffb_structs::{user, user::Model as User};

#[derive(Template, Debug, Default)]
#[template(path = "admin.html")]
struct Admin {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    chosen_user: Option<User>,
    data: Vec<User>,
    page: u32,
    per_page: u32,
}

#[get("/admin")]
pub async fn admin_dashboard(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let page: u32 = context_query.page.unwrap_or(0);
    let per_page: u32 = context_query.per_page.unwrap_or(10);
    let data: Vec<User> =
        user::Entity::get_users_with_pagination(jwt_user.role, per_page, page).await?;
    let chosen_user: Option<User> = match context_query.id {
        Some(v) => user::Entity::find_by_id(v).await?,
        None => None,
    };
    let index = Admin {
        title: "User management".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        data,
        chosen_user,
        page,
        per_page,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
