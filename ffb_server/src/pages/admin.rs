use ffb_auth::JwtUser;

use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use crate::ApplicationData;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

use ffb_structs::{bookmaker, bookmaker::Model as Bookmaker, user, user::Model as User};

#[derive(Template)]
#[template(path = "admin/users.html")]
struct Admin {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    chosen_user: Option<User>,
    data: Vec<User>,
    page: u32,
    per_page: u32,
    app_data: web::Data<ApplicationData>,
}

#[get("/admin/users")]
pub async fn admin_dashboard(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let page: u32 = context_query.page.unwrap_or(0);
    let per_page: u32 = context_query.per_page.unwrap_or(10);
    let data: Vec<User> =
        user::Entity::get_users_with_pagination(jwt_user.role, per_page, page).await?;
    let chosen_user: Option<User> = match context_query.id {
        Some(v) => user::Entity::find_by_id_with_role_check(v, jwt_user.role).await?,
        None => None,
    };
    let index = Admin {
        title: app_data
            .translate("M30001_TITLE", &jwt_user.locale_id)?
            .into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        data,
        chosen_user,
        page,
        per_page,
        app_data,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}

#[derive(Template)]
#[template(path = "admin/bookmakers.html")]
struct AdminBookmakers {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    data: Vec<Bookmaker>,
    app_data: web::Data<ApplicationData>,
}

#[get("/admin/bookmakers")]
pub async fn admin_bookmakers(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let bookmakers: Vec<Bookmaker> = bookmaker::Entity::get_all().await?;
    let index = AdminBookmakers {
        title: app_data
            .translate("M30002_TITLE", &jwt_user.locale_id)?
            .into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        data: bookmakers,
        app_data,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
