#[macro_use]
extern crate magic_crypt;
#[macro_use]
extern crate log;

mod auth;
mod constants;
mod controllers;
mod entities;
mod error;
mod middleware;
mod pages;

use crate::controllers::{login, logout, register_user, cookies_approved};
use crate::middleware::CookieChecker;
use crate::pages::{cookies, index, signup};
use actix_files as fs;
use actix_web::web;
use actix_web::{App, HttpServer};

use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .service(
                fs::Files::new("/styles", "./styles")
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .service(cookies_approved)
            .service(
                web::scope("")
                    .wrap(CookieChecker::default())
                    .service(index)
                    .service(cookies)
                    .service(login)
                    .service(register_user)
                    .service(logout)
                    .service(signup),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
