use crate::pages::ContextQuery;
use askama::Template;
use ffb_auth::JwtUser;

use ffb_structs::{country, country::Model as Country, league, league::Model as APILeague, user};

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
    fav_leagues_id: Vec<u32>,
    leagues: Option<Vec<APILeague>>,
    fav_leagues: Option<Vec<APILeague>>,
}

#[get("/profile/leagues")]
pub async fn user_leagues(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let fav_leagues_id : Vec<u32> = user::Entity::get_favorite_leagues_id(jwt_user.id).await?;
    let (leagues, fav_leagues): (Option<Vec<APILeague>>, Option<Vec<APILeague>>) =
        match &context_query.code {
            Some(v) => (Some(league::Entity::get_leagues_for_country_code(v)?), None),
            None => (
                None,
                Some(league::Entity::get_fav_leagues_of_user(
                        fav_leagues_id.clone()
                )?),
            ),
        };
    let countries: Vec<Country> = country::Entity::find_all()?;
    let index = UserLeagueTemplate {
        title: "Your informations".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        leagues,
        fav_leagues,
        fav_leagues_id,
        countries,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}

#[derive(Template)]
#[template(path = "users/clubs.html")]
struct UserClubsTemplate {
    title: String,
    user: Option<JwtUser>,
    error: Option<String>,
    info: Option<String>,
}

#[get("/profile/clubs")]
pub async fn user_club(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let index = UserClubsTemplate {
        title: "Your favorite club".into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
