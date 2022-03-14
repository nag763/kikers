use crate::auth::JwtUser;
use crate::database::Database;
use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

use crate::api_structs::{Fixture, Goals, League, Teams};
use chrono::{DateTime, Utc};

#[derive(Template)]
#[template(path = "games/game_row.html")]
struct GamesRowTemplate {
    games: Vec<Games>,
    now: DateTime<Utc>,
    fetched_date: String,
    title: String,
}

#[derive(Template)]
#[template(path = "games/games.html")]
struct GamesTemplate {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    next_three_games: Option<GamesRowTemplate>,
    yesterday_three_games: Option<GamesRowTemplate>,
    tomorow_three_games: Option<GamesRowTemplate>,
}

#[derive(serde::Deserialize, Clone)]
struct Games {
    fixture: Fixture,
    league: League,
    teams: Teams,
    goals: Goals,
}

#[get("/games")]
pub async fn games(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let now: DateTime<Utc> = Utc::now();
    let mut now_as_simple_date: String = now.to_rfc3339();
    now_as_simple_date.truncate(10);
    let mut yesterday_as_simple_date: String = (now - chrono::Duration::days(1)).to_rfc3339();
    yesterday_as_simple_date.truncate(10);
    let mut tomorow_as_simple_date: String = (now + chrono::Duration::days(1)).to_rfc3339();
    tomorow_as_simple_date.truncate(10);
    let now_as_simple_date: String = now_as_simple_date;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let mut redis_conn = Database::acquire_redis_connection()?;

    let next_three_games_as_string: Option<String> = redis::cmd("HGET")
        .arg("fixtures")
        .arg(now_as_simple_date.clone())
        .query(&mut redis_conn)?;

    let next_three_games: Option<GamesRowTemplate> = match next_three_games_as_string {
        Some(v) => {
            let games: Vec<Games> = serde_json::from_str(v.as_str())?;
            let mut next_games: Vec<Games> = games
                .iter()
                .filter(|game| now < game.fixture.date)
                .cloned()
                .collect();
            next_games.truncate(3);
            if next_games.is_empty() {
                None
            } else {
                Some(GamesRowTemplate {
                    games: next_games,
                    now,
                    fetched_date: now_as_simple_date.clone(),
                    title: "Next three games".to_string(),
                })
            }
        }
        None => None,
    };

    let yesterday_games_as_string: Option<String> = redis::cmd("HGET")
        .arg("fixtures")
        .arg(yesterday_as_simple_date.clone())
        .query(&mut redis_conn)?;

    let yesterday_three_games: Option<GamesRowTemplate> = match yesterday_games_as_string {
        Some(v) => {
            let mut games: Vec<Games> = serde_json::from_str(v.as_str())?;
            games.truncate(3);
            if games.is_empty() {
                None
            } else {
                Some(GamesRowTemplate {
                    games,
                    now,
                    fetched_date: yesterday_as_simple_date.clone(),
                    title: "Yesterday games".to_string(),
                })
            }
        }
        None => None,
    };

    let tomorow_games_as_string: Option<String> = redis::cmd("HGET")
        .arg("fixtures")
        .arg(tomorow_as_simple_date.clone())
        .query(&mut redis_conn)?;

    let tomorow_three_games: Option<GamesRowTemplate> = match tomorow_games_as_string {
        Some(v) => {
            let mut games: Vec<Games> = serde_json::from_str(v.as_str())?;
            games.truncate(3);
            if games.is_empty() {
                None
            } else {
                Some(GamesRowTemplate {
                    games,
                    now,
                    fetched_date: tomorow_as_simple_date.clone(),
                    title: "Tomorow games".to_string(),
                })
            }
        }
        None => None,
    };

    let index = GamesTemplate {
        title: "Games".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        next_three_games,
        yesterday_three_games,
        tomorow_three_games,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
