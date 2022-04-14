use chrono::DateTime;

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub login: String,
    pub password: String,
    pub is_authorized: i8,
    pub role: i32,
    pub joined_on: DateTime<chrono::offset::Utc>,
}
