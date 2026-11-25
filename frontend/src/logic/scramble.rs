use crate::models::{StoredResult, TrainerBundle, TrainerCase};
use js_sys::Math;
use std::collections::BTreeMap;

pub fn random_index(len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (Math::random() * len as f64).floor() as usize
    }
}

pub fn pick_recap_case(selected: &[String]) -> (Option<String>, Vec<String>) {
    let mut recap = selected.to_vec();
    let case_id = (!recap.is_empty()).then(|| recap.remove(random_index(recap.len())));
    (case_id, recap)
}

pub fn pick_initial_case_for_query(
    query: &str,
    selected: &[String],
    results: &[StoredResult],
    settings: &serde_json::Value,
) -> (Option<String>, Vec<String>, bool) {
    match query {
        "?recap" => {
            let (case_id, remaining) = pick_recap_case(selected);
            (case_id, remaining, true)
        }
        "?train" => (
            weighted_case(selected, results, settings),
            Vec::new(),
            false,
        ),
        _ => (None, Vec::new(), false),
    }
}

pub fn weighted_case(
    selected: &[String],
    results: &[StoredResult],
    settings: &serde_json::Value,
) -> Option<String> {
    if selected.is_empty() {
        return None;
    }

    let count_weighting = settings
        .get("weightedChoice")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let time_weighting = settings
        .get("weightedChoiceTime")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let std_weighting = settings
        .get("weightedChoiceStd")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !count_weighting && !time_weighting && !std_weighting {
        return selected.get(random_index(selected.len())).cloned();
    }

    let metrics = selected
        .iter()
        .map(|case_id| {
            let times = results
                .iter()
                .filter(|result| &result.case == case_id)
                .map(|result| result.ms as f64 / 1000.0)
                .collect::<Vec<_>>();
            let count = if count_weighting {
                (times.len() as f64 + 0.1).powf(-0.8)
            } else {
                1.0
            };
            let mean = if time_weighting {
                times.iter().copied().sum::<f64>() / times.len().max(1) as f64
            } else {
                1.0
            };
            let std = if std_weighting && times.len() > 1 {
                let mean = times.iter().copied().sum::<f64>() / times.len() as f64;
                (times.iter().map(|time| (time - mean).powi(2)).sum::<f64>() / times.len() as f64)
                    .sqrt()
            } else {
                1.0
            };
            count * mean.max(0.0).powf(if time_weighting { 0.8 } else { 0.0 }) * std.max(0.0)
        })
        .collect::<Vec<_>>();
    let maximum = metrics.iter().copied().fold(0.0_f64, f64::max);
    let weights = if maximum > 0.0 {
        let floor = maximum / 5.0;
        metrics
            .iter()
            .map(|weight| weight.max(floor))
            .collect::<Vec<_>>()
    } else {
        vec![1.0; selected.len()]
    };
    let total = weights.iter().sum::<f64>();
    let mut target = Math::random() * total;
    for (index, weight) in weights.into_iter().enumerate() {
        if target < weight {
            return selected.get(index).cloned();
        }
        target -= weight;
    }
    selected.last().cloned()
}

pub fn case_probabilities(
    selected: &[String],
    results: &[StoredResult],
    settings: &serde_json::Value,
) -> BTreeMap<String, f64> {
    if selected.is_empty() {
        return BTreeMap::new();
    }
    let weights = selected
        .iter()
        .map(|case_id| {
            let case_results = results
                .iter()
                .filter(|result| &result.case == case_id)
                .collect::<Vec<_>>();
            let mut weight = 1.0_f64;
            if settings
                .get("weightedChoice")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true)
            {
                weight *= (case_results.len() as f64 + 0.1).powf(-0.8);
            }
            if settings
                .get("weightedChoiceTime")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                let mean = case_results
                    .iter()
                    .map(|result| result.ms as f64)
                    .sum::<f64>()
                    / case_results.len().max(1) as f64;
                weight *= mean.max(1.0).powf(0.8);
            }
            if settings
                .get("weightedChoiceStd")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
                && case_results.len() > 1
            {
                let mean = case_results
                    .iter()
                    .map(|result| result.ms as f64)
                    .sum::<f64>()
                    / case_results.len() as f64;
                let std = (case_results
                    .iter()
                    .map(|result| (result.ms as f64 - mean).powi(2))
                    .sum::<f64>()
                    / case_results.len() as f64)
                    .sqrt();
                weight *= std.max(1.0);
            }
            weight
        })
        .collect::<Vec<_>>();
    let total = weights.iter().sum::<f64>();
    selected
        .iter()
        .cloned()
        .zip(weights.into_iter().map(|weight| {
            if total > 0.0 {
                weight / total
            } else {
                1.0 / selected.len() as f64
            }
        }))
        .collect()
}

fn random_option(options: &[String]) -> &str {
    options
        .get(random_index(options.len()))
        .map(String::as_str)
        .unwrap_or("")
}

pub fn make_scramble(
    case: &TrainerCase,
    bundle: &TrainerBundle,
    settings: &serde_json::Value,
    selection_key: &str,
) -> String {
    let scramble = case
        .scrambles
        .get(random_index(case.scrambles.len()))
        .map(String::as_str)
        .unwrap_or("");
    let override_options = |key: &str, defaults: &[String]| {
        settings
            .get("trainerScrambleOverrides")
            .and_then(|value| value.get(selection_key))
            .and_then(|value| value.get(key))
            .and_then(serde_json::Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .filter(|values| !values.is_empty())
            .unwrap_or_else(|| defaults.to_vec())
    };
    let pre_rotations = override_options("preRotations", &bundle.pre_rotations);
    let post_rotations = override_options("postRotations", &bundle.post_rotations);
    let pre_moves = override_options("preMoves", &bundle.pre_moves);
    let post_moves = override_options("postMoves", &bundle.post_moves);
    let chosen_scramble = scramble.to_owned();
    if bundle.title.contains("BLD") {
        let default_scheme = "abcdefghijklmnopqrstuvwx";
        let custom_scheme = settings
            .get(if bundle.title.contains("UFR") {
                "letterSchemeCorners"
            } else {
                "letterSchemeEdges"
            })
            .and_then(serde_json::Value::as_str)
            .unwrap_or(default_scheme);
        let letters = chosen_scramble
            .chars()
            .map(|letter| {
                let index = default_scheme
                    .to_lowercase()
                    .find(letter.to_ascii_lowercase());
                index
                    .and_then(|index| custom_scheme.chars().nth(index))
                    .unwrap_or(letter)
            })
            .map(|letter| letter.to_ascii_uppercase())
            .collect::<String>();
        let algset = case_algset(case, bundle).unwrap_or_else(|| bundle.title.clone());
        let words_enabled = settings
            .get(if bundle.title.contains("UFR") {
                "useCornersLetterPairs"
            } else {
                "useEdgesLetterPairs"
            })
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if words_enabled {
            let word = settings
                .get("letterpairs")
                .and_then(|pairs| pairs.get(&letters))
                .and_then(serde_json::Value::as_str);
            if let Some(word) = word {
                return format!("({algset}) {word} ({letters})");
            }
        }
        return format!("({algset}) {letters}");
    }
    if bundle.title.starts_with("Square-1") {
        return make_square1_scramble(
            &chosen_scramble,
            random_option(&pre_moves),
            random_option(&post_moves),
            bundle.title.contains("OBL"),
        );
    }
    let prefix = [random_option(&pre_rotations), random_option(&pre_moves)]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let suffix = [random_option(&post_moves), random_option(&post_rotations)]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    [prefix.as_str(), scramble, suffix.as_str()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn case_algset(case: &TrainerCase, bundle: &TrainerBundle) -> Option<String> {
    bundle
        .groups
        .iter()
        .find(|group| group.cases.contains(&case.case_id))
        .and_then(|group| {
            bundle
                .algsets
                .iter()
                .find(|algset| algset.groups.contains(&group.name))
                .map(|algset| algset.name.clone())
        })
}

fn make_square1_scramble(scramble: &str, pre_move: &str, post_move: &str, obl: bool) -> String {
    let Some(parts) = scramble
        .split('/')
        .map(str::trim)
        .map(parse_square_pair)
        .collect::<Option<Vec<_>>>()
    else {
        return scramble.to_owned();
    };
    let mut parts = parts;
    let flip_slice = Math::random() > 0.5;
    if obl && flip_slice && parts.len() > 1 {
        parts[1].0 += 6;
        parts[1].1 += 6;
        normalize_square_pair(&mut parts[1]);
    } else if flip_slice != (parts.len() % 2 == 0) && parts.len() >= 2 {
        let index = parts.len() - 2;
        parts[index].0 += 6;
        normalize_square_pair(&mut parts[index]);
        parts.insert(index + 1, (6, 0));
    }
    if let (Some(first), Some(pre)) = (parts.first_mut(), parse_square_pair(pre_move)) {
        first.0 += pre.0;
        first.1 += pre.1;
        normalize_square_pair(first);
    }
    if let (Some(last), Some(post)) = (parts.last_mut(), parse_square_pair(post_move)) {
        last.0 += post.0;
        last.1 += post.1;
        normalize_square_pair(last);
    }
    parts
        .iter()
        .map(|pair| format!("{},{}", pair.0, pair.1))
        .collect::<Vec<_>>()
        .join(" / ")
}

fn parse_square_pair(value: &str) -> Option<(i32, i32)> {
    let (upper, lower) = value.trim().split_once(',')?;
    Some((upper.trim().parse().ok()?, lower.trim().parse().ok()?))
}

fn normalize_square_pair(pair: &mut (i32, i32)) {
    while pair.0 > 6 {
        pair.0 -= 12;
    }
    while pair.0 < -6 {
        pair.0 += 12;
    }
    while pair.1 > 6 {
        pair.1 -= 12;
    }
    while pair.1 < -6 {
        pair.1 += 12;
    }
}
