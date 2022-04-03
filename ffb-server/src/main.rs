#[macro_use]
extern crate magic_crypt;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod api_structs;
mod auth;
mod controllers;
mod database;
mod entities;
mod error;
mod middleware;
mod pages;

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
use crate::pages::user::{user_leagues, user_profile};
use actix_files as fs;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::ResponseError;
use actix_web::{App, HttpServer};
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    log4rs::init_file("log4rs.yaml", Default::default())
        .expect("Log4rs file misconfigured or not found");
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::new("[%a]->'%U'(%s)"))
            .service(fs::Files::new("/styles", "./styles").use_last_modified(true))
            .service(fs::Files::new("/assets", "./assets").use_last_modified(true))
            .default_service(web::route().to(|| ApplicationError::NotFound.error_response()))
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
