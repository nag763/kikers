#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_more;

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
