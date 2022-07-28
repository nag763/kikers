//! The scoreboard entry is a MySQL row containing the user, its name,
//! and other important informations when displaying a scoreboard.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    /// The MySQL user's id.
    pub user_id: u32,
    /// The user name.
    pub user_name: String,
    /// The points won during the season.
    pub points: bigdecimal::BigDecimal,
    /// The numbers of bets made.
    pub bets_made: i64,
    /// The number of points per bets.
    pub ppb: bigdecimal::BigDecimal,
}
