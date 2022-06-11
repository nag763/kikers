use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Venue {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Status {
    pub long: String,
    pub short: String,
    pub elapsed: Option<u32>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Fixture {
    pub id: u32,
    pub referee: Option<String>,
    pub timezone: String,
    pub date: DateTime<Utc>,
    pub venue: Venue,
    pub status: Status,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Team {
    pub id: u32,
    pub name: String,
    pub logo: String,
    pub winner: Option<bool>,
    pub odd: Option<f32>,
    #[serde(rename = "localLogo", skip_serializing_if = "Option::is_none")]
    pub local_logo: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Teams {
    pub home: Team,
    pub away: Team,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Goals {
    pub home: Option<u8>,
    pub away: Option<u8>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Penalty {
    pub home: Option<u8>,
    pub away: Option<u8>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Score {
    pub penalty: Option<Penalty>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Country {
    pub name: String,
    pub code: Option<String>,
    pub flag: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Value {
    pub value: String,
    pub odd: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Bet {
    pub id: u32,
    pub name: String,
    pub values: Vec<Value>,
}
