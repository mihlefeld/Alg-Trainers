use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.get_connection()
            .execute_unprepared(
                r#"
                REVOKE DELETE ON
                    users, settings, results, user_algs, user_selections
                    FROM alg_trainers_app;
                REVOKE ALL PRIVILEGES ON sync_revisions FROM alg_trainers_app;

                UPDATE settings SET server_updated_at = NOW() WHERE server_updated_at > NOW();
                UPDATE user_algs SET server_updated_at = NOW() WHERE server_updated_at > NOW();
                UPDATE user_selections SET server_updated_at = NOW() WHERE server_updated_at > NOW();
                UPDATE results SET server_updated_at = NOW() WHERE server_updated_at > NOW();

                DROP INDEX IF EXISTS idx_results_user_server_updated_at;
                CREATE INDEX IF NOT EXISTS idx_results_user_server_updated_at
                    ON results (user_id, server_updated_at DESC);
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.get_connection()
            .execute_unprepared(
                r#"
                GRANT DELETE ON
                    users, settings, results, user_algs, user_selections
                    TO alg_trainers_app;
                GRANT SELECT, INSERT, UPDATE, DELETE ON sync_revisions TO alg_trainers_app;
                "#,
            )
            .await?;
        Ok(())
    }
}
