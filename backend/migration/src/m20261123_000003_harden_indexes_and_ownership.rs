use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE results DROP CONSTRAINT results_pkey;
                ALTER TABLE results ADD CONSTRAINT results_pkey PRIMARY KEY (user_id, id);

                CREATE INDEX IF NOT EXISTS idx_results_user_datetime
                    ON results (user_id, datetime DESC, id DESC);
                CREATE INDEX IF NOT EXISTS idx_algs_trainer_case_position
                    ON algs (trainer_id, case_id, position);
                CREATE INDEX IF NOT EXISTS idx_sync_revisions_user_updated_at
                    ON sync_revisions (user_id, updated_at);

                ALTER TABLE settings
                    ADD CONSTRAINT fk_settings_user
                    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
                ALTER TABLE results
                    ADD CONSTRAINT fk_results_user
                    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
                ALTER TABLE user_algs
                    ADD CONSTRAINT fk_user_algs_user
                    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
                ALTER TABLE sync_revisions
                    ADD CONSTRAINT fk_sync_revisions_user
                    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE sync_revisions DROP CONSTRAINT IF EXISTS fk_sync_revisions_user;
                ALTER TABLE user_algs DROP CONSTRAINT IF EXISTS fk_user_algs_user;
                ALTER TABLE results DROP CONSTRAINT IF EXISTS fk_results_user;
                ALTER TABLE settings DROP CONSTRAINT IF EXISTS fk_settings_user;

                DROP INDEX IF EXISTS idx_sync_revisions_user_updated_at;
                DROP INDEX IF EXISTS idx_algs_trainer_case_position;
                DROP INDEX IF EXISTS idx_results_user_datetime;

                ALTER TABLE results DROP CONSTRAINT results_pkey;
                ALTER TABLE results ADD CONSTRAINT results_pkey PRIMARY KEY (id);
                "#,
            )
            .await?;
        Ok(())
    }
}
