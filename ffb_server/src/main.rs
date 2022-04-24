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

use crate::application_data::ApplicationData;
use crate::controllers::auth::{login, logout, register_user};
use crate::controllers::cookies::cookies_approved;
use crate::controllers::user::{
    user_activation, user_change_leagues, user_deletion, user_modification, user_search,
};
use crate::error::ApplicationError;
use crate::middleware::cookie_approval::CookieChecker;
use crate::middleware::role_checker::RoleChecker;
use crate::pages::admin::admin_dashboard;
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
                  .add(("Content-Security-Policy", "default-src 'self'; script-src 'nonce-2726c7f26c'; style-src 'self' 'nonce-7a616b6f6b'; img-src 'self' https://media.api-sports.io"))
                  .add(("Content-Type", "text/html; charset=utf-8"))
                  .add(("X-Frame-Options", "DENY"))
                  .add(("Referrer-Policy", "no-referrer"))
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
            .service(fs::Files::new("/styles", "./styles").use_last_modified(true))
            .service(fs::Files::new("/assets", "./assets").use_last_modified(true))
            .service(cookies_approved)
            .service(
                web::scope("")
                    .service(cookies)
                    .wrap(CookieChecker::default())
                    .service(index)
                    .service(login)
                    .service(register_user)
                    .service(logout)
                    .service(signup)
                    .service(
                        web::scope("")
                            .wrap(RoleChecker::default())
                            .service(games)
                            .service(user_profile)
                            .service(user_leagues)
                            .service(user_change_leagues)
                            .service(user_club)
                            .service(admin_dashboard)
                            .service(user_search)
                            .service(user_activation)
                            .service(user_deletion)
                            .service(user_modification),
                    ),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
