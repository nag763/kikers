use askama::Template;

use crate::error::ApplicationError;
use crate::pages::ContextQuery;
use crate::ApplicationData;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use ffb_auth::JwtUser;
use ffb_structs::{
    game::Entity as GameEntity, game::EntityBuilder as GameEntityBuilder, game::Model as Game,
    info, info::Model as Info, scoreboard::EntityBuilder as ScoreboardBuilder,
    scoreboard::Model as Scoreboard, season, user,
};

#[derive(Template)]
#[template(path = "games/game_row.html")]
struct GamesRowTemplate {
    games: Vec<Game>,
    user_role: u32,
    current_season_id: u32,
    now: DateTime<Utc>,
    fetched_date: String,
    title: String,
    fetched_on: Option<String>,
    app_data: web::Data<ApplicationData>,
    user: Option<JwtUser>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    news: Option<Vec<Info>>,
    leaderboard: Option<Scoreboard>,
    games_going_on: Option<GamesRowTemplate>,
    app_data: web::Data<ApplicationData>,
}

#[get("/")]
pub async fn index(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let index: Index;
    let current_season_id: u32 = season::Entity::get_current_season_id().await?;
    match req.cookie(app_data.get_jwt_path()) {
        Some(token) => {
            let jwt_user = JwtUser::from_token(token.value())?;
            let now: DateTime<Utc> = Utc::now();
            let mut now_as_simple_date: String = now.to_rfc3339();
            now_as_simple_date.truncate(10);
            let games: Vec<Game> = GameEntityBuilder::build()
                .limit(2)
                .date(&now_as_simple_date)
                .clubs(user::Entity::get_favorite_clubs_id(jwt_user.id).await?)
                .leagues(user::Entity::get_favorite_leagues_id(jwt_user.id).await?)
                .finish()
                .await?;
            let games_going_on: Option<GamesRowTemplate> = match games.is_empty() {
                false => Some(GamesRowTemplate {
                    title: app_data
                        .translate("M10001_TODAY_TITLE", &jwt_user.locale_id)?
                        .into(),
                    games,
                    user_role: jwt_user.role,
                    now,
                    current_season_id,
                    app_data: app_data.clone(),
                    fetched_on: GameEntity::get_last_fetched_timestamp_for_date(
                        &now_as_simple_date,
                    )?,
                    fetched_date: now_as_simple_date,
                    user: Some(jwt_user.clone()),
                }),
                true => None,
            };
            let leaderboard: Scoreboard =
                ScoreboardBuilder::build().limit(Some(3)).finish().await?;

            index = Index {
                title: app_data
                    .translate("HOME_WELCOME_BACK", &jwt_user.locale_id)?
                    .to_string(),
                user: Some(jwt_user),
                error: context_query.error.clone(),
                info: context_query.info.clone(),
                news: Some(info::Entity::get_all()?),
                games_going_on,
                leaderboard: Some(leaderboard),
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
                games_going_on: None,
                leaderboard: None,
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
