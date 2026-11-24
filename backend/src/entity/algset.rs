use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "algsets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub trainer_id: String,
    pub name: String,
    #[sea_orm(column_name = "order")]
    pub order: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
