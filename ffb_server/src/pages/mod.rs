pub mod admin;
pub mod game;
pub mod leaderboard;
pub mod unauth;
pub mod user;

lazy_static! {
    static ref RE_VALID_DATE: regex::Regex =
        regex::Regex::new(r####"^\d{4}-[0-1][0-9]-[0-3][0-9]$"####).unwrap();
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct ContextQuery {
    info: Option<String>,
    error: Option<String>,
    search: Option<String>,
    page: Option<u32>,
    #[validate(range(min = 0))]
    id: Option<u32>,
    #[validate(regex = "RE_VALID_DATE")]
    date: Option<String>,
    per_page: Option<u32>,
    all: Option<bool>,
    bets: Option<bool>,
    favs: Option<bool>,
    potential_bets: Option<bool>,
}
