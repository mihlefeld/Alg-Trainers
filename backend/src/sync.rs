use chrono::{DateTime, Duration, Utc};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SyncBundle {
    pub settings: Option<SettingsSync>,
    pub selections: Vec<SelectionSync>,
    pub user_algs: Vec<UserAlgSync>,
    pub results: Vec<ResultSync>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SettingsSync {
    pub settings_json: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectionSync {
    pub trainer_id: String,
    #[serde(default)]
    pub selection_json: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserAlgSync {
    pub trainer_id: String,
    pub case_id: String,
    pub alg: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResultSync {
    pub id: Uuid,
    pub datetime: DateTime<Utc>,
    pub result_ms: i64,
    pub trainer_id: String,
    pub case_id: String,
    pub updated_at: DateTime<Utc>,
    pub deleted: bool,
}

pub async fn load(db: &DatabaseConnection, user_id: Uuid) -> Result<SyncBundle, sea_orm::DbErr> {
    let settings_row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT settings_json, updated_at FROM settings WHERE user_id = $1",
            vec![user_id.into()],
        ))
        .await?;
    let settings = settings_row.map(|row| SettingsSync {
        settings_json: row
            .try_get("", "settings_json")
            .unwrap_or(serde_json::json!({})),
        updated_at: row.try_get("", "updated_at").unwrap_or_else(|_| Utc::now()),
    });

    let selection_rows = db.query_all(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT payload, updated_at FROM sync_revisions WHERE user_id = $1 AND kind = 'selection'",
        vec![user_id.into()],
    )).await?;
    let selections = selection_rows
        .into_iter()
        .filter_map(|row| {
            let value: serde_json::Value = row.try_get("", "payload").ok()?;
            Some(SelectionSync {
                trainer_id: value.get("trainer_id")?.as_str()?.to_owned(),
                selection_json: value.get("selection_json")?.clone(),
                updated_at: row.try_get("", "updated_at").ok()?,
            })
        })
        .collect();

    let alg_rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT trainer_id, case_id, alg, updated_at FROM user_algs WHERE user_id = $1",
            vec![user_id.into()],
        ))
        .await?;
    let user_algs = alg_rows
        .into_iter()
        .filter_map(|row| {
            Some(UserAlgSync {
                trainer_id: row.try_get("", "trainer_id").ok()?,
                case_id: row.try_get("", "case_id").ok()?,
                alg: row.try_get("", "alg").ok()?,
                updated_at: row.try_get("", "updated_at").ok()?,
            })
        })
        .collect();

    let result_rows = db.query_all(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT id, datetime, result_ms, trainer_id, case_id, updated_at, deleted FROM results WHERE user_id = $1 ORDER BY datetime DESC",
        vec![user_id.into()],
    )).await?;
    let results = result_rows
        .into_iter()
        .filter_map(|row| {
            Some(ResultSync {
                id: row.try_get("", "id").ok()?,
                datetime: row.try_get("", "datetime").ok()?,
                result_ms: row.try_get("", "result_ms").ok()?,
                trainer_id: row.try_get("", "trainer_id").ok()?,
                case_id: row.try_get("", "case_id").ok()?,
                updated_at: row.try_get("", "updated_at").ok()?,
                deleted: row.try_get("", "deleted").unwrap_or(false),
            })
        })
        .collect();

    Ok(SyncBundle {
        settings,
        selections,
        user_algs,
        results,
    })
}

pub async fn save(
    db: &DatabaseConnection,
    user_id: Uuid,
    incoming: SyncBundle,
) -> Result<SyncBundle, sea_orm::DbErr> {
    if let Some(settings) = incoming.settings {
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO settings (user_id, settings_json, updated_at) VALUES ($1, $2, $3)
             ON CONFLICT (user_id) DO UPDATE SET settings_json = EXCLUDED.settings_json, updated_at = EXCLUDED.updated_at
             WHERE settings.updated_at <= EXCLUDED.updated_at",
            vec![user_id.into(), settings.settings_json.into(), settings.updated_at.into()],
        )).await?;
    }

    for selection in incoming.selections {
        let payload = serde_json::json!({
            "trainer_id": selection.trainer_id,
            "selection_json": selection.selection_json
        });
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO sync_revisions (user_id, kind, entity_id, payload, updated_at)
             VALUES ($1, 'selection', $2, $3, $4)
             ON CONFLICT (user_id, kind, entity_id) DO UPDATE
             SET payload = EXCLUDED.payload, updated_at = EXCLUDED.updated_at
             WHERE sync_revisions.updated_at <= EXCLUDED.updated_at",
            vec![
                user_id.into(),
                selection.trainer_id.into(),
                payload.into(),
                selection.updated_at.into(),
            ],
        ))
        .await?;
    }

    for alg in incoming.user_algs {
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO user_algs (user_id, trainer_id, case_id, alg, updated_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (user_id, trainer_id, case_id) DO UPDATE
             SET alg = EXCLUDED.alg, updated_at = EXCLUDED.updated_at
             WHERE user_algs.updated_at <= EXCLUDED.updated_at",
            vec![
                user_id.into(),
                alg.trainer_id.into(),
                alg.case_id.into(),
                alg.alg.into(),
                alg.updated_at.into(),
            ],
        ))
        .await?;
    }

    for result in incoming.results {
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO results (id, user_id, datetime, result_ms, trainer_id, case_id, updated_at, deleted)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (id) DO UPDATE SET datetime = EXCLUDED.datetime, result_ms = EXCLUDED.result_ms,
             trainer_id = EXCLUDED.trainer_id, case_id = EXCLUDED.case_id, updated_at = EXCLUDED.updated_at,
             deleted = EXCLUDED.deleted WHERE results.updated_at <= EXCLUDED.updated_at",
            vec![result.id.into(), user_id.into(), result.datetime.into(), result.result_ms.into(),
                 result.trainer_id.into(), result.case_id.into(), result.updated_at.into(), result.deleted.into()],
        )).await?;
    }

    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "DELETE FROM sync_revisions WHERE updated_at < $1",
        vec![(Utc::now() - Duration::days(30)).into()],
    ))
    .await?;
    load(db, user_id).await
}
