use chrono::{DateTime, Utc};

/// A venue is a place where a game is played.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Venue {
    /// Its in base id.
    pub id: Option<u32>,
    /// The venue name.
    pub name: Option<String>,
    /// The city of the venue.
    pub city: Option<String>,
}

/// The status of a game.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Display)]
#[serde(rename_all = "UPPERCASE")]
pub enum ShortStatus {
    /// To be defined.
    ///
    /// The game's play date is unknown yet.
    Tbd,
    /// Not started.
    ///
    /// The game's play date is known.
    Ns,
    /// First half.
    ///
    /// The game is being currently played, and the break didn't occur yet.
    #[serde(rename = "1H")]
    Fh,
    /// Half time.
    ///
    /// The game's break is going on.
    Ht,
    /// Second half.
    ///
    /// The game's break is over, the game has resumed.
    #[serde(rename = "2H")]
    Sh,
    /// Extra time.
    ///
    /// The game is in the extra time of play, 90 minutes have already been 
    /// played.
    Et,
    /// Penalties.
    ///
    /// The game's extra time didn't decide a winner, penalties are being taken
    /// to decide who is the winner.
    P,
    /// Full time.
    ///
    /// The game has been finished within the reglementary time.
    Ft,
    /// After extra time.
    ///
    /// The game has been finished after extra time.
    Aet,
    /// Penalties.
    ///
    /// The game's winner has been decided on penalties.
    Pen,
    Bt,
    /// Suspended.
    ///
    /// The game has been suspended, it will resume soon.
    Susp,
    /// Interupted.
    ///
    /// The game has been interupted, it will resume pretty soon.
    Int,
    Pst,
    /// The game has been cancelled.
    ///
    /// It won't resume.
    Canc,
    /// Abandonned.
    Abd,
    /// Awarded.
    ///
    /// The game has a winner that hasn't been decided during the play time.
    Awd,
    Wo,
    /// Live, the game is currently in live.
    Live,
}

/// The status of a game.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Status {
    /// The long's status, describe what is happening.
    pub long: String,
    /// The short status, describe in two letters code how is the game going.
    pub short: ShortStatus,
    /// How long the game has been going for.
    pub elapsed: Option<u32>,
}

/// A fixture defines several infomrations about a game.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Fixture {
    /// The inbase id.
    pub id: u32,
    /// The timestamp the game has been fetched.
    pub timestamp: f64,
    /// The date the game is being played on.
    pub date: DateTime<Utc>,
    /// The venue where it is played.
    pub venue: Venue,
    /// The status of the game.
    pub status: Status,
}

/// A team is a gathering of 11 players.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Team {
    /// Its in base id.
    pub id: u32,
    /// Its name.
    pub name: String,
    /// The logo of the team.
    pub logo: String,
    /// Whether the team is winner of the game it is asssocied when it is 
    /// associed to a game.
    pub winner: Option<bool>,
    /// Odds on this team winning the game it is associed.
    pub odd: Option<f32>,
}

/// The teams associed to a game.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Teams {
    /// The team that hosts the game.
    pub home: Team,
    /// The team that travels to play the game.
    pub away: Team,
}

/// The goals associed to a game.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Goals {
    /// The home team's goals.
    pub home: Option<i16>,
    /// The away team's goals.
    pub away: Option<i16>,
}

/// The score of a game.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Score {
    /// The score after 90 minutes of playtime.
    pub fulltime: Option<Goals>,
    /// The score on extratime if there has been extra time.
    pub extratime: Option<Goals>,
    /// The score after penalties.
    pub penalty: Option<Goals>,
}

/// The country linked to a team, competition or game.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Country {
    /// The name of the country.
    pub name: String,
    /// Its code, in two letters.
    pub code: Option<String>,
    /// It's flag.
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
    pub game_result: crate::bet::GameResult,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Odds {
    pub home: f32,
    pub draw: f32,
    pub away: f32,
}
