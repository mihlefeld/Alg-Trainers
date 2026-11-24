use chrono::{DateTime, Duration, Utc};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    DatabaseTransaction, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Statement,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity;

const MAX_SELECTIONS: usize = 100;
const MAX_USER_ALGS: usize = 10_000;
const MAX_RESULTS: usize = 10_000;
const MAX_IDENTIFIER_LENGTH: usize = 256;
const MAX_ALG_LENGTH: usize = 16 * 1024;
const MAX_JSON_BYTES: usize = 256 * 1024;
const RESULTS_PAGE_SIZE: u64 = 1_000;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SyncBundle {
    pub settings: Option<SettingsSync>,
    pub selections: Vec<SelectionSync>,
    pub user_algs: Vec<UserAlgSync>,
    pub results: Vec<ResultSync>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_results: Option<ResultCursor>,
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
    pub alg_id: String,
    pub alg: String,
    pub position: i32,
    pub hidden: bool,
    pub deleted: bool,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResultCursor {
    pub datetime: DateTime<Utc>,
    pub id: Uuid,
}

#[derive(Deserialize)]
struct SelectionPayload {
    trainer_id: String,
    selection_json: serde_json::Value,
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

impl SyncBundle {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.selections.len() > MAX_SELECTIONS {
            return Err(ValidationError::TooMany("selections"));
        }
        if self.user_algs.len() > MAX_USER_ALGS {
            return Err(ValidationError::TooMany("custom algorithms"));
        }
        if self.results.len() > MAX_RESULTS {
            return Err(ValidationError::TooMany("results"));
        }
        if let Some(settings) = &self.settings {
            validate_json(&settings.settings_json, "settings")?;
        }
        for selection in &self.selections {
            validate_identifier(&selection.trainer_id, "trainer_id")?;
            validate_json(&selection.selection_json, "selection")?;
        }
        for alg in &self.user_algs {
            validate_identifier(&alg.trainer_id, "trainer_id")?;
            validate_identifier(&alg.case_id, "case_id")?;
            validate_identifier(&alg.alg_id, "alg_id")?;
            if alg.alg.len() > MAX_ALG_LENGTH || alg.position < 0 {
                return Err(ValidationError::InvalidField("algorithm"));
            }
        }
        for result in &self.results {
            validate_identifier(&result.trainer_id, "trainer_id")?;
            validate_identifier(&result.case_id, "case_id")?;
            if !(0..=86_400_000).contains(&result.result_ms) {
                return Err(ValidationError::InvalidField("result_ms"));
            }
        }
        Ok(())
    }
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

pub async fn load(db: &DatabaseConnection, user_id: Uuid) -> Result<SyncBundle, DbErr> {
    load_page(db, user_id, None).await
}

pub async fn load_page(
    db: &DatabaseConnection,
    user_id: Uuid,
    cursor: Option<ResultCursor>,
) -> Result<SyncBundle, DbErr> {
    let settings_model = entity::setting::Entity::find_by_id(user_id).one(db).await?;
    let settings = settings_model.map(|model| SettingsSync {
        settings_json: model.settings_json,
        updated_at: model.updated_at,
    });

    let selection_models = entity::sync_revision::Entity::find()
        .filter(entity::sync_revision::Column::UserId.eq(user_id))
        .filter(entity::sync_revision::Column::Kind.eq("selection"))
        .order_by_asc(entity::sync_revision::Column::EntityId)
        .all(db)
        .await?;
    let selections = selection_models
        .into_iter()
        .map(|model| {
            let payload: SelectionPayload = serde_json::from_value(model.payload)
                .map_err(|error| DbErr::Custom(format!("invalid selection payload: {error}")))?;
            Ok(SelectionSync {
                trainer_id: payload.trainer_id,
                selection_json: payload.selection_json,
                updated_at: model.updated_at,
            })
        })
        .collect::<Result<Vec<_>, DbErr>>()?;

    let alg_models = entity::user_alg::Entity::find()
        .filter(entity::user_alg::Column::UserId.eq(user_id))
        .order_by_asc(entity::user_alg::Column::TrainerId)
        .order_by_asc(entity::user_alg::Column::CaseId)
        .order_by_asc(entity::user_alg::Column::Position)
        .all(db)
        .await?;
    let user_algs = alg_models
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

    let mut result_query =
        entity::result::Entity::find().filter(entity::result::Column::UserId.eq(user_id));
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
    };
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
    })
}

pub async fn save(
    db: &DatabaseConnection,
    user_id: Uuid,
    incoming: SyncBundle,
) -> Result<SyncBundle, SaveError> {
    incoming.validate().map_err(SaveError::Validation)?;
    let transaction = db.begin().await.map_err(SaveError::Database)?;
    save_in_transaction(&transaction, user_id, incoming)
        .await
        .map_err(SaveError::Database)?;
    transaction.commit().await.map_err(SaveError::Database)?;
    load(db, user_id).await.map_err(SaveError::Database)
}

async fn save_in_transaction(
    db: &DatabaseTransaction,
    user_id: Uuid,
    incoming: SyncBundle,
) -> Result<(), DbErr> {
    db.execute_unprepared("SET LOCAL statement_timeout = '5s'")
        .await?;

    if let Some(settings) = incoming.settings {
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO settings (user_id, settings_json, updated_at) VALUES ($1, $2, $3)
             ON CONFLICT (user_id) DO UPDATE
             SET settings_json = EXCLUDED.settings_json, updated_at = EXCLUDED.updated_at
             WHERE settings.updated_at <= EXCLUDED.updated_at",
            vec![
                user_id.into(),
                settings.settings_json.into(),
                settings.updated_at.into(),
            ],
        ))
        .await?;
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

    if !incoming.user_algs.is_empty() {
        let user_algs = serde_json::to_value(&incoming.user_algs)
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO user_algs
                (user_id, trainer_id, case_id, alg_id, alg, position, hidden, deleted, updated_at)
             SELECT $1, source.trainer_id, source.case_id, source.alg_id, source.alg,
                    source.position, source.hidden, source.deleted, source.updated_at
             FROM jsonb_to_recordset($2::jsonb) AS source(
                trainer_id text, case_id text, alg_id text, alg text, position integer,
                hidden boolean, deleted boolean, updated_at timestamptz
             )
             ON CONFLICT (user_id, alg_id) DO UPDATE
             SET trainer_id = EXCLUDED.trainer_id, case_id = EXCLUDED.case_id,
                 alg = EXCLUDED.alg, position = EXCLUDED.position, hidden = EXCLUDED.hidden,
                 deleted = EXCLUDED.deleted, updated_at = EXCLUDED.updated_at
             WHERE user_algs.user_id = EXCLUDED.user_id
               AND user_algs.updated_at <= EXCLUDED.updated_at",
            vec![user_id.into(), user_algs.into()],
        ))
        .await?;
    }

    if !incoming.results.is_empty() {
        let results = serde_json::to_value(&incoming.results)
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO results
                (id, user_id, datetime, result_ms, trainer_id, case_id, updated_at, deleted)
             SELECT source.id, $1, source.datetime, source.result_ms, source.trainer_id,
                    source.case_id, source.updated_at, source.deleted
             FROM jsonb_to_recordset($2::jsonb) AS source(
                id uuid, datetime timestamptz, result_ms bigint, trainer_id text,
                case_id text, updated_at timestamptz, deleted boolean
             )
             ON CONFLICT (user_id, id) DO UPDATE
             SET datetime = EXCLUDED.datetime, result_ms = EXCLUDED.result_ms,
                 trainer_id = EXCLUDED.trainer_id, case_id = EXCLUDED.case_id,
                 updated_at = EXCLUDED.updated_at, deleted = EXCLUDED.deleted
             WHERE results.user_id = EXCLUDED.user_id
               AND results.updated_at <= EXCLUDED.updated_at",
            vec![user_id.into(), results.into()],
        ))
        .await?;
    }

    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "DELETE FROM sync_revisions WHERE user_id = $1 AND updated_at < $2",
        vec![user_id.into(), (Utc::now() - Duration::days(30)).into()],
    ))
    .await?;
    Ok(())
}

#[derive(Debug)]
pub enum SaveError {
    Validation(ValidationError),
    Database(DbErr),
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
            bundle.validate(),
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
            bundle.validate(),
            Err(ValidationError::InvalidField("algorithm"))
        ));
    }
}
