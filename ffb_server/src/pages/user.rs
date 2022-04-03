use crate::auth::JwtUser;
use crate::pages::ContextQuery;
use askama::Template;

use crate::api_structs::{APILeague, Country};
use crate::database::Database;
use crate::error::ApplicationError;
use actix_web::web;
use actix_web::{get, HttpRequest, HttpResponse};

#[derive(Template)]
#[template(path = "users/profile.html")]
struct UserTemplate {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
}

#[get("/profile/edit")]
pub async fn user_profile(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let index = UserTemplate {
        title: "Your informations".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}

#[derive(Template)]
#[template(path = "users/leagues.html")]
struct UserLeagueTemplate {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
    countries: Vec<Country>,
    leagues: Option<Vec<APILeague>>,
    fav_leagues: Option<Vec<APILeague>>,
}

#[get("/profile/leagues")]
pub async fn user_leagues(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let mut redis_conn = Database::acquire_redis_connection()?;
    let (leagues, fav_leagues): (Option<Vec<APILeague>>, Option<Vec<APILeague>>) =
        match &context_query.code {
            Some(v) => {
                let leagues_as_string: String =
                    redis::cmd("GET").arg("leagues").query(&mut redis_conn)?;
                let leagues: Vec<APILeague> = serde_json::from_str(leagues_as_string.as_str())?;
                if v.is_empty() {
                    (
                        Some(
                            leagues
                                .into_iter()
                                .filter(|league| league.country.code.is_none())
                                .collect::<Vec<APILeague>>(),
                        ),
                        None,
                    )
                } else {
                    (
                        Some(
                            leagues
                                .into_iter()
                                .filter(|league| {
                                    if let Some(code) = &league.country.code {
                                        return code == v;
                                    } else {
                                        return false;
                                    }
                                })
                                .collect(),
                        ),
                        None,
                    )
                }
            }
            None => {
                let leagues_as_string: String =
                    redis::cmd("GET").arg("leagues").query(&mut redis_conn)?;
                let leagues: Vec<APILeague> = serde_json::from_str(leagues_as_string.as_str())?;
                let fav_leagues: Vec<APILeague> = leagues
                    .into_iter()
                    .filter(|league| jwt_user.fav_leagues.contains(&league.league.id))
                    .collect();
                (None, Some(fav_leagues))
            }
        };
    let countries_as_string: String = redis::cmd("GET").arg("countries").query(&mut redis_conn)?;
    let countries: Vec<Country> = serde_json::from_str(countries_as_string.as_str())?;
    let index = UserLeagueTemplate {
        title: "Your informations".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        leagues,
        fav_leagues,
        countries,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
