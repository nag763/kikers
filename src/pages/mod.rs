pub mod admin;
pub mod unauth;

#[derive(serde::Deserialize)]
pub struct ContextQuery {
    info: Option<String>,
    error: Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
}
