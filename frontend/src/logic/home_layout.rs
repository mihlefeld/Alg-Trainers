use crate::models::TrainerSummary;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SETTINGS_KEY: &str = "homePageLayout";

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct HomeLayout {
    pub edited: bool,
    pub puzzle_order: Vec<String>,
    pub trainer_order: BTreeMap<String, Vec<String>>,
    pub hidden_puzzles: BTreeSet<String>,
    pub hidden_trainers: BTreeSet<String>,
    pub known_trainers: BTreeSet<String>,
}

pub fn load(settings: &serde_json::Value) -> HomeLayout {
    settings
        .get(SETTINGS_KEY)
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

pub fn store(settings: &mut serde_json::Value, layout: &HomeLayout) {
    settings[SETTINGS_KEY] =
        serde_json::to_value(layout).expect("home page layout is serializable");
}

pub fn groups(
    trainers: &[TrainerSummary],
    layout: &HomeLayout,
    include_hidden: bool,
) -> Vec<(String, Vec<TrainerSummary>)> {
    let puzzle_position = positions(&layout.puzzle_order);
    let mut puzzle_names = Vec::<String>::new();
    for trainer in trainers {
        if !puzzle_names.contains(&trainer.puzzle) {
            puzzle_names.push(trainer.puzzle.clone());
        }
    }
    let puzzle_count = puzzle_names.len();
    puzzle_names.sort_by_cached_key(|name| {
        (
            layout.hidden_puzzles.contains(name) && include_hidden,
            puzzle_position
                .get(name)
                .copied()
                .unwrap_or(layout.puzzle_order.len() + puzzle_count),
        )
    });

    let mut result = Vec::new();
    for puzzle in puzzle_names {
        if !include_hidden && layout.hidden_puzzles.contains(&puzzle) {
            continue;
        }
        let mut entries = trainers
            .iter()
            .filter(|trainer| trainer.puzzle == puzzle)
            .cloned()
            .collect::<Vec<_>>();
        let configured = layout.trainer_order.get(&puzzle);
        let trainer_position = positions(configured.map(Vec::as_slice).unwrap_or_default());
        let entry_count = entries.len();
        entries.sort_by_cached_key(|trainer| {
            (
                layout.hidden_trainers.contains(&trainer.trainer_id) && include_hidden,
                trainer_position
                    .get(&trainer.trainer_id)
                    .copied()
                    .unwrap_or(configured.map(Vec::len).unwrap_or(0) + entry_count),
            )
        });
        if !include_hidden {
            entries.retain(|trainer| !layout.hidden_trainers.contains(&trainer.trainer_id));
        }
        if !entries.is_empty() || include_hidden {
            result.push((puzzle, entries));
        }
    }
    result
}

pub fn capture_catalog(layout: &mut HomeLayout, trainers: &[TrainerSummary]) {
    layout.edited = true;
    let current = groups(trainers, layout, true);
    layout.puzzle_order = current.iter().map(|(puzzle, _)| puzzle.clone()).collect();
    layout.trainer_order = current
        .into_iter()
        .map(|(puzzle, trainers)| {
            (
                puzzle,
                trainers
                    .into_iter()
                    .map(|trainer| trainer.trainer_id)
                    .collect(),
            )
        })
        .collect();
    layout.known_trainers = trainers
        .iter()
        .map(|trainer| trainer.trainer_id.clone())
        .collect();
}

pub fn has_new_trainers(layout: &HomeLayout, trainers: &[TrainerSummary]) -> bool {
    layout.edited
        && trainers
            .iter()
            .any(|trainer| !layout.known_trainers.contains(&trainer.trainer_id))
}

pub fn move_puzzle(layout: &mut HomeLayout, puzzle: &str, before: &str) {
    move_before(&mut layout.puzzle_order, puzzle, before);
}

pub fn move_trainer(layout: &mut HomeLayout, puzzle: &str, trainer: &str, before: &str) {
    move_before(
        layout.trainer_order.entry(puzzle.to_owned()).or_default(),
        trainer,
        before,
    );
}

pub fn reset_to_defaults(layout: &mut HomeLayout, trainers: &[TrainerSummary]) {
    *layout = HomeLayout::default();
    capture_catalog(layout, trainers);
}

fn move_before(order: &mut Vec<String>, item: &str, before: &str) {
    if item == before {
        return;
    }
    let Some(from) = order.iter().position(|value| value == item) else {
        return;
    };
    let moved = order.remove(from);
    let to = order
        .iter()
        .position(|value| value == before)
        .unwrap_or(order.len());
    order.insert(to, moved);
}

fn positions(order: &[String]) -> BTreeMap<String, usize> {
    order
        .iter()
        .enumerate()
        .map(|(index, value)| (value.clone(), index))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trainer(id: &str, puzzle: &str) -> TrainerSummary {
        TrainerSummary {
            trainer_id: id.to_owned(),
            puzzle: puzzle.to_owned(),
            method: String::new(),
            name: id.to_owned(),
            order: 0,
            max_algs_per_row: 1,
            selection_key: String::new(),
            times_key: String::new(),
        }
    }

    #[test]
    fn new_trainers_are_detected_after_homepage_editing() {
        let trainers = vec![trainer("oll", "3x3"), trainer("pll", "3x3")];
        let mut layout = HomeLayout::default();
        assert!(!has_new_trainers(&layout, &trainers));
        capture_catalog(&mut layout, &trainers);
        assert!(!has_new_trainers(&layout, &trainers));
        let trainers = [trainers, vec![trainer("zbll", "3x3")]].concat();
        assert!(has_new_trainers(&layout, &trainers));
    }

    #[test]
    fn hidden_items_are_last_and_can_be_excluded() {
        let trainers = vec![
            trainer("oll", "3x3"),
            trainer("pll", "3x3"),
            trainer("eg", "2x2"),
        ];
        let mut layout = HomeLayout::default();
        capture_catalog(&mut layout, &trainers);
        layout.hidden_trainers.insert("oll".to_owned());
        layout.hidden_puzzles.insert("3x3".to_owned());

        let editing = groups(&trainers, &layout, true);
        assert_eq!(editing[1].0, "3x3");
        assert_eq!(editing[1].1[1].trainer_id, "oll");

        let normal = groups(&trainers, &layout, false);
        assert_eq!(normal.len(), 1);
        assert_eq!(normal[0].0, "2x2");
    }

    #[test]
    fn moving_items_updates_explicit_order() {
        let mut layout = HomeLayout {
            puzzle_order: vec!["3x3".to_owned(), "2x2".to_owned()],
            trainer_order: BTreeMap::from([(
                "3x3".to_owned(),
                vec!["oll".to_owned(), "pll".to_owned(), "zbll".to_owned()],
            )]),
            ..HomeLayout::default()
        };
        move_puzzle(&mut layout, "2x2", "3x3");
        move_trainer(&mut layout, "3x3", "zbll", "pll");
        assert_eq!(layout.puzzle_order, ["2x2", "3x3"]);
        assert_eq!(layout.trainer_order["3x3"], ["oll", "zbll", "pll"]);
    }

    #[test]
    fn layout_round_trips_through_settings() {
        let layout = HomeLayout {
            edited: true,
            puzzle_order: vec!["3x3".to_owned()],
            hidden_trainers: BTreeSet::from(["oll".to_owned()]),
            ..HomeLayout::default()
        };
        let mut settings = serde_json::json!({});
        store(&mut settings, &layout);
        assert_eq!(load(&settings), layout);
    }

    #[test]
    fn resetting_restores_defaults_and_keeps_items_reorderable() {
        let trainers = vec![
            trainer("oll", "3x3"),
            trainer("pll", "3x3"),
            trainer("eg", "2x2"),
        ];
        let mut layout = HomeLayout {
            edited: true,
            puzzle_order: vec!["3x3".to_owned(), "2x2".to_owned()],
            trainer_order: BTreeMap::from([("3x3".to_owned(), vec!["pll".to_owned()])]),
            hidden_puzzles: BTreeSet::from(["2x2".to_owned()]),
            hidden_trainers: BTreeSet::from(["oll".to_owned()]),
            known_trainers: BTreeSet::from(["oll".to_owned()]),
        };

        reset_to_defaults(&mut layout, &trainers);

        assert_eq!(layout.puzzle_order, ["3x3", "2x2"]);
        assert_eq!(layout.trainer_order["3x3"], ["oll", "pll"]);
        assert!(layout.hidden_puzzles.is_empty());
        assert!(layout.hidden_trainers.is_empty());
        assert_eq!(
            layout.known_trainers,
            BTreeSet::from(["oll".to_owned(), "pll".to_owned(), "eg".to_owned()])
        );

        move_puzzle(&mut layout, "2x2", "3x3");
        move_trainer(&mut layout, "3x3", "pll", "oll");
        assert_eq!(layout.puzzle_order, ["2x2", "3x3"]);
        assert_eq!(layout.trainer_order["3x3"], ["pll", "oll"]);
    }
}
