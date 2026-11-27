use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE users
                    ADD COLUMN IF NOT EXISTS wca_user_id BIGINT;
                CREATE UNIQUE INDEX IF NOT EXISTS idx_users_wca_user_id
                    ON users (wca_user_id)
                    WHERE wca_user_id IS NOT NULL;

                DO $$
                BEGIN
                    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'alg_trainers_app') THEN
                        CREATE ROLE alg_trainers_app NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS;
                    ELSE
                        ALTER ROLE alg_trainers_app NOSUPERUSER NOCREATEDB NOCREATEROLE NOBYPASSRLS;
                    END IF;
                END
                $$;

                GRANT USAGE ON SCHEMA public TO alg_trainers_app;
                GRANT SELECT ON
                    trainers, cases, algs, scrambles, groups, group_assignments, algsets, algset_assignments
                    TO alg_trainers_app;
                GRANT SELECT, INSERT, UPDATE ON
                    users, settings, results, user_algs, user_selections
                    TO alg_trainers_app;
                GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO alg_trainers_app;

                ALTER TABLE settings ENABLE ROW LEVEL SECURITY;
                ALTER TABLE settings FORCE ROW LEVEL SECURITY;
                DROP POLICY IF EXISTS settings_user_isolation ON settings;
                CREATE POLICY settings_user_isolation ON settings
                    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid)
                    WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

                ALTER TABLE results ENABLE ROW LEVEL SECURITY;
                ALTER TABLE results FORCE ROW LEVEL SECURITY;
                DROP POLICY IF EXISTS results_user_isolation ON results;
                CREATE POLICY results_user_isolation ON results
                    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid)
                    WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

                ALTER TABLE user_algs ENABLE ROW LEVEL SECURITY;
                ALTER TABLE user_algs FORCE ROW LEVEL SECURITY;
                DROP POLICY IF EXISTS user_algs_user_isolation ON user_algs;
                CREATE POLICY user_algs_user_isolation ON user_algs
                    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid)
                    WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

                ALTER TABLE user_selections ENABLE ROW LEVEL SECURITY;
                ALTER TABLE user_selections FORCE ROW LEVEL SECURITY;
                DROP POLICY IF EXISTS user_selections_user_isolation ON user_selections;
                CREATE POLICY user_selections_user_isolation ON user_selections
                    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid)
                    WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

                ALTER TABLE sync_revisions ENABLE ROW LEVEL SECURITY;
                ALTER TABLE sync_revisions FORCE ROW LEVEL SECURITY;
                DROP POLICY IF EXISTS sync_revisions_user_isolation ON sync_revisions;
                CREATE POLICY sync_revisions_user_isolation ON sync_revisions
                    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid)
                    WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.get_connection()
            .execute_unprepared(
                r#"
                DROP POLICY IF EXISTS sync_revisions_user_isolation ON sync_revisions;
                ALTER TABLE sync_revisions NO FORCE ROW LEVEL SECURITY;
                ALTER TABLE sync_revisions DISABLE ROW LEVEL SECURITY;

                DROP POLICY IF EXISTS user_selections_user_isolation ON user_selections;
                ALTER TABLE user_selections NO FORCE ROW LEVEL SECURITY;
                ALTER TABLE user_selections DISABLE ROW LEVEL SECURITY;

                DROP POLICY IF EXISTS user_algs_user_isolation ON user_algs;
                ALTER TABLE user_algs NO FORCE ROW LEVEL SECURITY;
                ALTER TABLE user_algs DISABLE ROW LEVEL SECURITY;

                DROP POLICY IF EXISTS results_user_isolation ON results;
                ALTER TABLE results NO FORCE ROW LEVEL SECURITY;
                ALTER TABLE results DISABLE ROW LEVEL SECURITY;

                DROP POLICY IF EXISTS settings_user_isolation ON settings;
                ALTER TABLE settings NO FORCE ROW LEVEL SECURITY;
                ALTER TABLE settings DISABLE ROW LEVEL SECURITY;

                DROP INDEX IF EXISTS idx_users_wca_user_id;
                ALTER TABLE users DROP COLUMN IF EXISTS wca_user_id;
                "#,
            )
            .await?;
        Ok(())
    }
}
