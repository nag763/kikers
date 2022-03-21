use crate::api_entities::{fixture::Fixture, goals::Goals, league::League, teams::Teams};

#[derive(serde::Deserialize, Clone)]
pub struct Games {
    pub fixture: Fixture,
    pub league: League,
    pub teams: Teams,
    pub goals: Goals,
}
