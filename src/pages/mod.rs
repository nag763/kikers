pub mod admin;
pub mod unauth;

#[derive(serde::Deserialize)]
pub struct ContextQuery {
    info: Option<String>,
    error: Option<String>,
    offset: Option<u64>,
}
