#[derive(Debug, Clone, serde::Deserialize)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub logo: String,
    pub winner: Option<bool>,
}
