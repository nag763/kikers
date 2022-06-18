use ffb_auth::JwtUser;

use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use crate::ApplicationData;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};
use ffb_structs::{game::EntityBuilder as GameEntityBuilder, game::Model as Game};

#[derive(Template)]
#[template(path = "mybets.html")]
struct Admin {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    bets_available: Vec<Game>,
    app_data: web::Data<ApplicationData>,
}

#[get("/mybets")]
pub async fn my_bets(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let bets_available: Vec<Game> = GameEntityBuilder::build().bets_only(true).finish().await?;
    let index = Admin {
        title: app_data
            .translate("M30001_TITLE", &jwt_user.locale_id)?
            .into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        bets_available: bets_available,
        info: context_query.info.clone(),
        app_data,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
