use ffb_auth::JwtUser;

use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use actix_web::{get, HttpRequest, HttpResponse};

use chrono::{DateTime, Utc};
use ffb_structs::{games::Entity as GamesEntity, games::Model as Games, user};

#[derive(Template)]
#[template(path = "games/game_row.html")]
struct GamesRowTemplate {
    games: Games,
    now: DateTime<Utc>,
    fetched_date: String,
    title: String,
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
}

#[get("/games")]
pub async fn games(
    req: HttpRequest,
    context_query: actix_web_validator::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let now: DateTime<Utc> = Utc::now();
    let fav_leagues: Option<Vec<u32>> = match context_query.all {
        Some(v) if v => None,
        _ => {
            let fav_leagues : Vec<u32> = user::Entity::get_favorite_leagues_id(jwt_user.id).await?;
            Some(fav_leagues)
        },
    };
    if let Some(query_date) = &context_query.date {
        let games: Option<Games> =
            GamesEntity::find_all_for_date(query_date.as_str(), fav_leagues)?;
        let subtemplate: Option<GamesRowTemplate> = match games {
            Some(games) if !games.games.is_empty() => Some(GamesRowTemplate {
                games,
                now,
                fetched_date: query_date.clone(),
                title: format!("Games for the {0}", query_date.as_str()),
            }),
            _ => None,
        };

        return Ok(HttpResponse::Ok().body(
            GamesOfDayTemplate {
                title: "Games".into(),
                user: Some(jwt_user),
                error: context_query.error.clone(),
                fetched_date: Some(query_date.clone()),
                info: context_query.info.clone(),
                day_games: subtemplate,
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

    let next_three_games: Option<Games> = GamesEntity::find_all_for_date_truncate_games(
        now_as_simple_date.as_str(),
        fav_leagues.clone(),
        3,
    )?;

    let next_three_games: Option<GamesRowTemplate> = match next_three_games {
        Some(games) if !games.games.is_empty() => Some(GamesRowTemplate {
            games,
            now,
            fetched_date: now_as_simple_date,
            title: "Next three games".to_string(),
        }),
        _ => None,
    };

    let yesterday_three_games: Option<Games> = GamesEntity::find_all_for_date_truncate_games(
        yesterday_as_simple_date.as_str(),
        fav_leagues.clone(),
        3,
    )?;

    let yesterday_three_games: Option<GamesRowTemplate> = match yesterday_three_games {
        Some(games) if !games.games.is_empty() => Some(GamesRowTemplate {
            games,
            now,
            fetched_date: yesterday_as_simple_date.clone(),
            title: "Yesterday games".to_string(),
        }),
        _ => None,
    };
    let tomorow_three_games: Option<Games> = GamesEntity::find_all_for_date_truncate_games(
        tomorow_as_simple_date.as_str(),
        fav_leagues,
        3,
    )?;

    let tomorow_three_games: Option<GamesRowTemplate> = match tomorow_three_games {
        Some(games) if !games.games.is_empty() => Some(GamesRowTemplate {
            games,
            now,
            fetched_date: tomorow_as_simple_date.clone(),
            title: "Tomorow games".to_string(),
        }),
        _ => None,
    };

    let index = GamesTemplate {
        title: "Games".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        next_three_games,
        yesterday_three_games,
        tomorow_three_games,
        fetched_date: None,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
