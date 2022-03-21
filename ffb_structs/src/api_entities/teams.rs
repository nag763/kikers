use crate::api_entities::team::Team;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Teams {
    pub home: Team,
    pub away: Team,
}
