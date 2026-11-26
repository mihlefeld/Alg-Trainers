use crate::models::{DisplayAlg, TrainerCase, UserAlgState};
use std::collections::{BTreeMap, BTreeSet};

pub fn parse_custom_algs(value: &serde_json::Value, case_id: &str) -> Vec<UserAlgState> {
    value
        .get("items")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| serde_json::from_value(serde_json::Value::Array(items.clone())).ok())
        .or_else(|| {
            value
                .get("a")
                .and_then(serde_json::Value::as_array)
                .map(|algs| {
                    algs.iter()
                        .filter_map(serde_json::Value::as_str)
                        .enumerate()
                        .map(|(position, alg)| UserAlgState {
                            alg_id: format!("legacy:{case_id}:{position}"),
                            alg: alg.to_owned(),
                            position: position as i32,
                            hidden: false,
                            deleted: false,
                        })
                        .collect()
                })
        })
        .or_else(|| {
            value
                .get("alg")
                .and_then(serde_json::Value::as_str)
                .map(|alg| {
                    vec![UserAlgState {
                        alg_id: format!("legacy:{case_id}:0"),
                        alg: alg.to_owned(),
                        position: 0,
                        hidden: false,
                        deleted: false,
                    }]
                })
        })
        .unwrap_or_default()
}

pub fn custom_algs_payload(
    custom_algs: &BTreeMap<String, Vec<UserAlgState>>,
) -> serde_json::Map<String, serde_json::Value> {
    custom_algs
        .iter()
        .map(|(case_id, items)| (case_id.clone(), serde_json::json!({ "items": items })))
        .collect()
}

pub fn display_algs(
    case: &TrainerCase,
    custom_algs: &BTreeMap<String, Vec<UserAlgState>>,
) -> Vec<DisplayAlg> {
    let overrides = custom_algs.get(&case.case_id).cloned().unwrap_or_default();
    let mut result = case
        .algs
        .iter()
        .filter_map(|alg| {
            let override_item = overrides.iter().find(|item| item.alg_id == alg.alg_id);
            if override_item.is_some_and(|item| item.hidden || item.deleted) {
                return None;
            }
            Some(DisplayAlg {
                alg_id: alg.alg_id.clone(),
                alg: override_item
                    .map(|item| item.alg.clone())
                    .unwrap_or_else(|| alg.alg.clone()),
                position: override_item
                    .map(|item| item.position)
                    .unwrap_or(alg.position),
                user: false,
                hidden: false,
            })
        })
        .collect::<Vec<_>>();
    let builtin_ids = case
        .algs
        .iter()
        .map(|alg| alg.alg_id.as_str())
        .collect::<BTreeSet<_>>();
    result.extend(
        overrides
            .iter()
            .filter(|item| {
                !item.hidden && !item.deleted && !builtin_ids.contains(item.alg_id.as_str())
            })
            .map(|item| DisplayAlg {
                alg_id: item.alg_id.clone(),
                alg: item.alg.clone(),
                position: item.position,
                user: true,
                hidden: false,
            }),
    );
    result.sort_by_key(|item| item.position);
    result
}

pub fn hidden_algs(
    case: &TrainerCase,
    custom_algs: &BTreeMap<String, Vec<UserAlgState>>,
) -> Vec<DisplayAlg> {
    let overrides = custom_algs.get(&case.case_id).cloned().unwrap_or_default();
    let builtin_ids = case
        .algs
        .iter()
        .map(|alg| alg.alg_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut result = Vec::new();

    for alg in &case.algs {
        if overrides
            .iter()
            .any(|item| item.alg_id == alg.alg_id && item.hidden && !item.deleted)
        {
            let override_item = overrides.iter().find(|item| item.alg_id == alg.alg_id);
            result.push(DisplayAlg {
                alg_id: alg.alg_id.clone(),
                alg: override_item
                    .map(|item| item.alg.clone())
                    .unwrap_or_else(|| alg.alg.clone()),
                position: override_item
                    .map(|item| item.position)
                    .unwrap_or(alg.position),
                user: false,
                hidden: true,
            });
        }
    }

    result.extend(
        overrides
            .iter()
            .filter(|item| {
                item.hidden && !item.deleted && !builtin_ids.contains(item.alg_id.as_str())
            })
            .map(|item| DisplayAlg {
                alg_id: item.alg_id.clone(),
                alg: item.alg.clone(),
                position: item.position,
                user: true,
                hidden: true,
            }),
    );
    result.sort_by_key(|item| item.position);
    result
}

pub fn upgrade_legacy_custom_algs(
    case: &TrainerCase,
    items: Vec<UserAlgState>,
) -> Vec<UserAlgState> {
    if !items.iter().any(|item| item.alg_id.starts_with("legacy:")) {
        return items;
    }
    let mut upgraded = Vec::new();
    for alg in &case.algs {
        if !items.iter().any(|item| item.alg_id == alg.alg_id) {
            upgraded.push(UserAlgState {
                alg_id: alg.alg_id.clone(),
                alg: alg.alg.clone(),
                position: alg.position,
                hidden: true,
                deleted: false,
            });
        }
    }
    upgraded.extend(items);
    upgraded
}

pub fn upsert_user_alg(
    custom_algs: &BTreeMap<String, Vec<UserAlgState>>,
    case: &TrainerCase,
    alg_id: Option<&str>,
    new_alg_id: impl FnOnce() -> String,
    value: &str,
) -> Option<BTreeMap<String, Vec<UserAlgState>>> {
    let trimmed = value.trim().to_owned();
    if trimmed.is_empty() {
        return None;
    }
    let mut next = custom_algs.clone();
    if let Some(alg_id) = alg_id {
        let current = next.entry(case.case_id.clone()).or_default();
        if let Some(item) = current.iter_mut().find(|item| item.alg_id == alg_id) {
            item.alg = trimmed;
            item.deleted = false;
        }
    } else {
        let position = display_algs(case, &next)
            .iter()
            .map(|item| item.position)
            .max()
            .unwrap_or(-1)
            + 1;
        next.entry(case.case_id.clone())
            .or_default()
            .push(UserAlgState {
                alg_id: new_alg_id(),
                alg: trimmed,
                position,
                hidden: false,
                deleted: false,
            });
    }
    Some(next)
}

pub fn hide_alg(
    custom_algs: &BTreeMap<String, Vec<UserAlgState>>,
    case_id: &str,
    item: DisplayAlg,
) -> BTreeMap<String, Vec<UserAlgState>> {
    let mut next = custom_algs.clone();
    let current = next.entry(case_id.to_owned()).or_default();
    if let Some(existing) = current.iter_mut().find(|alg| alg.alg_id == item.alg_id) {
        existing.hidden = !item.hidden;
    } else {
        current.push(UserAlgState {
            alg_id: item.alg_id,
            alg: item.alg,
            position: item.position,
            hidden: true,
            deleted: false,
        });
    }
    next
}

pub fn delete_alg(
    custom_algs: &BTreeMap<String, Vec<UserAlgState>>,
    case_id: &str,
    alg_id: &str,
) -> BTreeMap<String, Vec<UserAlgState>> {
    let mut next = custom_algs.clone();
    if let Some(item) = next
        .entry(case_id.to_owned())
        .or_default()
        .iter_mut()
        .find(|item| item.alg_id == alg_id)
    {
        item.deleted = true;
    }
    next
}

pub fn reorder_alg(
    custom_algs: &BTreeMap<String, Vec<UserAlgState>>,
    case: &TrainerCase,
    from: &str,
    to: &str,
) -> Option<BTreeMap<String, Vec<UserAlgState>>> {
    let mut items = display_algs(case, custom_algs);
    let from_index = items.iter().position(|item| item.alg_id == from)?;
    let to_index = items.iter().position(|item| item.alg_id == to)?;
    let moved = items.remove(from_index);
    items.insert(to_index, moved);

    let mut next = custom_algs.clone();
    let overrides = next.entry(case.case_id.clone()).or_default();
    for (position, item) in items.into_iter().enumerate() {
        if let Some(existing) = overrides
            .iter_mut()
            .find(|existing| existing.alg_id == item.alg_id)
        {
            existing.position = position as i32;
        } else {
            overrides.push(UserAlgState {
                alg_id: item.alg_id,
                alg: item.alg,
                position: position as i32,
                hidden: false,
                deleted: false,
            });
        }
    }
    Some(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TrainerAlg;

    fn test_case() -> TrainerCase {
        TrainerCase {
            case_id: "case".to_owned(),
            name: "Case".to_owned(),
            setup: None,
            image_svg: None,
            algs: vec![
                TrainerAlg {
                    alg_id: "builtin:1".to_owned(),
                    alg: "R U".to_owned(),
                    position: 0,
                },
                TrainerAlg {
                    alg_id: "builtin:2".to_owned(),
                    alg: "R' U'".to_owned(),
                    position: 1,
                },
            ],
            scrambles: Vec::new(),
        }
    }

    #[test]
    fn user_alg_state_controls_visibility_order_and_additions() {
        let case = test_case();
        let custom = BTreeMap::from([(
            "case".to_owned(),
            vec![
                UserAlgState {
                    alg_id: "builtin:1".to_owned(),
                    alg: "R U".to_owned(),
                    position: 2,
                    hidden: true,
                    deleted: false,
                },
                UserAlgState {
                    alg_id: "user:1".to_owned(),
                    alg: "F R".to_owned(),
                    position: 0,
                    hidden: false,
                    deleted: false,
                },
                UserAlgState {
                    alg_id: "builtin:2".to_owned(),
                    alg: "R' U'".to_owned(),
                    position: 1,
                    hidden: false,
                    deleted: false,
                },
            ],
        )]);

        let displayed = display_algs(&case, &custom);
        assert_eq!(
            displayed
                .iter()
                .map(|item| item.alg.as_str())
                .collect::<Vec<_>>(),
            vec!["F R", "R' U'"]
        );
        assert!(displayed[0].user);
        assert!(!displayed[1].user);
    }

    #[test]
    fn legacy_custom_algs_hide_catalog_defaults() {
        let case = test_case();
        let migrated = upgrade_legacy_custom_algs(
            &case,
            vec![UserAlgState {
                alg_id: "legacy:case:0".to_owned(),
                alg: "U R".to_owned(),
                position: 0,
                hidden: false,
                deleted: false,
            }],
        );
        let displayed = display_algs(&case, &BTreeMap::from([("case".to_owned(), migrated)]));

        assert_eq!(displayed.len(), 1);
        assert_eq!(displayed[0].alg, "U R");
        assert!(displayed[0].user);
    }

    #[test]
    fn reorder_and_hide_and_upsert_mutate_cleanly() {
        let case = test_case();
        let custom = BTreeMap::new();
        let custom = upsert_user_alg(&custom, &case, None, || "user:a".to_owned(), "  M2 U2  ")
            .expect("added");
        let custom = reorder_alg(&custom, &case, "user:a", "builtin:1").expect("reordered");
        let displayed = display_algs(&case, &custom);
        assert_eq!(
            displayed
                .iter()
                .map(|d| d.alg_id.as_str())
                .collect::<Vec<_>>(),
            vec!["user:a", "builtin:1", "builtin:2"]
        );
        let custom = hide_alg(&custom, &case.case_id, displayed[1].clone());
        let custom = delete_alg(&custom, &case.case_id, "user:a");
        let remaining = display_algs(&case, &custom);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].alg_id, "builtin:2");

        let hidden = hidden_algs(&case, &custom);
        assert_eq!(hidden.len(), 1);
        assert_eq!(hidden[0].alg_id, "builtin:1");
        assert!(hidden[0].hidden);

        let unhidden = hide_alg(&custom, &case.case_id, hidden[0].clone());
        assert_eq!(display_algs(&case, &unhidden).len(), 2);
        assert!(hidden_algs(&case, &unhidden).is_empty());
    }
}
