use ffb_auth::JwtUser;

use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use crate::ApplicationData;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

#[derive(Template)]
#[template(path = "mybets.html")]
struct Admin {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    app_data: web::Data<ApplicationData>,
}

#[get("/mybets")]
pub async fn my_bets(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let index = Admin {
        title: app_data
            .translate("M30001_TITLE", &jwt_user.locale_id)?
            .into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        app_data,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
