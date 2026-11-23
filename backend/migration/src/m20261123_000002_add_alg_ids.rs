use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE algs ADD COLUMN alg_id TEXT;
                UPDATE algs
                SET alg_id = 'builtin:' || trainer_id || ':' || case_id || ':' || position;
                ALTER TABLE algs ALTER COLUMN alg_id SET NOT NULL;
                ALTER TABLE algs DROP CONSTRAINT algs_pkey;
                ALTER TABLE algs ADD CONSTRAINT algs_pkey PRIMARY KEY (alg_id);

                ALTER TABLE user_algs ADD COLUMN alg_id TEXT;
                ALTER TABLE user_algs ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
                ALTER TABLE user_algs ADD COLUMN hidden BOOLEAN NOT NULL DEFAULT FALSE;
                ALTER TABLE user_algs ADD COLUMN deleted BOOLEAN NOT NULL DEFAULT FALSE;
                UPDATE user_algs
                SET alg_id = 'legacy:' || user_id || ':' || trainer_id || ':' || case_id;
                ALTER TABLE user_algs ALTER COLUMN alg_id SET NOT NULL;
                ALTER TABLE user_algs DROP CONSTRAINT user_algs_pkey;
                ALTER TABLE user_algs ADD CONSTRAINT user_algs_pkey PRIMARY KEY (user_id, alg_id);
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, db: &SchemaManager) -> Result<(), DbErr> {
        db.get_connection()
            .execute_unprepared(
                r#"
                DELETE FROM user_algs older
                USING user_algs newer
                WHERE older.user_id = newer.user_id
                  AND older.trainer_id = newer.trainer_id
                  AND older.case_id = newer.case_id
                  AND older.position > newer.position;
                ALTER TABLE user_algs DROP CONSTRAINT user_algs_pkey;
                ALTER TABLE user_algs ADD CONSTRAINT user_algs_pkey
                    PRIMARY KEY (user_id, trainer_id, case_id);
                ALTER TABLE user_algs DROP COLUMN alg_id;
                ALTER TABLE user_algs DROP COLUMN position;
                ALTER TABLE user_algs DROP COLUMN hidden;
                ALTER TABLE user_algs DROP COLUMN deleted;

                ALTER TABLE algs DROP CONSTRAINT algs_pkey;
                ALTER TABLE algs ADD CONSTRAINT algs_pkey
                    PRIMARY KEY (trainer_id, case_id, position);
                ALTER TABLE algs DROP COLUMN alg_id;
                "#,
            )
            .await?;
        Ok(())
    }
}
