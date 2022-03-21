#[derive(Debug, Clone, serde::Deserialize)]
pub struct Venue {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub city: Option<String>,
}
