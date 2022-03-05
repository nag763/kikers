//! SeaORM Entity. Generated by sea-orm-codegen 0.6.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "EDITION")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub competition_id: i32,
    pub year_begin: i32,
    pub year_end: Option<i32>,
    pub winner_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::competition::Entity",
        from = "Column::CompetitionId",
        to = "super::competition::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Competition,
    #[sea_orm(
        belongs_to = "super::club::Entity",
        from = "Column::WinnerId",
        to = "super::club::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Club,
    #[sea_orm(has_many = "super::game::Entity")]
    Game,
}

impl Related<super::competition::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Competition.def()
    }
}

impl Related<super::club::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Club.def()
    }
}

impl Related<super::game::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Game.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
