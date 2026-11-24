use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scrambles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub trainer_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub case_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub position: i32,
    pub scramble: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
