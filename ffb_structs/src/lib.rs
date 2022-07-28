//! This crate is used to the common structures that are used within the app.
//!
//! Most of the data used within the application is either stored in a MySQL
//! or Mongo database. To retrieve or store them, it is needed to define them
//! within a crate.
//!
//! Some of the structures can also be stored within Redis. Some are entirely
//! stored within, others are cached through an entity builder.
//!
//! There are often three different types of structs declared within a crate :
//!
//! - Model : the structure independently from what is stored in DB.
//! - Entity : the methods used to fetch the entity from the database.
//! - EntityBuilder : a way to facilitate querying the most complex entities,
//! most of the time, the entity builder's request is cached until it either
//! expires or is revocked, winning some precious time.

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_more;

lazy_static! {
    static ref ASSETS_BASE_PATH: String = std::env::var("ASSETS_BASE_PATH").unwrap();
    static ref RE_HOST_REPLACER: regex::Regex =
        regex::Regex::new(r#"(?P<host>http(?:s)+://[^/]+)"#).unwrap();
}

#[cfg(feature = "cli")]
pub mod api_token;
pub mod bet;
pub mod bookmaker;
pub mod club;
pub(crate) mod common_api_structs;
pub(crate) mod database;
#[cfg(feature = "server")]
pub mod ddos;
pub mod error;
pub mod game;
pub mod info;
pub mod league;
#[cfg(feature = "server")]
pub mod locale;
pub mod navaccess;
pub mod odd;
pub mod role;
pub mod scoreboard;
pub mod scoreboard_entry;
pub mod season;
pub mod token;
pub mod transaction_result;
pub(crate) mod translation;
pub mod translation_manager;
pub mod user;
