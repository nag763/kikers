#[macro_use]
extern crate magic_crypt;
#[macro_use]
extern crate log;

mod auth;
mod controllers;
mod entities;
mod error;
mod middleware;
mod pages;

use crate::controllers::{cookies_approved, login, logout, register_user};
use crate::error::ApplicationError;
use crate::middleware::cookie_approval::CookieChecker;
use crate::middleware::role_checker::RoleChecker;
use crate::pages::admin::admin_dashboard;
use crate::pages::unauth::{cookies, index, signup};
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
            .service(
                fs::Files::new("/styles", "./styles")
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .default_service(web::route().to(|| ApplicationError::NotFound.error_response()))
            .service(cookies_approved)
            .service(
                web::scope("")
                    .wrap(CookieChecker::default())
                    .service(index)
                    .service(cookies)
                    .service(login)
                    .service(register_user)
                    .service(logout)
                    .service(signup)
                    .service(
                        web::scope("")
                            .wrap(RoleChecker::default())
                            .service(admin_dashboard),
                    ),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
