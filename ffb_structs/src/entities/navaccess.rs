use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: i32,
    pub label: String,
    pub href: String,
    pub position: Option<i32>,
}
