use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "algs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub alg_id: String,
    pub trainer_id: String,
    pub case_id: String,
    pub alg: String,
    pub position: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
