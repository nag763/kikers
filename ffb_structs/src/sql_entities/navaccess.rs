//! SeaORM Entity. Generated by sea-orm-codegen 0.6.0

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "NAVACCESS")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub label: String,
    pub href: String,
    #[sea_orm(unique)]
    pub position: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::role_navaccess::Entity")]
    RoleNavaccess,
}

impl Related<super::role_navaccess::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RoleNavaccess.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
