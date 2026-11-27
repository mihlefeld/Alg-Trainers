use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE IF NOT EXISTS user_selections (
                    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    trainer_id TEXT NOT NULL,
                    selection_json JSONB NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL,
                    server_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    PRIMARY KEY (user_id, trainer_id)
                );

                INSERT INTO user_selections (user_id, trainer_id, selection_json, updated_at, server_updated_at)
                SELECT
                    user_id,
                    COALESCE(NULLIF(payload->>'trainer_id', ''), entity_id),
                    COALESCE(payload->'selection_json', '{}'::jsonb),
                    updated_at,
                    updated_at
                FROM sync_revisions
                WHERE kind = 'selection'
                ON CONFLICT (user_id, trainer_id) DO UPDATE
                SET selection_json = EXCLUDED.selection_json,
                    updated_at = EXCLUDED.updated_at,
                    server_updated_at = GREATEST(user_selections.server_updated_at, EXCLUDED.server_updated_at)
                WHERE user_selections.updated_at <= EXCLUDED.updated_at;

                ALTER TABLE settings
                    ADD COLUMN IF NOT EXISTS server_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

                ALTER TABLE user_algs
                    ADD COLUMN IF NOT EXISTS server_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

                ALTER TABLE results
                    ADD COLUMN IF NOT EXISTS server_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

                CREATE INDEX IF NOT EXISTS idx_user_selections_user_server_updated_at
                    ON user_selections (user_id, server_updated_at);
                CREATE INDEX IF NOT EXISTS idx_user_algs_user_server_updated_at
                    ON user_algs (user_id, server_updated_at);
                CREATE INDEX IF NOT EXISTS idx_results_user_server_updated_at
                    ON results (user_id, server_updated_at DESC);

                ALTER TABLE trainers
                    ADD COLUMN IF NOT EXISTS selection_key TEXT,
                    ADD COLUMN IF NOT EXISTS times_key TEXT,
                    ADD COLUMN IF NOT EXISTS u_move_cycles JSONB NOT NULL DEFAULT '[]'::jsonb;
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE trainers
                    DROP COLUMN IF EXISTS u_move_cycles,
                    DROP COLUMN IF EXISTS times_key,
                    DROP COLUMN IF EXISTS selection_key;

                DROP INDEX IF EXISTS idx_results_user_server_updated_at;
                DROP INDEX IF EXISTS idx_user_algs_user_server_updated_at;
                DROP INDEX IF EXISTS idx_user_selections_user_server_updated_at;

                ALTER TABLE results DROP COLUMN IF EXISTS server_updated_at;
                ALTER TABLE user_algs DROP COLUMN IF EXISTS server_updated_at;
                ALTER TABLE settings DROP COLUMN IF EXISTS server_updated_at;

                INSERT INTO sync_revisions (user_id, kind, entity_id, payload, updated_at)
                SELECT
                    user_id,
                    'selection',
                    trainer_id,
                    jsonb_build_object('trainer_id', trainer_id, 'selection_json', selection_json),
                    updated_at
                FROM user_selections
                ON CONFLICT (user_id, kind, entity_id) DO UPDATE
                SET payload = EXCLUDED.payload,
                    updated_at = EXCLUDED.updated_at
                WHERE sync_revisions.updated_at <= EXCLUDED.updated_at;

                DROP TABLE IF EXISTS user_selections;
                "#,
            )
            .await?;
        Ok(())
    }
}
