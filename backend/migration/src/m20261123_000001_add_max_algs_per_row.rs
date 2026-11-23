use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.alter_table(
            Table::alter()
                .table(Trainers::Table)
                .add_column(
                    ColumnDef::new(Trainers::MaxAlgsPerRow)
                        .integer()
                        .not_null()
                        .default(8),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.alter_table(
            Table::alter()
                .table(Trainers::Table)
                .drop_column(Trainers::MaxAlgsPerRow)
                .to_owned(),
        )
        .await
    }
}

#[derive(DeriveIden)]
enum Trainers {
    Table,
    MaxAlgsPerRow,
}
