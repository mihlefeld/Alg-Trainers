use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "algset_assignments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub algset_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub trainer_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub group_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
