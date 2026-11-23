use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, DbErr, Statement,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

pub type CatalogError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainerSummary {
    pub trainer_id: String,
    pub puzzle: String,
    pub method: String,
    pub name: String,
    pub order: i32,
    pub max_algs_per_row: i32,
    pub selection_key: String,
    pub times_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainerCase {
    pub case_id: String,
    pub name: String,
    pub setup: Option<String>,
    pub image_svg: Option<String>,
    pub algs: Vec<TrainerAlg>,
    pub scrambles: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainerAlg {
    pub alg_id: String,
    pub alg: String,
    pub position: i32,
}

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

async fn update_max_algs_per_row(
    db: &DatabaseConnection,
    root: &Path,
    index: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), CatalogError> {
    for entries in index.values() {
        let entries: Vec<IndexEntry> = serde_json::from_value(entries.clone())?;
        for entry in entries {
            let max_algs_per_row = read_trainer_metadata(root, &entry.location)
                .map(|metadata| metadata.max_algs_per_row)
                .unwrap_or(8);
            db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE trainers SET max_algs_per_row=$1 WHERE trainer_id=$2",
                vec![max_algs_per_row.into(), entry.location.into()],
            ))
            .await?;
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainerGroup {
    pub name: String,
    pub cases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainerAlgset {
    pub name: String,
    pub groups: Vec<String>,
    pub selected: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainerBundle {
    pub trainer_id: String,
    pub title: String,
    pub puzzle: String,
    pub method: String,
    pub cases: Vec<TrainerCase>,
    pub groups: Vec<TrainerGroup>,
    pub algsets: Vec<TrainerAlgset>,
    pub pre_rotations: Vec<String>,
    pub post_rotations: Vec<String>,
    pub pre_moves: Vec<String>,
    pub post_moves: Vec<String>,
    pub max_algs_per_row: i32,
    #[serde(default)]
    pub u_move_cycles: Vec<Vec<usize>>,
    pub selection_key: String,
    pub times_key: String,
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

fn storage_keys(trainer_id: &str) -> (String, String) {
    let root = std::env::var("CATALOG_ROOT").unwrap_or_else(|_| "static".to_owned());
    let metadata = read_trainer_metadata(Path::new(&root), trainer_id);
    (
        metadata
            .as_ref()
            .and_then(|value| value.selection_array_key.clone())
            .unwrap_or_else(|| format!("{trainer_id}Selection")),
        metadata
            .as_ref()
            .and_then(|value| value.times_array_key.clone())
            .unwrap_or_else(|| format!("{trainer_id}Times")),
    )
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
        let row = db
            .query_one(Statement::from_string(
                DatabaseBackend::Postgres,
                "SELECT count(*) AS count FROM trainers",
            ))
            .await?;
        let existing = row
            .and_then(|row| row.try_get::<i64>("", "count").ok())
            .unwrap_or_default();
        if existing >= expected {
            update_max_algs_per_row(db, root, &index).await?;
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
    let post_rotations = metadata
        .as_ref()
        .map(|value| value.post_rotations.clone())
        .unwrap_or_default();
    let pre_moves = metadata
        .as_ref()
        .map(|value| value.pre_moves.clone())
        .unwrap_or_default();
    let post_moves = metadata
        .as_ref()
        .map(|value| value.post_moves.clone())
        .unwrap_or_default();
    let icon_path = format!("/catalog/{}/icon.png", entry.location);

    execute(
        db,
        "INSERT INTO trainers (trainer_id, puzzle, method, name, \"order\", icon_path, title, pre_rotations, post_rotations, pre_moves, post_moves, selected_algsets, max_algs_per_row)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
         ON CONFLICT (trainer_id) DO UPDATE SET puzzle=EXCLUDED.puzzle, method=EXCLUDED.method,
         name=EXCLUDED.name, \"order\"=EXCLUDED.\"order\", icon_path=EXCLUDED.icon_path, title=EXCLUDED.title,
         pre_rotations=EXCLUDED.pre_rotations, post_rotations=EXCLUDED.post_rotations,
         pre_moves=EXCLUDED.pre_moves, post_moves=EXCLUDED.post_moves, selected_algsets=EXCLUDED.selected_algsets,
         max_algs_per_row=EXCLUDED.max_algs_per_row",
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
    let rows = db
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT trainer_id,puzzle,method,name,\"order\",max_algs_per_row FROM trainers ORDER BY \"order\"",
        ))
        .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let trainer_id: String = row.try_get("", "trainer_id").ok()?;
            let (selection_key, times_key) = storage_keys(&trainer_id);
            Some(TrainerSummary {
                trainer_id,
                puzzle: row.try_get("", "puzzle").ok()?,
                method: row.try_get("", "method").ok()?,
                name: row.try_get("", "name").ok()?,
                order: row.try_get("", "order").ok()?,
                max_algs_per_row: row.try_get("", "max_algs_per_row").ok()?,
                selection_key,
                times_key,
            })
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
    let Some(trainer) = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT trainer_id,title,puzzle,method,pre_rotations,post_rotations,pre_moves,post_moves,selected_algsets,max_algs_per_row
             FROM trainers WHERE trainer_id=$1",
            vec![trainer_id.into()],
        ))
        .await?
    else {
        return Ok(None);
    };
    let selected_algsets: serde_json::Value = trainer.try_get("", "selected_algsets")?;
    let root = std::env::var("CATALOG_ROOT").unwrap_or_else(|_| "static".to_owned());
    let metadata = read_trainer_metadata(Path::new(&root), trainer_id);
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
    let u_move_cycles = fs::read(Path::new(&root).join(trainer_id).join("u_move.json"))
        .ok()
        .and_then(|value| serde_json::from_slice(&value).ok())
        .unwrap_or_default();

    let case_rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT case_id,name,case_setup,image_svg FROM cases
             WHERE trainer_id=$1 ORDER BY case_id",
            vec![trainer_id.into()],
        ))
        .await?;
    let alg_rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT case_id,alg_id,alg,position FROM algs
             WHERE trainer_id=$1 ORDER BY case_id, position",
            vec![trainer_id.into()],
        ))
        .await?;
    let scramble_rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT case_id,scramble FROM scrambles
             WHERE trainer_id=$1 ORDER BY case_id, position",
            vec![trainer_id.into()],
        ))
        .await?;

    let mut algs_by_case: BTreeMap<String, Vec<TrainerAlg>> = BTreeMap::new();
    for row in alg_rows {
        let case_id: String = row.try_get("", "case_id")?;
        algs_by_case.entry(case_id).or_default().push(TrainerAlg {
            alg_id: row.try_get("", "alg_id")?,
            alg: row.try_get("", "alg")?,
            position: row.try_get("", "position")?,
        });
    }
    let mut scrambles_by_case: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in scramble_rows {
        let case_id: String = row.try_get("", "case_id")?;
        scrambles_by_case
            .entry(case_id)
            .or_default()
            .push(row.try_get("", "scramble")?);
    }

    let cases = case_rows
        .into_iter()
        .map(|row| -> Result<TrainerCase, DbErr> {
            let case_id: String = row.try_get("", "case_id")?;
            Ok(TrainerCase {
                name: row.try_get("", "name")?,
                setup: row.try_get("", "case_setup")?,
                image_svg: row.try_get("", "image_svg")?,
                algs: algs_by_case.get(&case_id).cloned().unwrap_or_default(),
                scrambles: scrambles_by_case.get(&case_id).cloned().unwrap_or_default(),
                case_id,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let group_rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT g.name,array_agg(ga.case_id ORDER BY ga.\"order\") cases
             FROM groups g LEFT JOIN group_assignments ga ON ga.group_id=g.id
             WHERE g.trainer_id=$1 GROUP BY g.id,g.name,g.\"order\" ORDER BY g.\"order\"",
            vec![trainer_id.into()],
        ))
        .await?;
    let groups = group_rows
        .into_iter()
        .filter_map(|row| {
            Some(TrainerGroup {
                name: row.try_get("", "name").ok()?,
                cases: row.try_get::<Vec<String>>("", "cases").unwrap_or_default(),
            })
        })
        .collect();

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
        .filter_map(|row| {
            let name: String = row.try_get("", "name").ok()?;
            Some(TrainerAlgset {
                selected: selected_algsets
                    .get(&name)
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true),
                name,
                groups: row.try_get::<Vec<String>>("", "groups").unwrap_or_default(),
            })
        })
        .collect();

    let bundle = TrainerBundle {
        selection_key: storage_keys(trainer_id).0,
        times_key: storage_keys(trainer_id).1,
        trainer_id: trainer.try_get("", "trainer_id")?,
        title: trainer.try_get("", "title")?,
        puzzle: trainer.try_get("", "puzzle")?,
        method: trainer.try_get("", "method")?,
        cases,
        groups,
        algsets,
        pre_rotations,
        post_rotations,
        pre_moves,
        post_moves,
        max_algs_per_row: trainer.try_get("", "max_algs_per_row")?,
        u_move_cycles,
    };
    transaction.commit().await?;
    Ok(Some(bundle))
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
