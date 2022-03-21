#[derive(Debug, Clone, serde::Deserialize)]
pub struct League {
    pub id: i64,
    pub name: String,
    pub country: String,
    pub logo: String,
    pub flag: Option<String>,
    pub season: i64,
    pub round: String,
}
