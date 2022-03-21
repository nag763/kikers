use crate::api_entities::status::Status;
use crate::api_entities::venue::Venue;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Fixture {
    pub id: i64,
    pub referee: Option<String>,
    pub timezone: String,
    pub date: DateTime<Utc>,
    pub venue: Venue,
    pub status: Status,
}
