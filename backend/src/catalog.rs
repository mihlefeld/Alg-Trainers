pub use alg_trainers_shared::{
    TrainerAlg, TrainerAlgset, TrainerBundle, TrainerCase, TrainerGroup, TrainerSummary,
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Statement, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

use crate::entity;

pub type CatalogError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrainerMetadata {
    trainer_title: Option<String>,
    selection_array_key: Option<String>,
    times_array_key: Option<String>,
    #[serde(default)]
    pre_rotations: Vec<String>,
    #[serde(default)]
    post_rotations: Vec<String>,
    #[serde(default)]
    pre_moves: Vec<String>,
    #[serde(default)]
    post_moves: Vec<String>,
    #[serde(default = "default_max_algs_per_row")]
    max_algs_per_row: i32,
}

fn default_max_algs_per_row() -> i32 {
    8
}

fn read_trainer_metadata(root: &Path, trainer_id: &str) -> Option<TrainerMetadata> {
    read_json(root.join(trainer_id).join("metadata.json")).ok()
}

fn read_u_move_cycles(root: &Path, trainer_id: &str) -> Vec<Vec<usize>> {
    fs::read(root.join(trainer_id).join("u_move.json"))
        .ok()
        .and_then(|value| serde_json::from_slice(&value).ok())
        .unwrap_or_default()
}

fn resolve_storage_keys(trainer_id: &str, metadata: Option<&TrainerMetadata>) -> (String, String) {
    (
        metadata
            .and_then(|value| value.selection_array_key.clone())
            .unwrap_or_else(|| format!("{trainer_id}Selection")),
        metadata
            .and_then(|value| value.times_array_key.clone())
            .unwrap_or_else(|| format!("{trainer_id}Times")),
    )
}

async fn update_catalog_metadata(
    db: &DatabaseConnection,
    root: &Path,
    index: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), CatalogError> {
    let transaction = db.begin().await?;
    for entries in index.values() {
        let entries: Vec<IndexEntry> = serde_json::from_value(entries.clone())?;
        for entry in entries {
            let metadata = read_trainer_metadata(root, &entry.location);
            let max_algs_per_row = metadata
                .as_ref()
                .map(|value| value.max_algs_per_row)
                .unwrap_or(8);
            let pre_rotations = metadata
                .as_ref()
                .map(|value| value.pre_rotations.clone())
                .unwrap_or_default();
            let post_rotations = normalize_option_alias_text(
                metadata
                    .as_ref()
                    .map(|value| value.post_rotations.clone())
                    .unwrap_or_default(),
                &pre_rotations,
                "preRotations",
            );
            let pre_moves = metadata
                .as_ref()
                .map(|value| value.pre_moves.clone())
                .unwrap_or_default();
            let post_moves = normalize_option_alias_text(
                metadata
                    .as_ref()
                    .map(|value| value.post_moves.clone())
                    .unwrap_or_default(),
                &pre_moves,
                "preMoves",
            );
            let (selection_key, times_key) =
                resolve_storage_keys(&entry.location, metadata.as_ref());
            let u_move_cycles = read_u_move_cycles(root, &entry.location);
            execute(
                &transaction,
                "UPDATE trainers
                 SET max_algs_per_row = $1,
                     pre_rotations = $2,
                     post_rotations = $3,
                     pre_moves = $4,
                     post_moves = $5,
                     selection_key = $6,
                     times_key = $7,
                     u_move_cycles = $8
                 WHERE trainer_id = $9",
                vec![
                    max_algs_per_row.into(),
                    serde_json::to_value(pre_rotations)?.into(),
                    serde_json::to_value(post_rotations)?.into(),
                    serde_json::to_value(pre_moves)?.into(),
                    serde_json::to_value(post_moves)?.into(),
                    selection_key.into(),
                    times_key.into(),
                    serde_json::to_value(u_move_cycles)?.into(),
                    entry.location.into(),
                ],
            )
            .await?;
        }
    }
    transaction.commit().await?;
    Ok(())
}

#[derive(Clone, Deserialize, Serialize)]
struct IndexEntry {
    name: String,
    location: String,
}

#[derive(Deserialize)]
struct AlgInfo {
    a: Vec<String>,
    name: serde_json::Value,
    group: String,
    #[serde(rename = "algset")]
    _algset: String,
    s: Option<String>,
}

fn read_json<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, CatalogError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn trainer_storage_keys(trainer: &entity::trainer::Model) -> (String, String) {
    (
        trainer
            .selection_key
            .clone()
            .unwrap_or_else(|| format!("{}Selection", trainer.trainer_id)),
        trainer
            .times_key
            .clone()
            .unwrap_or_else(|| format!("{}Times", trainer.trainer_id)),
    )
}

fn json_string_vec(value: serde_json::Value) -> Vec<String> {
    serde_json::from_value(value).unwrap_or_default()
}

fn json_u_move_cycles(value: serde_json::Value) -> Vec<Vec<usize>> {
    serde_json::from_value(value).unwrap_or_default()
}

pub async fn seed(db: &DatabaseConnection, root: &Path) -> Result<(), CatalogError> {
    let index: serde_json::Map<String, serde_json::Value> = read_json(root.join("index.json"))?;
    let expected = index
        .values()
        .map(|value| value.as_array().map(Vec::len).unwrap_or(0))
        .sum::<usize>() as i64;
    if std::env::var("FORCE_CATALOG_SEED")
        .ok()
        .filter(|value| !value.is_empty())
        .is_none()
    {
        let existing = entity::trainer::Entity::find().count(db).await?;
        if existing >= expected as u64 {
            update_catalog_metadata(db, root, &index).await?;
            return Ok(());
        }
    }
    eprintln!("Importing {expected} trainers into the catalog...");
    let transaction = db.begin().await?;
    let mut order = 0;
    for (puzzle, entries) in index {
        let entries: Vec<IndexEntry> = serde_json::from_value(entries)?;
        for entry in entries {
            order += 1;
            eprintln!("Importing trainer {order}/{expected}: {}", entry.location);
            seed_trainer(&transaction, root, &puzzle, &entry, order).await?;
        }
    }
    transaction.commit().await?;
    eprintln!("Catalog import complete.");
    Ok(())
}

async fn seed_trainer(
    db: &DatabaseTransaction,
    root: &Path,
    puzzle: &str,
    entry: &IndexEntry,
    order: i32,
) -> Result<(), CatalogError> {
    let directory = root.join(&entry.location);
    let metadata = read_trainer_metadata(root, &entry.location);
    let title = metadata
        .as_ref()
        .and_then(|value| value.trainer_title.clone())
        .unwrap_or_else(|| format!("{puzzle} {} Trainer", entry.name));
    let max_algs_per_row = metadata
        .as_ref()
        .map(|value| value.max_algs_per_row)
        .unwrap_or(8);
    let pre_rotations = metadata
        .as_ref()
        .map(|value| value.pre_rotations.clone())
        .unwrap_or_default();
    let post_rotations = normalize_option_alias_text(
        metadata
            .as_ref()
            .map(|value| value.post_rotations.clone())
            .unwrap_or_default(),
        &pre_rotations,
        "preRotations",
    );
    let pre_moves = metadata
        .as_ref()
        .map(|value| value.pre_moves.clone())
        .unwrap_or_default();
    let post_moves = normalize_option_alias_text(
        metadata
            .as_ref()
            .map(|value| value.post_moves.clone())
            .unwrap_or_default(),
        &pre_moves,
        "preMoves",
    );
    let (selection_key, times_key) = resolve_storage_keys(&entry.location, metadata.as_ref());
    let u_move_cycles = read_u_move_cycles(root, &entry.location);
    let icon_path = format!("/catalog/{}/icon.png", entry.location);

    execute(
        db,
        "INSERT INTO trainers (trainer_id, puzzle, method, name, \"order\", icon_path, title, pre_rotations, post_rotations, pre_moves, post_moves, selected_algsets, max_algs_per_row, selection_key, times_key, u_move_cycles)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
         ON CONFLICT (trainer_id) DO UPDATE SET puzzle=EXCLUDED.puzzle, method=EXCLUDED.method,
         name=EXCLUDED.name, \"order\"=EXCLUDED.\"order\", icon_path=EXCLUDED.icon_path, title=EXCLUDED.title,
         pre_rotations=EXCLUDED.pre_rotations, post_rotations=EXCLUDED.post_rotations,
         pre_moves=EXCLUDED.pre_moves, post_moves=EXCLUDED.post_moves, selected_algsets=EXCLUDED.selected_algsets,
         max_algs_per_row=EXCLUDED.max_algs_per_row, selection_key=EXCLUDED.selection_key,
         times_key=EXCLUDED.times_key, u_move_cycles=EXCLUDED.u_move_cycles",
        vec![
            entry.location.clone().into(),
            puzzle.into(),
            entry.name.clone().into(),
            entry.name.clone().into(),
            order.into(),
            icon_path.into(),
            title.into(),
            serde_json::to_value(pre_rotations)?.into(),
            serde_json::to_value(post_rotations)?.into(),
            serde_json::to_value(pre_moves)?.into(),
            serde_json::to_value(post_moves)?.into(),
            read_json::<serde_json::Value>(directory.join("selected_algsets.json"))?.into(),
            max_algs_per_row.into(),
            selection_key.into(),
            times_key.into(),
            serde_json::to_value(u_move_cycles)?.into(),
        ],
    )
    .await?;

    let algs: BTreeMap<String, AlgInfo> = read_json(directory.join("algs_info.json"))?;
    let images: BTreeMap<String, String> = read_json(directory.join("combined.json"))?;
    let scrambles: BTreeMap<String, Vec<String>> = read_json(directory.join("scrambles.json"))?;

    for (position, (case_id, info)) in algs.into_iter().enumerate() {
        let name = match info.name {
            serde_json::Value::String(value) => value,
            value => value.to_string(),
        };
        execute(
            db,
            "INSERT INTO cases (trainer_id, case_id, name, case_setup, image_svg)
             VALUES ($1,$2,$3,$4,$5)
             ON CONFLICT (trainer_id, case_id) DO UPDATE SET name=EXCLUDED.name,
             case_setup=EXCLUDED.case_setup, image_svg=EXCLUDED.image_svg",
            vec![
                entry.location.clone().into(),
                case_id.clone().into(),
                name.into(),
                info.s.clone().into(),
                images.get(&case_id).cloned().into(),
            ],
        )
        .await?;
        execute(
            db,
            "DELETE FROM algs WHERE trainer_id=$1 AND case_id=$2",
            vec![entry.location.clone().into(), case_id.clone().into()],
        )
        .await?;
        for (alg_position, alg) in info.a.into_iter().enumerate() {
            execute(
                db,
                "INSERT INTO algs (alg_id,trainer_id,case_id,alg,position) VALUES ($1,$2,$3,$4,$5)",
                vec![
                    format!("builtin:{}:{}:{}", entry.location, case_id, alg_position).into(),
                    entry.location.clone().into(),
                    case_id.clone().into(),
                    alg.into(),
                    (alg_position as i32).into(),
                ],
            )
            .await?;
        }
        execute(
            db,
            "DELETE FROM scrambles WHERE trainer_id=$1 AND case_id=$2",
            vec![entry.location.clone().into(), case_id.clone().into()],
        )
        .await?;
        for (scramble_position, scramble) in scrambles
            .get(&case_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .enumerate()
        {
            execute(
                db,
                "INSERT INTO scrambles (trainer_id,case_id,scramble,position) VALUES ($1,$2,$3,$4)",
                vec![
                    entry.location.clone().into(),
                    case_id.clone().into(),
                    scramble.into(),
                    (scramble_position as i32).into(),
                ],
            )
            .await?;
        }
        let _ = position;
        let _ = info.group;
    }

    let groups: serde_json::Map<String, serde_json::Value> =
        read_json(directory.join("groups_info.json"))?;
    let algsets: serde_json::Map<String, serde_json::Value> =
        read_json(directory.join("algsets_info.json"))?;
    execute(
        db,
        "DELETE FROM group_assignments WHERE trainer_id=$1",
        vec![entry.location.clone().into()],
    )
    .await?;
    execute(
        db,
        "DELETE FROM algset_assignments WHERE trainer_id=$1",
        vec![entry.location.clone().into()],
    )
    .await?;
    execute(
        db,
        "DELETE FROM groups WHERE trainer_id=$1",
        vec![entry.location.clone().into()],
    )
    .await?;
    execute(
        db,
        "DELETE FROM algsets WHERE trainer_id=$1",
        vec![entry.location.clone().into()],
    )
    .await?;

    let mut group_ids = BTreeMap::new();
    for (group_order, (name, case_ids)) in groups.into_iter().enumerate() {
        let case_ids = serde_json::from_value::<Vec<serde_json::Value>>(case_ids)?
            .into_iter()
            .map(|value| match value {
                serde_json::Value::String(value) => value,
                value => value.to_string(),
            })
            .collect::<Vec<_>>();
        let id = execute_returning_id(
            db,
            "INSERT INTO groups (trainer_id,name,\"order\") VALUES ($1,$2,$3) RETURNING id",
            vec![
                entry.location.clone().into(),
                name.clone().into(),
                (group_order as i32).into(),
            ],
        )
        .await?;
        for (case_order, case_id) in case_ids.into_iter().enumerate() {
            execute(
                db,
                "INSERT INTO group_assignments (group_id,trainer_id,case_id,\"order\") VALUES ($1,$2,$3,$4)",
                vec![id.into(), entry.location.clone().into(), case_id.into(), (case_order as i32).into()],
            )
            .await?;
        }
        group_ids.insert(name, id);
    }
    for (set_order, (name, group_names)) in algsets.into_iter().enumerate() {
        let group_names: Vec<String> = serde_json::from_value(group_names)?;
        let id = execute_returning_id(
            db,
            "INSERT INTO algsets (trainer_id,name,\"order\") VALUES ($1,$2,$3) RETURNING id",
            vec![
                entry.location.clone().into(),
                name.into(),
                (set_order as i32).into(),
            ],
        )
        .await?;
        for group_name in group_names {
            if let Some(group_id) = group_ids.get(&group_name) {
                execute(
                    db,
                    "INSERT INTO algset_assignments (algset_id,trainer_id,group_id) VALUES ($1,$2,$3)",
                    vec![id.into(), entry.location.clone().into(), (*group_id).into()],
                )
                .await?;
            }
        }
    }
    Ok(())
}

async fn execute(
    db: &DatabaseTransaction,
    sql: &str,
    values: Vec<sea_orm::Value>,
) -> Result<(), DbErr> {
    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        values,
    ))
    .await?;
    Ok(())
}

async fn execute_returning_id(
    db: &DatabaseTransaction,
    sql: &str,
    values: Vec<sea_orm::Value>,
) -> Result<i64, CatalogError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            values,
        ))
        .await?
        .ok_or("INSERT did not return a row")?;
    Ok(row.try_get("", "id")?)
}

pub async fn list(db: &DatabaseConnection) -> Result<Vec<TrainerSummary>, DbErr> {
    let trainers = entity::trainer::Entity::find()
        .order_by_asc(entity::trainer::Column::Order)
        .all(db)
        .await?;
    Ok(trainers
        .into_iter()
        .map(|trainer| {
            let (selection_key, times_key) = trainer_storage_keys(&trainer);
            TrainerSummary {
                trainer_id: trainer.trainer_id,
                puzzle: trainer.puzzle,
                method: trainer.method,
                name: trainer.name,
                order: trainer.order,
                max_algs_per_row: trainer.max_algs_per_row,
                selection_key,
                times_key,
            }
        })
        .collect())
}

pub async fn load(
    db: &DatabaseConnection,
    trainer_id: &str,
) -> Result<Option<TrainerBundle>, CatalogError> {
    let transaction = db.begin().await?;
    transaction
        .execute_unprepared("SET LOCAL statement_timeout = '5s'")
        .await?;
    let db = &transaction;
    let Some(trainer) = entity::trainer::Entity::find_by_id(trainer_id)
        .one(db)
        .await?
    else {
        return Ok(None);
    };
    let selected_algsets = trainer.selected_algsets.clone();
    let (selection_key, times_key) = trainer_storage_keys(&trainer);
    let pre_rotations = json_string_vec(trainer.pre_rotations);
    let post_rotations = normalize_option_alias_text(
        json_string_vec(trainer.post_rotations),
        &pre_rotations,
        "preRotations",
    );
    let pre_moves = json_string_vec(trainer.pre_moves);
    let post_moves = normalize_option_alias_text(
        json_string_vec(trainer.post_moves),
        &pre_moves,
        "preMoves",
    );
    let u_move_cycles = json_u_move_cycles(trainer.u_move_cycles);

    let case_models = entity::case::Entity::find()
        .filter(entity::case::Column::TrainerId.eq(trainer_id))
        .order_by_asc(entity::case::Column::CaseId)
        .all(db)
        .await?;
    let alg_models = entity::alg::Entity::find()
        .filter(entity::alg::Column::TrainerId.eq(trainer_id))
        .order_by_asc(entity::alg::Column::CaseId)
        .order_by_asc(entity::alg::Column::Position)
        .all(db)
        .await?;
    let scramble_models = entity::scramble::Entity::find()
        .filter(entity::scramble::Column::TrainerId.eq(trainer_id))
        .order_by_asc(entity::scramble::Column::CaseId)
        .order_by_asc(entity::scramble::Column::Position)
        .all(db)
        .await?;

    let mut algs_by_case: BTreeMap<String, Vec<TrainerAlg>> = BTreeMap::new();
    for model in alg_models {
        algs_by_case
            .entry(model.case_id)
            .or_default()
            .push(TrainerAlg {
                alg_id: model.alg_id,
                alg: model.alg,
                position: model.position,
            });
    }
    let mut scrambles_by_case: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for model in scramble_models {
        scrambles_by_case
            .entry(model.case_id)
            .or_default()
            .push(model.scramble);
    }

    let cases = case_models
        .into_iter()
        .map(|model| TrainerCase {
            name: model.name,
            setup: model.case_setup,
            image_svg: model.image_svg,
            algs: algs_by_case
                .get(&model.case_id)
                .cloned()
                .unwrap_or_default(),
            scrambles: scrambles_by_case
                .get(&model.case_id)
                .cloned()
                .unwrap_or_default(),
            case_id: model.case_id,
        })
        .collect();

    let group_rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT g.name,coalesce(
                array_agg(ga.case_id ORDER BY ga.\"order\") FILTER (WHERE ga.case_id IS NOT NULL),
                '{}'::text[]
             ) cases
             FROM groups g LEFT JOIN group_assignments ga ON ga.group_id=g.id
             WHERE g.trainer_id=$1 GROUP BY g.id,g.name,g.\"order\" ORDER BY g.\"order\"",
            vec![trainer_id.into()],
        ))
        .await?;
    let groups = group_rows
        .into_iter()
        .map(|row| -> Result<TrainerGroup, DbErr> {
            Ok(TrainerGroup {
                name: row.try_get("", "name")?,
                cases: row.try_get::<Vec<String>>("", "cases")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let algset_rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT a.name,array_agg(g.name ORDER BY g.\"order\") groups
             FROM algsets a
             JOIN algset_assignments aa ON aa.algset_id=a.id
             JOIN groups g ON g.id=aa.group_id
             WHERE a.trainer_id=$1 GROUP BY a.id,a.name,a.\"order\" ORDER BY a.\"order\"",
            vec![trainer_id.into()],
        ))
        .await?;
    let algsets = algset_rows
        .into_iter()
        .map(|row| -> Result<TrainerAlgset, DbErr> {
            let name: String = row.try_get("", "name")?;
            Ok(TrainerAlgset {
                selected: selected_algsets
                    .get(&name)
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true),
                name,
                groups: row.try_get::<Vec<String>>("", "groups")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let bundle = TrainerBundle {
        selection_key,
        times_key,
        trainer_id: trainer.trainer_id,
        title: trainer.title.ok_or("trainer title is missing")?,
        puzzle: trainer.puzzle,
        method: trainer.method,
        cases,
        groups,
        algsets,
        pre_rotations,
        post_rotations,
        pre_moves,
        post_moves,
        max_algs_per_row: trainer.max_algs_per_row,
        u_move_cycles,
    };
    transaction.commit().await?;
    Ok(Some(bundle))
}

pub async fn load_all(db: &DatabaseConnection) -> Result<Vec<TrainerBundle>, CatalogError> {
    let transaction = db.begin().await?;
    transaction
        .execute_unprepared("SET LOCAL statement_timeout = '30s'")
        .await?;
    let tx = &transaction;

    let trainers = entity::trainer::Entity::find()
        .order_by_asc(entity::trainer::Column::Order)
        .all(tx)
        .await?;
    let case_models = entity::case::Entity::find()
        .order_by_asc(entity::case::Column::TrainerId)
        .order_by_asc(entity::case::Column::CaseId)
        .all(tx)
        .await?;
    let alg_models = entity::alg::Entity::find()
        .order_by_asc(entity::alg::Column::TrainerId)
        .order_by_asc(entity::alg::Column::CaseId)
        .order_by_asc(entity::alg::Column::Position)
        .all(tx)
        .await?;
    let scramble_models = entity::scramble::Entity::find()
        .order_by_asc(entity::scramble::Column::TrainerId)
        .order_by_asc(entity::scramble::Column::CaseId)
        .order_by_asc(entity::scramble::Column::Position)
        .all(tx)
        .await?;

    let mut algs_by_case: BTreeMap<(String, String), Vec<TrainerAlg>> = BTreeMap::new();
    for model in alg_models {
        algs_by_case
            .entry((model.trainer_id, model.case_id))
            .or_default()
            .push(TrainerAlg {
                alg_id: model.alg_id,
                alg: model.alg,
                position: model.position,
            });
    }
    let mut scrambles_by_case: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for model in scramble_models {
        scrambles_by_case
            .entry((model.trainer_id, model.case_id))
            .or_default()
            .push(model.scramble);
    }
    let mut cases_by_trainer: BTreeMap<String, Vec<TrainerCase>> = BTreeMap::new();
    for model in case_models {
        let key = (model.trainer_id.clone(), model.case_id.clone());
        cases_by_trainer
            .entry(model.trainer_id)
            .or_default()
            .push(TrainerCase {
                name: model.name,
                setup: model.case_setup,
                image_svg: model.image_svg,
                algs: algs_by_case.remove(&key).unwrap_or_default(),
                scrambles: scrambles_by_case.remove(&key).unwrap_or_default(),
                case_id: model.case_id,
            });
    }

    let group_rows = tx
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT g.trainer_id, g.name, coalesce(
                array_agg(ga.case_id ORDER BY ga.\"order\") FILTER (WHERE ga.case_id IS NOT NULL),
                '{}'::text[]
             ) cases
             FROM groups g LEFT JOIN group_assignments ga ON ga.group_id=g.id
             GROUP BY g.id, g.trainer_id, g.name, g.\"order\"
             ORDER BY g.trainer_id, g.\"order\""
                .to_owned(),
        ))
        .await?;
    let mut groups_by_trainer: BTreeMap<String, Vec<TrainerGroup>> = BTreeMap::new();
    for row in group_rows {
        let trainer_id: String = row.try_get("", "trainer_id")?;
        groups_by_trainer
            .entry(trainer_id)
            .or_default()
            .push(TrainerGroup {
                name: row.try_get("", "name")?,
                cases: row.try_get::<Vec<String>>("", "cases")?,
            });
    }

    let algset_rows = tx
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT a.trainer_id, a.name, array_agg(g.name ORDER BY g.\"order\") groups
             FROM algsets a
             JOIN algset_assignments aa ON aa.algset_id=a.id
             JOIN groups g ON g.id=aa.group_id
             GROUP BY a.id, a.trainer_id, a.name, a.\"order\"
             ORDER BY a.trainer_id, a.\"order\""
                .to_owned(),
        ))
        .await?;
    let mut algsets_by_trainer: BTreeMap<String, Vec<(String, Vec<String>)>> = BTreeMap::new();
    for row in algset_rows {
        let trainer_id: String = row.try_get("", "trainer_id")?;
        algsets_by_trainer
            .entry(trainer_id)
            .or_default()
            .push((row.try_get("", "name")?, row.try_get::<Vec<String>>("", "groups")?));
    }

    let mut bundles = Vec::with_capacity(trainers.len());
    for trainer in trainers {
        let (selection_key, times_key) = trainer_storage_keys(&trainer);
        let trainer_id = trainer.trainer_id;
        let selected_algsets = trainer.selected_algsets;
        let pre_rotations = json_string_vec(trainer.pre_rotations);
        let post_rotations = normalize_option_alias_text(
            json_string_vec(trainer.post_rotations),
            &pre_rotations,
            "preRotations",
        );
        let pre_moves = json_string_vec(trainer.pre_moves);
        let post_moves = normalize_option_alias_text(
            json_string_vec(trainer.post_moves),
            &pre_moves,
            "preMoves",
        );
        let u_move_cycles = json_u_move_cycles(trainer.u_move_cycles);
        let algsets = algsets_by_trainer
            .remove(&trainer_id)
            .unwrap_or_default()
            .into_iter()
            .map(|(name, groups)| TrainerAlgset {
                selected: selected_algsets
                    .get(&name)
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true),
                name,
                groups,
            })
            .collect();
        bundles.push(TrainerBundle {
            selection_key,
            times_key,
            title: trainer.title.ok_or("trainer title is missing")?,
            puzzle: trainer.puzzle,
            method: trainer.method,
            cases: cases_by_trainer.remove(&trainer_id).unwrap_or_default(),
            groups: groups_by_trainer.remove(&trainer_id).unwrap_or_default(),
            algsets,
            pre_rotations,
            post_rotations,
            pre_moves,
            post_moves,
            max_algs_per_row: trainer.max_algs_per_row,
            u_move_cycles,
            trainer_id,
        });
    }
    transaction.commit().await?;
    Ok(bundles)
}

fn normalize_option_alias_text(
    values: Vec<String>,
    fallback: &[String],
    alias: &str,
) -> Vec<String> {
    if values == [alias] {
        fallback.to_vec()
    } else {
        values
    }
}

#[cfg(test)]
mod tests {
    use super::{read_json, TrainerMetadata};
    use std::path::Path;

    #[test]
    fn reads_json_trainer_metadata() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../static/2x2-EG-Trainer/metadata.json");
        let metadata: TrainerMetadata = read_json(path).expect("metadata");
        assert_eq!(metadata.max_algs_per_row, 6);
        assert_eq!(
            metadata.selection_array_key.as_deref(),
            Some("2x2egSelection")
        );
    }
}
