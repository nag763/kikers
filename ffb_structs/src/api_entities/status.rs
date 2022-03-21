#[derive(Debug, Clone, serde::Deserialize)]
pub struct Status {
    pub long: String,
    pub short: String,
    pub elapsed: Option<i64>,
}
