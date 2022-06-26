#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, Eq, Hash, sqlx::Type)]
#[repr(u32)]
pub enum GameResult {
    Win = 1,
    Draw = 2,
    Loss = 3,
}
