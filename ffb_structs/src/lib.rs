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

pub mod bookmaker;
pub mod club;
pub(crate) mod common_api_structs;
pub(crate) mod database;
pub mod ddos;
pub mod error;
pub mod game;
pub mod league;
pub mod locale;
pub mod navaccess;
pub mod role;
pub mod token;
pub mod transaction_result;
pub(crate) mod translation;
pub mod translation_manager;
pub mod user;
pub mod api_token;
