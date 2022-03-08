use crate::auth::JwtUser;
use crate::database::Database;
use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

use crate::api_structs::{Fixture, League, Teams};
use chrono::{DateTime, Utc};

#[derive(Template)]
#[template(path = "games/games.html")]
struct GamesTemplate {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    next_three_games: Option<Vec<Games>>,
}

#[derive(serde::Deserialize, Clone)]
struct Games {
    fixture: Fixture,
    league: League,
    teams: Teams,
}

#[get("/games")]
pub async fn games(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let now: DateTime<Utc> = Utc::now();
    let mut now_as_simple_date: String = now.to_rfc3339();
    now_as_simple_date.truncate(10);
    let now_as_simple_date : String = now_as_simple_date;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let mut redis_conn = Database::acquire_redis_connection()?;

    let games_as_string: Option<String> =
        redis::cmd("GET").arg(format!("fixtures-{}", now_as_simple_date)).query(&mut redis_conn)?;

    let next_three_games: Option<Vec<Games>> = match games_as_string {
        Some(v) => {
            let games: Vec<Games> = serde_json::from_str(v.as_str())?;
            let mut next_games : Vec<Games> = games
                .iter()
                .filter(|game| now < game.fixture.date)
                .cloned()
                .collect();
            next_games.truncate(3);
            if next_games.is_empty() {
                None
            } else {
                Some(next_games)
            }
        },
        None => None,
    };

    let index = GamesTemplate {
        title: "Games".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        next_three_games
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
