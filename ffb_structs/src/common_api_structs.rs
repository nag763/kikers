use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Venue {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Display)]
pub enum ShortStatus {
    TBD,
    NS,
    #[serde(rename = "1H")] 
    FH,
    HT,
    #[serde(rename = "2H")] 
    SH,
    ET,
    P,
    FT,
    AET,
    PEN,
    BT,
    SUSP,
    INT,
    PST,
    CANC,
    ABD,
    AWD,
    WO,
    LIVE
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Status {
    pub long: String,
    pub short: ShortStatus,
    pub elapsed: Option<u32>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Fixture {
    pub id: u32,
    pub referee: Option<String>,
    pub timezone: String,
    pub timestamp: f64,
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
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Teams {
    pub home: Team,
    pub away: Team,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Goals {
    pub home: Option<i16>,
    pub away: Option<i16>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Score {
    pub fulltime: Option<Goals>,
    pub extratime: Option<Goals>,
    pub penalty: Option<Goals>,
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct Better {
    pub user_id: u32,
    pub game_result: crate::bet::GameResult
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Odds {
    pub home: f32,
    pub draw: f32,
    pub away: f32,
}
