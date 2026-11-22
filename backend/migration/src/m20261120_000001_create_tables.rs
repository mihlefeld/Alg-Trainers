use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.create_table(
            Table::create()
                .table(Users::Table)
                .if_not_exists()
                .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                .col(
                    ColumnDef::new(Users::Email)
                        .string()
                        .not_null()
                        .unique_key(),
                )
                .col(ColumnDef::new(Users::WcaId).string())
                .col(
                    ColumnDef::new(Users::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;

        db.create_table(
            Table::create()
                .table(Settings::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Settings::UserId)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Settings::SettingsJson)
                        .json_binary()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Settings::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;

        db.create_table(
            Table::create()
                .table(Trainers::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Trainers::TrainerId)
                        .string()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(Trainers::Puzzle).string().not_null())
                .col(ColumnDef::new(Trainers::Method).string().not_null())
                .col(ColumnDef::new(Trainers::Name).string().not_null())
                .to_owned(),
        )
        .await?;

        db.create_table(
            Table::create()
                .table(Cases::Table)
                .if_not_exists()
                .col(ColumnDef::new(Cases::TrainerId).string().not_null())
                .col(ColumnDef::new(Cases::CaseId).string().not_null())
                .col(ColumnDef::new(Cases::Name).string().not_null())
                .col(ColumnDef::new(Cases::CaseSetup).text())
                .primary_key(Index::create().col(Cases::TrainerId).col(Cases::CaseId))
                .to_owned(),
        )
        .await?;

        db.create_table(
            Table::create()
                .table(Results::Table)
                .if_not_exists()
                .col(ColumnDef::new(Results::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Results::UserId).uuid().not_null())
                .col(
                    ColumnDef::new(Results::Datetime)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .col(ColumnDef::new(Results::ResultMs).big_integer().not_null())
                .col(ColumnDef::new(Results::TrainerId).string().not_null())
                .col(ColumnDef::new(Results::CaseId).string().not_null())
                .col(
                    ColumnDef::new(Results::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Results::Deleted)
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .to_owned(),
        )
        .await?;

        db.create_table(
            Table::create()
                .table(UserAlgs::Table)
                .if_not_exists()
                .col(ColumnDef::new(UserAlgs::UserId).uuid().not_null())
                .col(ColumnDef::new(UserAlgs::TrainerId).string().not_null())
                .col(ColumnDef::new(UserAlgs::CaseId).string().not_null())
                .col(ColumnDef::new(UserAlgs::Alg).text().not_null())
                .col(
                    ColumnDef::new(UserAlgs::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .primary_key(
                    Index::create()
                        .col(UserAlgs::UserId)
                        .col(UserAlgs::TrainerId)
                        .col(UserAlgs::CaseId),
                )
                .to_owned(),
        )
        .await?;

        db.create_table(
            Table::create()
                .table(SyncRevisions::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(SyncRevisions::Id)
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(SyncRevisions::UserId).uuid().not_null())
                .col(ColumnDef::new(SyncRevisions::Kind).string().not_null())
                .col(ColumnDef::new(SyncRevisions::EntityId).string().not_null())
                .col(
                    ColumnDef::new(SyncRevisions::Payload)
                        .json_binary()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(SyncRevisions::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;
        db.create_index(
            Index::create()
                .name("idx_sync_revisions_user_kind_entity")
                .table(SyncRevisions::Table)
                .col(SyncRevisions::UserId)
                .col(SyncRevisions::Kind)
                .col(SyncRevisions::EntityId)
                .unique()
                .to_owned(),
        )
        .await?;
        Ok(())
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.drop_table(
            Table::drop()
                .table(SyncRevisions::Table)
                .if_exists()
                .to_owned(),
        )
        .await?;
        db.drop_table(Table::drop().table(UserAlgs::Table).if_exists().to_owned())
            .await?;
        db.drop_table(Table::drop().table(Results::Table).if_exists().to_owned())
            .await?;
        db.drop_table(Table::drop().table(Cases::Table).if_exists().to_owned())
            .await?;
        db.drop_table(Table::drop().table(Trainers::Table).if_exists().to_owned())
            .await?;
        db.drop_table(Table::drop().table(Settings::Table).if_exists().to_owned())
            .await?;
        db.drop_table(Table::drop().table(Users::Table).if_exists().to_owned())
            .await?;
        Ok(())
    }
}

macro_rules! table_enum {
    ($name:ident, $table:expr, [$($variant:ident),+]) => {
        #[derive(Iden)]
        enum $name { #[iden=$table] Table, $($variant),+ }
    };
}

table_enum!(Users, "users", [Id, Email, WcaId, CreatedAt]);
table_enum!(Settings, "settings", [UserId, SettingsJson, UpdatedAt]);
table_enum!(Trainers, "trainers", [TrainerId, Puzzle, Method, Name]);
table_enum!(Cases, "cases", [TrainerId, CaseId, Name, CaseSetup]);
table_enum!(
    Results,
    "results",
    [Id, UserId, Datetime, ResultMs, TrainerId, CaseId, UpdatedAt, Deleted]
);
table_enum!(
    UserAlgs,
    "user_algs",
    [UserId, TrainerId, CaseId, Alg, UpdatedAt]
);
table_enum!(
    SyncRevisions,
    "sync_revisions",
    [Id, UserId, Kind, EntityId, Payload, UpdatedAt]
);
