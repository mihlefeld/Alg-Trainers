use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DatabaseBackend, DbErr, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Statement,
};
use uuid::Uuid;

use crate::{database::UserTx, entity};

pub type SyncBundle = alg_trainers_shared::SyncBundle<Uuid, DateTime<Utc>>;
pub type SettingsSync = alg_trainers_shared::SettingsSync<DateTime<Utc>>;
pub type SelectionSync = alg_trainers_shared::SelectionSync<DateTime<Utc>>;
pub type UserAlgSync = alg_trainers_shared::UserAlgSync<DateTime<Utc>>;
pub type ResultSync = alg_trainers_shared::ResultSync<Uuid, DateTime<Utc>>;
pub type ResultCursor = alg_trainers_shared::ResultCursor<Uuid, DateTime<Utc>>;

const MAX_SELECTIONS: usize = 100;
const MAX_USER_ALGS: usize = 10_000;
const MAX_RESULTS: usize = 10_000;
const MAX_IDENTIFIER_LENGTH: usize = 256;
const MAX_ALG_LENGTH: usize = 16 * 1024;
const MAX_JSON_BYTES: usize = 256 * 1024;
const RESULTS_PAGE_SIZE: u64 = 1_000;
/// Safety overlap subtracted from `since` on incremental pulls so writes from
/// concurrent transactions (bounded by the 5s `statement_timeout`) that commit
/// after a pull's `NOW()` are never permanently skipped.
const SYNC_WATERMARK_OVERLAP_SECS: i64 = 6;

#[derive(Debug)]
pub struct ValidatedSyncBundle(SyncBundle);

impl TryFrom<SyncBundle> for ValidatedSyncBundle {
    type Error = ValidationError;

    fn try_from(bundle: SyncBundle) -> Result<Self, Self::Error> {
        validate_bundle(&bundle)?;
        Ok(Self(bundle))
    }
}

#[derive(Debug)]
pub enum ValidationError {
    TooMany(&'static str),
    InvalidField(&'static str),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooMany(field) => write!(formatter, "too many {field}"),
            Self::InvalidField(field) => write!(formatter, "invalid {field}"),
        }
    }
}

impl std::error::Error for ValidationError {}

fn validate_bundle(bundle: &SyncBundle) -> Result<(), ValidationError> {
    if bundle.selections.len() > MAX_SELECTIONS {
        return Err(ValidationError::TooMany("selections"));
    }
    if bundle.user_algs.len() > MAX_USER_ALGS {
        return Err(ValidationError::TooMany("custom algorithms"));
    }
    if bundle.results.len() > MAX_RESULTS {
        return Err(ValidationError::TooMany("results"));
    }
    if let Some(settings) = &bundle.settings {
        validate_json(&settings.settings_json, "settings")?;
    }
    for selection in &bundle.selections {
        validate_identifier(&selection.trainer_id, "trainer_id")?;
        validate_json(&selection.selection_json, "selection")?;
    }
    for alg in &bundle.user_algs {
        validate_identifier(&alg.trainer_id, "trainer_id")?;
        validate_identifier(&alg.case_id, "case_id")?;
        validate_identifier(&alg.alg_id, "alg_id")?;
        if alg.alg.len() > MAX_ALG_LENGTH || alg.position < 0 {
            return Err(ValidationError::InvalidField("algorithm"));
        }
    }
    for result in &bundle.results {
        validate_identifier(&result.trainer_id, "trainer_id")?;
        validate_identifier(&result.case_id, "case_id")?;
        if !(0..=86_400_000).contains(&result.result_ms) {
            return Err(ValidationError::InvalidField("result_ms"));
        }
    }
    Ok(())
}

fn validate_identifier(value: &str, field: &'static str) -> Result<(), ValidationError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_LENGTH {
        return Err(ValidationError::InvalidField(field));
    }
    Ok(())
}

fn validate_json(value: &serde_json::Value, field: &'static str) -> Result<(), ValidationError> {
    if serde_json::to_vec(value)
        .map_err(|_| ValidationError::InvalidField(field))?
        .len()
        > MAX_JSON_BYTES
    {
        return Err(ValidationError::InvalidField(field));
    }
    Ok(())
}

pub async fn load_page(
    tx: &UserTx,
    since: Option<DateTime<Utc>>,
    cursor: Option<ResultCursor>,
) -> Result<SyncBundle, DbErr> {
    let user_id = tx.user_id().0;
    let db = tx.conn();

    let server_time = tx.now();
    let effective_since = since.map(|ts| ts - chrono::Duration::seconds(SYNC_WATERMARK_OVERLAP_SECS));
    let is_first_page = cursor.is_none();

    let (settings, selections, user_algs) = if is_first_page {
        let mut settings_query = entity::setting::Entity::find_by_id(user_id);
        if let Some(since) = effective_since {
            settings_query =
                settings_query.filter(entity::setting::Column::ServerUpdatedAt.gte(since));
        }
        let settings = settings_query
            .one(db)
            .await?
            .map(|model| SettingsSync {
                settings_json: model.settings_json,
                updated_at: model.updated_at,
            });

        let mut selection_query = entity::user_selection::Entity::find()
            .filter(entity::user_selection::Column::UserId.eq(user_id));
        if let Some(since) = effective_since {
            selection_query =
                selection_query.filter(entity::user_selection::Column::ServerUpdatedAt.gte(since));
        }
        let selections = selection_query
            .order_by_asc(entity::user_selection::Column::TrainerId)
            .all(db)
            .await?
            .into_iter()
            .map(|model| SelectionSync {
                trainer_id: model.trainer_id,
                selection_json: model.selection_json,
                updated_at: model.updated_at,
            })
            .collect();

        let mut alg_query = entity::user_alg::Entity::find()
            .filter(entity::user_alg::Column::UserId.eq(user_id));
        if let Some(since) = effective_since {
            alg_query = alg_query.filter(entity::user_alg::Column::ServerUpdatedAt.gte(since));
        }
        let user_algs = alg_query
            .order_by_asc(entity::user_alg::Column::TrainerId)
            .order_by_asc(entity::user_alg::Column::CaseId)
            .order_by_asc(entity::user_alg::Column::Position)
            .all(db)
            .await?
            .into_iter()
            .map(|model| UserAlgSync {
                trainer_id: model.trainer_id,
                case_id: model.case_id,
                alg_id: model.alg_id,
                alg: model.alg,
                position: model.position,
                hidden: model.hidden,
                deleted: model.deleted,
                updated_at: model.updated_at,
            })
            .collect();

        (settings, selections, user_algs)
    } else {
        (None, Vec::new(), Vec::new())
    };

    let mut result_query =
        entity::result::Entity::find().filter(entity::result::Column::UserId.eq(user_id));
    if let Some(since) = effective_since {
        result_query = result_query.filter(entity::result::Column::ServerUpdatedAt.gte(since));
    }
    if let Some(cursor) = cursor {
        result_query = result_query.filter(
            Condition::any()
                .add(entity::result::Column::Datetime.lt(cursor.datetime))
                .add(
                    Condition::all()
                        .add(entity::result::Column::Datetime.eq(cursor.datetime))
                        .add(entity::result::Column::Id.lt(cursor.id)),
                ),
        );
    }
    let result_models = result_query
        .order_by_desc(entity::result::Column::Datetime)
        .order_by_desc(entity::result::Column::Id)
        .limit(RESULTS_PAGE_SIZE + 1)
        .all(db)
        .await?;
    let has_more = result_models.len() as u64 > RESULTS_PAGE_SIZE;
    let results: Vec<ResultSync> = result_models
        .into_iter()
        .take(RESULTS_PAGE_SIZE as usize)
        .map(|model| ResultSync {
            id: model.id,
            datetime: model.datetime,
            result_ms: model.result_ms,
            trainer_id: model.trainer_id,
            case_id: model.case_id,
            updated_at: model.updated_at,
            deleted: model.deleted,
        })
        .collect();
    let next_results = has_more.then(|| {
        let last = results
            .last()
            .expect("a non-empty result page must have a cursor");
        ResultCursor {
            datetime: last.datetime,
            id: last.id,
        }
    });

    Ok(SyncBundle {
        settings,
        selections,
        user_algs,
        results,
        next_results,
        server_time: Some(server_time),
    })
}

pub async fn save(tx: &UserTx, incoming: ValidatedSyncBundle) -> Result<SyncBundle, DbErr> {
    let incoming = incoming.0;
    let user_id = tx.user_id().0;
    let db = tx.conn();

    if let Some(settings) = incoming.settings {
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO settings (user_id, settings_json, updated_at, server_updated_at)
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (user_id) DO UPDATE
             SET settings_json = EXCLUDED.settings_json,
                 updated_at = EXCLUDED.updated_at,
                 server_updated_at = NOW()
             WHERE settings.updated_at <= EXCLUDED.updated_at",
            vec![
                user_id.into(),
                settings.settings_json.into(),
                settings.updated_at.into(),
            ],
        ))
        .await?;
    }

    let selections = latest_by_key(
        incoming.selections,
        |s| s.trainer_id.clone(),
        |s| s.updated_at,
    );
    let user_algs = latest_by_key(incoming.user_algs, |a| a.alg_id.clone(), |a| a.updated_at);
    let results = latest_by_key(incoming.results, |r| r.id, |r| r.updated_at);

    if !selections.is_empty() {
        let selections = serde_json::to_value(&selections)
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO user_selections
                (user_id, trainer_id, selection_json, updated_at, server_updated_at)
             SELECT $1, source.trainer_id, source.selection_json, source.updated_at, NOW()
             FROM jsonb_to_recordset($2::jsonb) AS source(
                trainer_id text, selection_json jsonb, updated_at timestamptz
             )
             ON CONFLICT (user_id, trainer_id) DO UPDATE
             SET selection_json = EXCLUDED.selection_json,
                 updated_at = EXCLUDED.updated_at,
                 server_updated_at = NOW()
             WHERE user_selections.user_id = EXCLUDED.user_id
               AND user_selections.updated_at <= EXCLUDED.updated_at",
            vec![user_id.into(), selections.into()],
        ))
        .await?;
    }

    if !user_algs.is_empty() {
        let user_algs = serde_json::to_value(&user_algs)
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO user_algs
                (user_id, trainer_id, case_id, alg_id, alg, position, hidden, deleted, updated_at, server_updated_at)
             SELECT $1, source.trainer_id, source.case_id, source.alg_id, source.alg,
                    source.position, source.hidden, source.deleted, source.updated_at, NOW()
             FROM jsonb_to_recordset($2::jsonb) AS source(
                trainer_id text, case_id text, alg_id text, alg text, position integer,
                hidden boolean, deleted boolean, updated_at timestamptz
             )
             ON CONFLICT (user_id, alg_id) DO UPDATE
             SET trainer_id = EXCLUDED.trainer_id, case_id = EXCLUDED.case_id,
                 alg = EXCLUDED.alg, position = EXCLUDED.position, hidden = EXCLUDED.hidden,
                 deleted = EXCLUDED.deleted, updated_at = EXCLUDED.updated_at,
                 server_updated_at = NOW()
             WHERE user_algs.user_id = EXCLUDED.user_id
               AND user_algs.updated_at <= EXCLUDED.updated_at",
            vec![user_id.into(), user_algs.into()],
        ))
        .await?;
    }

    if !results.is_empty() {
        let results = serde_json::to_value(&results)
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO results
                (id, user_id, datetime, result_ms, trainer_id, case_id, updated_at, deleted, server_updated_at)
             SELECT source.id, $1, source.datetime, source.result_ms, source.trainer_id,
                    source.case_id, source.updated_at, source.deleted, NOW()
             FROM jsonb_to_recordset($2::jsonb) AS source(
                id uuid, datetime timestamptz, result_ms bigint, trainer_id text,
                case_id text, updated_at timestamptz, deleted boolean
             )
             ON CONFLICT (user_id, id) DO UPDATE
             SET datetime = EXCLUDED.datetime, result_ms = EXCLUDED.result_ms,
                 trainer_id = EXCLUDED.trainer_id, case_id = EXCLUDED.case_id,
                 updated_at = EXCLUDED.updated_at, deleted = EXCLUDED.deleted,
                 server_updated_at = NOW()
             WHERE results.user_id = EXCLUDED.user_id
               AND results.updated_at <= EXCLUDED.updated_at",
            vec![user_id.into(), results.into()],
        ))
        .await?;
    }

    Ok(SyncBundle {
        server_time: Some(tx.now()),
        ..SyncBundle::default()
    })
}

/// A single upsert statement cannot touch the same row twice, so keep only the
/// newest entry per key (the last one wins on ties).
fn latest_by_key<T, K: std::hash::Hash + Eq>(
    items: Vec<T>,
    key: impl Fn(&T) -> K,
    updated_at: impl Fn(&T) -> DateTime<Utc>,
) -> Vec<T> {
    let mut latest: std::collections::HashMap<K, T> = std::collections::HashMap::new();
    for item in items {
        match latest.entry(key(&item)) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if updated_at(entry.get()) <= updated_at(&item) {
                    entry.insert(item);
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(item);
            }
        }
    }
    latest.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unbounded_result_batches() {
        let timestamp = Utc::now();
        let result = ResultSync {
            id: Uuid::new_v4(),
            datetime: timestamp,
            result_ms: 1,
            trainer_id: "trainer".to_owned(),
            case_id: "case".to_owned(),
            updated_at: timestamp,
            deleted: false,
        };
        let bundle = SyncBundle {
            results: vec![result; MAX_RESULTS + 1],
            ..SyncBundle::default()
        };
        assert!(matches!(
            ValidatedSyncBundle::try_from(bundle),
            Err(ValidationError::TooMany("results"))
        ));
    }

    #[test]
    fn rejects_oversized_algorithms() {
        let bundle = SyncBundle {
            user_algs: vec![UserAlgSync {
                trainer_id: "trainer".to_owned(),
                case_id: "case".to_owned(),
                alg_id: "alg".to_owned(),
                alg: "R".repeat(MAX_ALG_LENGTH + 1),
                position: 0,
                hidden: false,
                deleted: false,
                updated_at: Utc::now(),
            }],
            ..SyncBundle::default()
        };
        assert!(matches!(
            ValidatedSyncBundle::try_from(bundle),
            Err(ValidationError::InvalidField("algorithm"))
        ));
    }
}
