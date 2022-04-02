pub mod admin;
pub mod game;
pub mod unauth;
pub mod user;

lazy_static! {
    static ref RE_TABLE_NAME: regex::Regex =
        regex::Regex::new(r####"^\d{4}-[0-1][0-9]-[0-3][0-9]$"####).unwrap();
}

#[derive(serde::Deserialize, validator::Validate)]
pub struct ContextQuery {
    info: Option<String>,
    error: Option<String>,
    page: Option<usize>,
    #[validate(range(min = 0))]
    id: Option<i32>,
    code: Option<String>,
    #[validate(regex = "RE_TABLE_NAME")]
    date: Option<String>,
    per_page: Option<usize>,
}
