use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Venue {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Status {
    pub long: String,
    pub short: String,
    pub elapsed: Option<i32>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Fixture {
    pub id: i32,
    pub referee: Option<String>,
    pub timezone: String,
    pub date: DateTime<Utc>,
    pub venue: Venue,
    pub status: Status,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct League {
    pub id: i32,
    pub name: String,
    pub country: Option<String>,
    pub logo: String,
    pub flag: Option<String>,
    pub round: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub logo: String,
    pub winner: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Teams {
    pub home: Team,
    pub away: Team,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Goals {
    pub home: Option<u8>,
    pub away: Option<u8>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Country {
    pub name: String,
    pub code: Option<String>,
    pub flag: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct APILeague {
    pub league: League,
    pub country: Country,
}

#[derive(serde::Deserialize, Clone)]
pub struct Game {
    pub fixture: Fixture,
    pub league: League,
    pub teams: Teams,
    pub goals: Goals,
}

#[derive(serde::Deserialize, Clone)]
pub struct Games {
    pub games: Vec<Game>,
    pub fetched_on: Option<String>,
}
