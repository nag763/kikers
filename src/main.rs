#[macro_use] extern crate magic_crypt;

mod auth;
mod constants;
mod controllers;
mod entities;
mod pages;

use crate::controllers::{login, logout};
use crate::pages::index;
use actix_web::{App, HttpServer};
use actix_files as fs;

use dotenv::dotenv;



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    HttpServer::new(|| App::new().service(index).service(login).service(logout).service(
                fs::Files::new("/styles", "./styles")
                    .show_files_listing()
                    .use_last_modified(true),
            ))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
