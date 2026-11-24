use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "trainers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub trainer_id: String,
    pub puzzle: String,
    pub method: String,
    pub name: String,
    #[sea_orm(column_name = "order")]
    pub order: i32,
    pub icon_path: Option<String>,
    pub title: Option<String>,
    pub pre_rotations: Json,
    pub post_rotations: Json,
    pub pre_moves: Json,
    pub post_moves: Json,
    pub selected_algsets: Json,
    pub max_algs_per_row: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
