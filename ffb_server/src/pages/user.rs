use crate::pages::ContextQuery;
use crate::ApplicationData;
use askama::Template;
use ffb_auth::JwtUser;

use ffb_structs::{club, club::Model as Club, league::Model as APILeague, league::EntityBuilder as LeagueBuilder, user};

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
    app_data: web::Data<ApplicationData>,
}

#[get("/profile/edit")]
pub async fn user_profile(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let index = UserTemplate {
        title: app_data
            .translate("M2002_TITLE", &jwt_user.locale_id)?
            .into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        app_data,
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
    fav_leagues_id: Vec<u32>,
    searched_leagues: Option<Vec<APILeague>>,
    fav_leagues: Option<Vec<APILeague>>,
    app_data: web::Data<ApplicationData>,
}

#[get("/profile/leagues")]
pub async fn user_leagues(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let fav_leagues_id: Vec<u32> = user::Entity::get_favorite_leagues_id(jwt_user.id).await?;
    let (searched_leagues, fav_leagues): (Option<Vec<APILeague>>, Option<Vec<APILeague>>) =
        match &context_query.search {
            Some(v) => (Some(LeagueBuilder::build().name(Some(v.into())).finish().await?), None),
            None => (
                None,
                Some(LeagueBuilder::build().ids(Some(fav_leagues_id.clone())).finish().await?),
            ),
        };
    let index = UserLeagueTemplate {
        title: app_data
            .translate("M2003_TITLE", &jwt_user.locale_id)?
            .into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        searched_leagues,
        fav_leagues,
        fav_leagues_id,
        app_data,
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
    searched_clubs: Option<Vec<Club>>,
    fav_clubs: Option<Vec<Club>>,
    fav_clubs_id: Vec<u32>,
    app_data: web::Data<ApplicationData>,
}

#[get("/profile/clubs")]
pub async fn user_club(
    req: HttpRequest,
    context_query: web::Query<ContextQuery>,
    app_data: web::Data<ApplicationData>,
) -> Result<HttpResponse, ApplicationError> {
    let jwt_user: JwtUser = JwtUser::from_request(req)?;
    let fav_clubs_id: Vec<u32> = user::Entity::get_favorite_clubs_id(jwt_user.id).await?;
    let (fav_clubs, searched_clubs): (Option<Vec<Club>>, Option<Vec<Club>>) =
        match &context_query.search {
            Some(search) => (
                None,
                Some(
                    club::EntityBuilder::build()
                        .name(Some(search.into()))
                        .finish()
                        .await?,
                ),
            ),
            None => (
                Some(
                    club::EntityBuilder::build()
                        .ids(Some(fav_clubs_id.clone()))
                        .finish()
                        .await?,
                ),
                None,
            ),
        };
    let index = UserClubsTemplate {
        title: app_data
            .translate("M2004_TITLE", &jwt_user.locale_id)?
            .into(),
        user: Some(jwt_user),
        error: context_query.error.clone(),
        info: context_query.info.clone(),
        app_data,
        searched_clubs,
        fav_clubs,
        fav_clubs_id,
    };
    Ok(HttpResponse::Ok().body(index.render()?))
}
