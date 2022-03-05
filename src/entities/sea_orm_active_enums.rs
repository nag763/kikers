//! SeaORM Entity. Generated by sea-orm-codegen 0.6.0

use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "result")]
pub enum Result {
    #[sea_orm(string_value = "W")]
    W,
    #[sea_orm(string_value = "A")]
    A,
    #[sea_orm(string_value = "D")]
    D,
    #[sea_orm(string_value = "C")]
    C,
}
