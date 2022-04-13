use crate::auth::JwtUser;
use crate::database::Database;
use crate::pages::ContextQuery;
use askama::Template;

use crate::error::ApplicationError;
use actix_web::{get, HttpRequest, HttpResponse};

use crate::api_structs::{Game, Games};
use chrono::{DateTime, Utc};

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
    let mut redis_conn = Database::acquire_redis_connection()?;
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let now: DateTime<Utc> = Utc::now();
    let all_games: bool = context_query.all.unwrap_or(false);
    let fav_leagues: Vec<i32> = jwt_user.clone().fav_leagues;
    match &context_query.date {
        Some(v) => {
            let games_of_the_day_as_string: Option<String> = redis::cmd("HGET")
                .arg("fixtures")
                .arg(v)
                .query(&mut redis_conn)?;
            let games_of_the_day: Option<GamesRowTemplate> = match games_of_the_day_as_string {
                Some(games) => {
                    let mut games: Games = serde_json::from_str(games.as_str())?;
                    if !all_games {
                        let list_of_games: Vec<Game> = games.games;
                        games.games = list_of_games
                            .into_iter()
                            .filter(|game| *&fav_leagues.contains(&game.league.id))
                            .collect();
                    }
                    if games.games.is_empty() {
                        None
                    } else {
                        Some(GamesRowTemplate {
                            games,
                            now,
                            fetched_date: v.clone(),
                            title: format!("Games for the {0}", v),
                        })
                    }
                }
                None => None,
            };
            return Ok(HttpResponse::Ok().body(
                GamesOfDayTemplate {
                    title: "Games".into(),
                    user: Some(jwt_user),
                    error: context_query.error.clone(),
                    fetched_date: Some(v.clone()),
                    info: context_query.info.clone(),
                    day_games: games_of_the_day,
                }
                .render()?,
            ));
        }
        None => {}
    }
    let mut now_as_simple_date: String = now.to_rfc3339();
    now_as_simple_date.truncate(10);
    let mut yesterday_as_simple_date: String = (now - chrono::Duration::days(1)).to_rfc3339();
    yesterday_as_simple_date.truncate(10);
    let mut tomorow_as_simple_date: String = (now + chrono::Duration::days(1)).to_rfc3339();
    tomorow_as_simple_date.truncate(10);
    let now_as_simple_date: String = now_as_simple_date;

    let next_three_games_as_string: Option<String> = redis::cmd("HGET")
        .arg("fixtures")
        .arg(now_as_simple_date.clone())
        .query(&mut redis_conn)?;

    let next_three_games: Option<GamesRowTemplate> = match next_three_games_as_string {
        Some(v) => {
            let mut games: Games = serde_json::from_str(v.as_str())?;
            if !all_games {
                let list_of_games: Vec<Game> = games.games;
                games.games = list_of_games
                    .into_iter()
                    .filter(|game| *&fav_leagues.contains(&game.league.id))
                    .collect();
            }
            games.games.truncate(3);
            if games.games.is_empty() {
                None
            } else {
                Some(GamesRowTemplate {
                    games: games,
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
            let mut games: Games = serde_json::from_str(v.as_str())?;
            if !all_games {
                let list_of_games: Vec<Game> = games.games;
                games.games = list_of_games
                    .into_iter()
                    .filter(|game| *&fav_leagues.contains(&game.league.id))
                    .collect();
            }
            games.games.truncate(3);
            if games.games.is_empty() {
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
            let mut games: Games = serde_json::from_str(v.as_str())?;
            if !all_games {
                let list_of_games: Vec<Game> = games.games;
                games.games = list_of_games
                    .into_iter()
                    .filter(|game| *&fav_leagues.contains(&game.league.id))
                    .collect();
            }
            games.games.truncate(3);
            if games.games.is_empty() {
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
        fetched_date: None,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
