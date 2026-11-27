use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(unique)]
    pub wca_user_id: Option<i64>,
    pub wca_id: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::setting::Entity")]
    Setting,
    #[sea_orm(has_many = "super::result::Entity")]
    Result,
    #[sea_orm(has_many = "super::user_alg::Entity")]
    UserAlg,
    #[sea_orm(has_many = "super::user_selection::Entity")]
    UserSelection,
    #[sea_orm(has_many = "super::sync_revision::Entity")]
    SyncRevision,
}

impl Related<super::setting::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Setting.def()
    }
}

impl Related<super::result::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Result.def()
    }
}

impl Related<super::user_alg::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAlg.def()
    }
}

impl Related<super::user_selection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserSelection.def()
    }
}

impl Related<super::sync_revision::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SyncRevision.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
