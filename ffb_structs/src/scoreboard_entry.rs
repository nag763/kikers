use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    pub user_id: u32,
    pub user_name: String,
    pub points: bigdecimal::BigDecimal,
    pub bets_made: i64,
    pub ppb: bigdecimal::BigDecimal,
}
