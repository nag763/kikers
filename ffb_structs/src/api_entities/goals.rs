#[derive(Debug, Clone, serde::Deserialize)]
pub struct Goals {
    pub home: Option<u8>,
    pub away: Option<u8>,
}
