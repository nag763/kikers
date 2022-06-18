#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate lazy_static;

mod application_data;
mod controllers;
mod error;
mod middleware;
mod pages;
mod uri_builder;

use crate::application_data::ApplicationData;
use crate::controllers::admin::admin_bookmakers as c_admin_bookmakers;
use crate::controllers::auth::{login, logout, register_user};
use crate::controllers::club::update_club_status;
use crate::controllers::cookies::cookies_approved;
use crate::controllers::game::update_game_status;
use crate::controllers::user::{
    user_activation, user_change_leagues, user_deletion, user_modification, user_search,
    user_self_modification,
};
use crate::error::ApplicationError;
use crate::middleware::cookie_approval::CookieChecker;
use crate::middleware::ddos_limiter::DDosLimiter;
use crate::middleware::protect_assets::AssetsProtector;
use crate::middleware::role_checker::RoleChecker;
use crate::pages::admin::{admin_bookmakers, admin_dashboard};
use crate::pages::bets::my_bets;
use crate::pages::game::games;
use crate::pages::unauth::{cookies, index, signup};
use crate::pages::user::{user_club, user_leagues, user_profile};
use actix_files as fs;
use actix_web::middleware as actix_middleware;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::ResponseError;
use actix_web::{App, HttpServer};
use actix_web_validator::{FormConfig, QueryConfig};
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    log4rs::init_file("log4rs.yaml", Default::default())
        .expect("Log4rs file misconfigured or not found");
    let mydata = web::Data::new(ApplicationData::init().await.unwrap());
    HttpServer::new(move || {
        App::new()
            // Logging config
            .wrap(Logger::new("[%a]->'%U'(%s)"))
            .wrap(actix_middleware::DefaultHeaders::new()
                  .add(("X-Version", "0.1"))
                  .add(("Upgrade-Insecure-Requests", "1"))
                  .add(("Strict-Transport-Security", "max-age=63072000; includeSubDomains; preload"))
                  .add(("Content-Security-Policy", "default-src 'self'; base-uri 'self'; object-src 'none'; script-src 'nonce-2726c7f26c'; style-src 'self'; img-src 'self'"))
                  .add(("Content-Type", "text/html; charset=utf-8"))
                  .add(("X-Frame-Options", "DENY"))
                  .add(("Referrer-Policy", "origin-when-cross-origin"))
                  .add(("X-Content-Type-Options", "nosniff"))
                  .add(("Access-Control-Allow-Origin", "null"))
                  .add(("Permissions-Policy", "accelerometer=(), ambient-light-sensor=(), autoplay=(), battery=(), camera=(), cross-origin-isolated=(), display-capture=(), document-domain=(), encrypted-media=(), execution-while-not-rendered=(), execution-while-out-of-viewport=(), fullscreen=(), geolocation=(), gyroscope=(), keyboard-map=(), magnetometer=(), microphone=(), midi=(), navigation-override=(), payment=(), picture-in-picture=(), publickey-credentials-get=(), screen-wake-lock=(), sync-xhr=(), usb=(), web-share=(), xr-spatial-tracking=()"))
                  )
            // Default service config
            .default_service(web::to(|| async {
                ApplicationError::NotFound.error_response()
            }))
            // Validators config
            .app_data(FormConfig::default().error_handler(|err, _| {
                actix_web::error::InternalError::from_response(
                    err,
                    ApplicationError::BadRequest.error_response(),
                )
                .into()
            }))
            .app_data(QueryConfig::default().error_handler(|err, _| {
                actix_web::error::InternalError::from_response(
                    err,
                    ApplicationError::BadRequest.error_response(),
                )
                .into()
            }))
            .app_data(web::Data::clone(&mydata))
            // File services
            .service(
                web::scope("assets")
                    .wrap(AssetsProtector::default())
            .service(fs::Files::new("/styles", "./styles").use_last_modified(true))
                    .service(fs::Files::new("", "./assets").use_last_modified(true))
            )
            .service(cookies_approved)
            .service(
                web::scope("")
                    .service(cookies)
                    .wrap(CookieChecker::default())
                    .wrap(DDosLimiter::default())
                    .service(index)
                    .service(login)
                    .service(register_user)
                    .service(logout)
                    .service(signup)
                    .service(
                        web::scope("")
                            .wrap(RoleChecker::default())
                            .service(games)
                            .service(update_game_status)
                            .service(user_profile)
                            .service(user_leagues)
                            .service(user_change_leagues)
                            .service(user_club)
                            .service(user_self_modification)
                            .service(admin_dashboard)
                            .service(admin_bookmakers)
                            .service(user_search)
                            .service(user_activation)
                            .service(user_deletion)
                            .service(user_modification)
                            .service(update_club_status)
                            .service(c_admin_bookmakers)
                            .service(my_bets),
                    ),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
