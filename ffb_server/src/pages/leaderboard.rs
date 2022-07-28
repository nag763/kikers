use ffb_auth::JwtUser;

use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use crate::ApplicationData;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};
use ffb_structs::scoreboard::{EntityBuilder as ScoreboardBuilder, Model as Scoreboard};
use ffb_structs::season::{EntityBuilder as SeasonBuilder, Model as Season};

#[derive(Template)]
#[template(path = "leaderboard.html")]
struct Leaderboard {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    data: Scoreboard,
    seasons: Vec<Season>,
    app_data: web::Data<ApplicationData>,
}

#[get("/leaderboard")]
pub async fn leaderboard(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let mut scoreboard_builder = ScoreboardBuilder::build();
    if let Some(season_id) = context_query.id {
        scoreboard_builder.season_id(Some(season_id));
    } else if let Some(all_time) = context_query.all {
        scoreboard_builder.all_time(all_time);
    }
    let seasons: Vec<Season> = SeasonBuilder::build().finish().await?;
    let index = Leaderboard {
        title: app_data
            .translate("M40001_TITLE", &jwt_user.locale_id)?
            .into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        data: scoreboard_builder.finish().await?,
        seasons,
        app_data,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
