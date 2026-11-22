pub use sea_orm_migration::prelude::*;

mod m20261120_000001_create_tables;
mod m20261120_000002_add_catalog_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261120_000001_create_tables::Migration),
            Box::new(m20261120_000002_add_catalog_tables::Migration),
        ]
    }
}
