use ffb_auth::JwtUser;

use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use crate::ApplicationData;
use actix_web::{get, web, HttpRequest, HttpResponse};

use chrono::{DateTime, Utc};
use ffb_structs::{game::Entity as GameEntity, game::Model as Game, user};

#[derive(Template)]
#[template(path = "games/game_row.html")]
struct GamesRowTemplate {
    games: Vec<Game>,
    user_role: u32,
    now: DateTime<Utc>,
    fetched_date: String,
    title: String,
    fetched_on: Option<String>,
    app_data: web::Data<ApplicationData>,
    user: Option<JwtUser>,
}

#[derive(Template)]
#[template(path = "games/games_dashboard.html")]
struct GamesTemplate {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    next_three_games: Option<GamesRowTemplate>,
    yesterday_three_games: Option<GamesRowTemplate>,
    tomorow_three_games: Option<GamesRowTemplate>,
    fetched_date: Option<String>,
    app_data: web::Data<ApplicationData>,
}

#[derive(Template)]
#[template(path = "games/games_of_day.html")]
struct GamesOfDayTemplate {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    fetched_date: Option<String>,
    day_games: Option<GamesRowTemplate>,
    app_data: web::Data<ApplicationData>,
}

#[get("/games")]
pub async fn games(
    req: HttpRequest,
    context_query: actix_web_validator::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let now: DateTime<Utc> = Utc::now();
    let (fav_leagues, fav_clubs): (Option<Vec<u32>>, Option<Vec<u32>>) = match context_query.all {
        Some(v) if v => (None, None),
        _ => (
            Some(user::Entity::get_favorite_leagues_id(jwt_user.id).await?),
            Some(user::Entity::get_favorite_clubs_id(jwt_user.id).await?),
        ),
    };
    if let Some(query_date) = &context_query.date {
        let games: Vec<Game> =
            GameEntity::find_all_for_date(query_date.as_str(), fav_leagues, fav_clubs, None)
                .await?;
        let subtemplate: Option<GamesRowTemplate> = match games.is_empty() {
            false => Some(GamesRowTemplate {
                games,
                now,
                fetched_date: query_date.clone(),
                fetched_on: GameEntity::get_last_fetched_timestamp_for_date(query_date)?,
                user_role: jwt_user.role,
                title: app_data.translate("M10001_GAME_OF_DAY", &jwt_user.locale_id)?.into(),
                app_data: app_data.clone(),
                user: Some(jwt_user.clone()),
            }),
            true => None,
        };

        return Ok(HttpResponse::Ok().body(
            GamesOfDayTemplate {
                title: "Games".into(),
                user: Some(jwt_user),
                error: context_query.error.clone(),
                fetched_date: Some(query_date.clone()),
                info: context_query.info.clone(),
                day_games: subtemplate,
                app_data,
            }
            .render()?,
        ));
    }
    let mut now_as_simple_date: String = now.to_rfc3339();
    now_as_simple_date.truncate(10);
    let mut yesterday_as_simple_date: String = (now - chrono::Duration::days(1)).to_rfc3339();
    yesterday_as_simple_date.truncate(10);
    let mut tomorow_as_simple_date: String = (now + chrono::Duration::days(1)).to_rfc3339();
    tomorow_as_simple_date.truncate(10);
    let now_as_simple_date: String = now_as_simple_date;

    let next_three_games: Vec<Game> = GameEntity::find_all_for_date(
        now_as_simple_date.as_str(),
        fav_leagues.clone(),
        fav_clubs.clone(),
        Some(3),
    )
    .await?;

    let next_three_games: Option<GamesRowTemplate> = match next_three_games.is_empty() {
        false => Some(GamesRowTemplate {
            games: next_three_games,
            fetched_on: GameEntity::get_last_fetched_timestamp_for_date(
                now_as_simple_date.as_str(),
            )?,
            now,
            fetched_date: now_as_simple_date,
            title: app_data.translate("M10001_TODAY_TITLE", &jwt_user.locale_id)?.into(),
            user_role: jwt_user.role,
            app_data:app_data.clone(),
                user: Some(jwt_user.clone()),
        }),
        true => None,
    };

    let yesterday_three_games: Vec<Game> = GameEntity::find_all_for_date(
        yesterday_as_simple_date.as_str(),
        fav_leagues.clone(),
        fav_clubs.clone(),
        Some(3),
    )
    .await?;

    let yesterday_three_games: Option<GamesRowTemplate> = match yesterday_three_games.is_empty() {
        false => Some(GamesRowTemplate {
            games: yesterday_three_games,
            now,
            fetched_date: yesterday_as_simple_date.clone(),
            title: app_data.translate("M10001_YESTERDAY_TITLE", &jwt_user.locale_id)?.into(),
            fetched_on: GameEntity::get_last_fetched_timestamp_for_date(
                yesterday_as_simple_date.as_str(),
            )?,
            user_role: jwt_user.role,
            app_data:app_data.clone(),
                user: Some(jwt_user.clone()),
        }),
        _ => None,
    };
    let tomorow_three_games: Vec<Game> = GameEntity::find_all_for_date(
        tomorow_as_simple_date.as_str(),
        fav_leagues,
        fav_clubs,
        Some(3),
    )
    .await?;

    let tomorow_three_games: Option<GamesRowTemplate> = match tomorow_three_games.is_empty() {
        false => Some(GamesRowTemplate {
            games: tomorow_three_games,
            now,
            fetched_date: tomorow_as_simple_date.clone(),
            title: app_data.translate("M10001_TOMOROW_TITLE", &jwt_user.locale_id)?.into(),
            fetched_on: GameEntity::get_last_fetched_timestamp_for_date(
                tomorow_as_simple_date.as_str(),
            )?,
            user_role: jwt_user.role,
            app_data:app_data.clone(),
                user: Some(jwt_user.clone()),
        }),
        true => None,
    };

    let index = GamesTemplate {
        title: app_data.translate("M10001_TITLE", &jwt_user.locale_id)?.into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        next_three_games,
        yesterday_three_games,
        tomorow_three_games,
        fetched_date: None,
        app_data,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
