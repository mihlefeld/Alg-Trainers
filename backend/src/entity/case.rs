use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cases")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub trainer_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub case_id: String,
    pub name: String,
    pub case_setup: Option<String>,
    pub image_svg: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
