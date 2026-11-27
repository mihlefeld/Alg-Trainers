pub use sea_orm_migration::prelude::*;

mod m20261120_000001_create_tables;
mod m20261120_000002_add_catalog_tables;
mod m20261123_000001_add_max_algs_per_row;
mod m20261123_000002_add_alg_ids;
mod m20261123_000003_harden_indexes_and_ownership;
mod m20261127_000001_add_user_selections_and_sync_timestamps;
mod m20261127_000002_wca_user_id_and_rls;
mod m20261127_000003_tighten_rls_grants_and_indexes;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261120_000001_create_tables::Migration),
            Box::new(m20261120_000002_add_catalog_tables::Migration),
            Box::new(m20261123_000001_add_max_algs_per_row::Migration),
            Box::new(m20261123_000002_add_alg_ids::Migration),
            Box::new(m20261123_000003_harden_indexes_and_ownership::Migration),
            Box::new(m20261127_000001_add_user_selections_and_sync_timestamps::Migration),
            Box::new(m20261127_000002_wca_user_id_and_rls::Migration),
            Box::new(m20261127_000003_tighten_rls_grants_and_indexes::Migration),
        ]
    }
}
