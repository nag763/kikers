pub mod admin;
pub mod game;
pub mod unauth;
pub mod user;

#[derive(serde::Deserialize)]
pub struct ContextQuery {
    info: Option<String>,
    error: Option<String>,
    page: Option<usize>,
    id: Option<i32>,
    per_page: Option<usize>,
}
