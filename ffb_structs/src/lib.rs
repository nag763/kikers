#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_more;

lazy_static! {
    static ref ASSETS_BASE_PATH : String = std::env::var("ASSETS_BASE_PATH").unwrap();
    static ref RE_HOST_REPLACER: regex::Regex =
        regex::Regex::new(r#"(?P<host>http(?:s)+://[^/]+)"#).unwrap();
}

pub(crate) mod common_api_structs;
pub mod country;
pub(crate) mod database;
pub mod ddos;
pub mod error;
pub mod game;
pub mod league;
pub mod navaccess;
pub mod role;
pub mod token;
pub mod transaction_result;
pub mod user;
