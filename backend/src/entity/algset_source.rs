use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "algset_sources")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub algset_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub name: String,
    pub person: Option<String>,
    pub link: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
