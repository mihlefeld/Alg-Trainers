use crate::logic::search::case_matches_search;
use crate::models::{AlgsetClickAction, SelectionPreset, TrainerBundle};
use std::collections::{BTreeMap, BTreeSet};

pub fn algset_click_action(
    enabled: bool,
    selected_count: usize,
    total: usize,
) -> AlgsetClickAction {
    if !enabled {
        AlgsetClickAction::SelectAll
    } else if selected_count == 0 {
        AlgsetClickAction::Disable
    } else if selected_count < total {
        AlgsetClickAction::SelectAll
    } else {
        AlgsetClickAction::SelectNone
    }
}

pub fn toggle_single_case(selected: &[String], case_id: &str) -> Vec<String> {
    let mut next = selected.to_vec();
    if let Some(index) = next.iter().position(|id| id == case_id) {
        next.remove(index);
    } else {
        next.push(case_id.to_owned());
    }
    next
}

pub fn toggle_case_ids(selected: &[String], case_ids: &[String]) -> Vec<String> {
    let mut next = selected.to_vec();
    if case_ids.iter().all(|id| next.contains(id)) {
        next.retain(|id| !case_ids.contains(id));
    } else {
        for id in case_ids {
            if !next.contains(id) {
                next.push(id.clone());
            }
        }
    }
    next
}

pub fn apply_algset_toggle(
    selected: &[String],
    selected_algsets: &BTreeMap<String, bool>,
    name: String,
    group_case_ids: &[String],
) -> (Vec<String>, BTreeMap<String, bool>) {
    let mut algsets = selected_algsets.clone();
    let mut cases = selected.to_vec();
    let enabled = algsets.get(&name).copied().unwrap_or(true);
    let selected_count = group_case_ids
        .iter()
        .filter(|id| cases.contains(id))
        .count();
    match algset_click_action(enabled, selected_count, group_case_ids.len()) {
        AlgsetClickAction::Disable => {
            algsets.insert(name, false);
        }
        AlgsetClickAction::SelectAll => {
            algsets.insert(name, true);
            for id in group_case_ids {
                if !cases.contains(id) {
                    cases.push(id.clone());
                }
            }
        }
        AlgsetClickAction::SelectNone => {
            cases.retain(|id| !group_case_ids.contains(id));
        }
    }
    (cases, algsets)
}

pub fn add_preset_cases(
    selected: &[String],
    selected_algsets: &BTreeMap<String, bool>,
    preset: SelectionPreset,
) -> (Vec<String>, BTreeMap<String, bool>) {
    let mut cases = selected.to_vec();
    for case_id in preset.sel_cases {
        if !cases.contains(&case_id) {
            cases.push(case_id);
        }
    }
    let mut algsets = selected_algsets.clone();
    algsets.extend(preset.selected_alg_sets);
    (cases, algsets)
}

pub fn remove_preset_cases(
    selected: &[String],
    selected_algsets: &BTreeMap<String, bool>,
    preset: SelectionPreset,
) -> (Vec<String>, BTreeMap<String, bool>) {
    let cases = selected
        .iter()
        .filter(|id| !preset.sel_cases.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    let mut algsets = selected_algsets.clone();
    for (name, value) in preset.selected_alg_sets {
        if !value {
            algsets.insert(name, false);
        }
    }
    (cases, algsets)
}

pub fn toggle_bookmark(
    presets: &BTreeMap<String, SelectionPreset>,
    case_id: &str,
) -> BTreeMap<String, SelectionPreset> {
    let mut next = presets.clone();
    let preset = next
        .entry("Bookmarks".to_owned())
        .or_insert_with(|| SelectionPreset {
            sel_cases: Vec::new(),
            selected_alg_sets: BTreeMap::new(),
        });
    if let Some(index) = preset.sel_cases.iter().position(|id| id == case_id) {
        preset.sel_cases.remove(index);
    } else {
        preset.sel_cases.push(case_id.to_owned());
    }
    if preset.sel_cases.is_empty() {
        next.remove("Bookmarks");
    }
    next
}

pub struct SelectionMetrics {
    pub search_active: bool,
    pub enabled_case_ids: BTreeSet<String>,
    pub visible_case_ids: Vec<String>,
    pub hidden_cases: usize,
    pub hidden_selected: usize,
    pub widest_group_cases: usize,
    pub selection_columns: usize,
}

pub fn compute_selection_metrics(
    bundle: &TrainerBundle,
    selected: &[String],
    selected_algsets: &BTreeMap<String, bool>,
    search_needle: &str,
    search_cells: &BTreeMap<usize, String>,
    search_all_angles: bool,
) -> SelectionMetrics {
    let search_active = !search_needle.is_empty() || !search_cells.is_empty();
    let enabled_case_ids = bundle
        .groups
        .iter()
        .filter(|group| {
            bundle.algsets.iter().any(|algset| {
                algset.groups.contains(&group.name)
                    && selected_algsets
                        .get(&algset.name)
                        .copied()
                        .unwrap_or(algset.selected)
            })
        })
        .flat_map(|group| group.cases.iter().cloned())
        .collect::<BTreeSet<_>>();
    let visible_case_ids = bundle
        .cases
        .iter()
        .filter(|case| enabled_case_ids.contains(&case.case_id))
        .filter(|case| {
            case_matches_search(case, bundle, search_needle, search_cells, search_all_angles)
        })
        .map(|case| case.case_id.clone())
        .collect::<Vec<_>>();
    let hidden_cases = bundle.cases.len().saturating_sub(visible_case_ids.len());
    let hidden_selected = selected
        .iter()
        .filter(|id| !visible_case_ids.contains(id))
        .count();
    let visible_case_id_set = visible_case_ids.iter().collect::<BTreeSet<_>>();
    let widest_group_cases = bundle
        .groups
        .iter()
        .filter(|group| {
            bundle.algsets.iter().any(|algset| {
                algset.groups.contains(&group.name)
                    && selected_algsets
                        .get(&algset.name)
                        .copied()
                        .unwrap_or(algset.selected)
            })
        })
        .map(|group| {
            group
                .cases
                .iter()
                .filter(|id| visible_case_id_set.contains(id))
                .count()
        })
        .max()
        .unwrap_or(0);
    let selection_columns = widest_group_cases
        .max(1)
        .min(bundle.max_algs_per_row.max(1) as usize);

    SelectionMetrics {
        search_active,
        enabled_case_ids,
        visible_case_ids,
        hidden_cases,
        hidden_selected,
        widest_group_cases,
        selection_columns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_algset_is_enabled_and_selects_all_cases() {
        assert_eq!(
            algset_click_action(false, 0, 4),
            AlgsetClickAction::SelectAll
        );
    }

    #[test]
    fn empty_algset_is_disabled() {
        assert_eq!(algset_click_action(true, 0, 4), AlgsetClickAction::Disable);
    }

    #[test]
    fn partial_algset_selects_all_cases() {
        assert_eq!(
            algset_click_action(true, 2, 4),
            AlgsetClickAction::SelectAll
        );
    }

    #[test]
    fn complete_algset_clears_cases() {
        assert_eq!(
            algset_click_action(true, 4, 4),
            AlgsetClickAction::SelectNone
        );
    }

    #[test]
    fn toggle_case_ids_selects_or_clears_all() {
        let group = vec!["1".to_owned(), "2".to_owned()];
        assert_eq!(toggle_case_ids(&["1".to_owned()], &group), vec!["1", "2"]);
        assert!(toggle_case_ids(&["1".to_owned(), "2".to_owned()], &group).is_empty());
    }

    #[test]
    fn toggle_bookmark_adds_and_prunes_empty_preset() {
        let presets = toggle_bookmark(&BTreeMap::new(), "case1");
        assert_eq!(presets["Bookmarks"].sel_cases, vec!["case1"]);
        let presets = toggle_bookmark(&presets, "case1");
        assert!(!presets.contains_key("Bookmarks"));
    }
}
