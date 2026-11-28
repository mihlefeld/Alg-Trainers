use crate::logic::theme::default_cube_colors;
use crate::models::{SavedSearchQueries, TrainerBundle, TrainerCase};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const SAVED_SEARCHES_SETTING: &str = "trainerSearchQueries";

pub fn saved_queries(settings: &Value, trainer_id: &str) -> SavedSearchQueries {
    let mut saved: SavedSearchQueries = settings
        .get(SAVED_SEARCHES_SETTING)
        .and_then(|trainers| trainers.get(trainer_id))
        .cloned()
        .and_then(|value| serde_json::from_value::<SavedSearchQueries>(value).ok())
        .unwrap_or_default();
    if saved
        .default_query
        .as_ref()
        .is_some_and(|name| !saved.queries.contains_key(name))
    {
        saved.default_query = None;
    }
    saved
}

pub fn store_saved_queries(settings: &mut Value, trainer_id: &str, saved: SavedSearchQueries) {
    if !settings.is_object() {
        *settings = serde_json::json!({});
    }
    if !settings
        .get(SAVED_SEARCHES_SETTING)
        .is_some_and(Value::is_object)
    {
        settings[SAVED_SEARCHES_SETTING] = serde_json::json!({});
    }
    settings[SAVED_SEARCHES_SETTING][trainer_id] =
        serde_json::to_value(saved).unwrap_or_else(|_| serde_json::json!({}));
}

pub fn case_matches_search(
    case: &TrainerCase,
    bundle: &TrainerBundle,
    needle: &str,
    cells: &BTreeMap<usize, String>,
    all_angles: bool,
) -> bool {
    if !needle.is_empty() && !case_matches_text(case, bundle, needle) {
        return false;
    }
    if cells.is_empty() {
        return true;
    }
    let fills = svg_polygon_fills(case.image_svg.as_deref().unwrap_or(""));
    let angles = if all_angles && !bundle.u_move_cycles.is_empty() {
        case_angles(&fills, &bundle.u_move_cycles)
    } else {
        vec![fills]
    };
    angles.iter().any(|fills| fills_match_pattern(fills, cells))
}

pub fn case_matches_text(case: &TrainerCase, bundle: &TrainerBundle, needle: &str) -> bool {
    let mut terms = vec![
        case.case_id.as_str(),
        case.name.as_str(),
        case.setup.as_deref().unwrap_or(""),
    ];
    terms.extend(case.algs.iter().map(|item| item.alg.as_str()));
    terms.extend(
        bundle
            .groups
            .iter()
            .filter(|group| group.cases.contains(&case.case_id))
            .map(|group| group.name.as_str()),
    );
    terms.extend(
        bundle
            .algsets
            .iter()
            .filter(|algset| {
                bundle
                    .groups
                    .iter()
                    .filter(|group| group.cases.contains(&case.case_id))
                    .any(|group| algset.groups.contains(&group.name))
            })
            .map(|algset| algset.name.as_str()),
    );
    let haystack = terms.join(" ").to_lowercase();
    haystack.contains(needle) || fuzzy_matches(needle, &haystack)
}

pub fn fuzzy_matches(needle: &str, haystack: &str) -> bool {
    let mut remaining = haystack
        .chars()
        .filter(|character| !character.is_whitespace());
    needle
        .chars()
        .filter(|character| !character.is_whitespace())
        .all(|wanted| remaining.any(|candidate| candidate == wanted))
}

pub fn svg_polygon_fills(svg: &str) -> Vec<String> {
    let mut fills = Vec::new();
    let mut remaining = svg;
    while let Some(start) = remaining.find("<polygon") {
        remaining = &remaining[start..];
        let Some(end) = remaining.find('>') else {
            break;
        };
        let tag = &remaining[..=end];
        if let Some(fill) = svg_attribute(tag, "fill") {
            fills.push(fill.to_lowercase());
        }
        remaining = &remaining[end + 1..];
    }
    fills
}

fn svg_attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("{name}=");
    let start = tag.find(&marker)? + marker.len();
    let tail = &tag[start..];
    let quote = tail.chars().next()?;
    if quote != '"' && quote != '\'' {
        return tail.split_whitespace().next();
    }
    let value = tail.get(quote.len_utf8()..)?;
    let end = value.find(quote)?;
    value.get(..end)
}

pub fn fills_match_pattern(fills: &[String], cells: &BTreeMap<usize, String>) -> bool {
    let mut label_colors = BTreeMap::<&str, &str>::new();
    let mut used_colors = BTreeSet::new();
    for (index, label) in cells {
        let Some(fill) = fills.get(*index).map(String::as_str) else {
            return false;
        };
        if let Some(expected) = label_colors.get(label.as_str()) {
            if *expected != fill {
                return false;
            }
        } else {
            if !used_colors.insert(fill) {
                return false;
            }
            label_colors.insert(label.as_str(), fill);
        }
    }
    true
}

fn apply_u_move(fills: &[String], cycles: &[Vec<usize>]) -> Vec<String> {
    let mut moved = fills.to_vec();
    for cycle in cycles {
        if cycle.is_empty() {
            continue;
        }
        for (index, source) in cycle.iter().enumerate() {
            let destination = cycle[(index + 1) % cycle.len()];
            if let (Some(fill), Some(target)) = (fills.get(*source), moved.get_mut(destination)) {
                *target = fill.clone();
            }
        }
    }
    moved
}

fn gcd(mut left: usize, mut right: usize) -> usize {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn case_angles(fills: &[String], cycles: &[Vec<usize>]) -> Vec<Vec<String>> {
    let order = cycles.iter().fold(1_usize, |order, cycle| {
        if cycle.is_empty() {
            order
        } else {
            order.saturating_mul(cycle.len()) / gcd(order, cycle.len())
        }
    });
    let mut angles = Vec::with_capacity(order);
    angles.push(fills.to_vec());
    for _ in 1..order {
        angles.push(apply_u_move(angles.last().unwrap(), cycles));
    }
    angles
}

pub fn search_color_names() -> [&'static str; 12] {
    [
        "white",
        "ignore",
        "lightblue",
        "blue",
        "red",
        "pink",
        "beige",
        "yellow",
        "green",
        "lightgreen",
        "purple",
        "orange",
    ]
}

pub fn cube_color(colors: &BTreeMap<String, String>, name: &str) -> String {
    colors
        .get(name)
        .map(String::as_str)
        .or_else(|| default_cube_colors().get(name).copied())
        .unwrap_or("#222222")
        .to_owned()
}

pub fn search_template_svg(
    bundle: &TrainerBundle,
    cells: &BTreeMap<usize, String>,
    colors: &BTreeMap<String, String>,
) -> Option<String> {
    let svg = bundle
        .cases
        .iter()
        .filter_map(|case| case.image_svg.as_deref())
        .max_by_key(|svg| svg_polygon_fills(svg).len())?;
    let mut rendered = String::with_capacity(svg.len() + 160);
    let mut remaining = svg;
    let mut index = 0;
    while let Some(start) = remaining.find("<polygon") {
        rendered.push_str(&remaining[..start]);
        remaining = &remaining[start..];
        let end = remaining.find('>')?;
        let tag = &remaining[..=end];
        let fill = cells
            .get(&index)
            .map(|name| cube_color(colors, name))
            .unwrap_or_else(|| "#222222".to_owned());
        let mut replacement = tag
            .strip_prefix("<polygon")
            .unwrap_or(tag)
            .trim_end_matches('>')
            .to_owned();
        if let Some(value) = svg_attribute(tag, "fill") {
            let marker = format!(
                "fill={quote}{value}{quote}",
                quote = if tag.contains(&format!("fill=\"{value}\"")) {
                    '"'
                } else {
                    '\''
                }
            );
            replacement = replacement.replace(&marker, &format!("fill='{fill}'"));
        } else {
            replacement.push_str(&format!(" fill='{fill}'"));
        }
        rendered.push_str(&format!(
            "<polygon data-search-index='{index}' cursor='pointer'{replacement}>"
        ));
        index += 1;
        remaining = &remaining[end + 1..];
    }
    rendered.push_str(remaining);
    Some(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_matching_requires_bijective_color_mapping() {
        let fills = vec![
            "#e15759".to_owned(),
            "#e15759".to_owned(),
            "#4e79a7".to_owned(),
        ];
        let matching = BTreeMap::from([
            (0, "red".to_owned()),
            (1, "red".to_owned()),
            (2, "blue".to_owned()),
        ]);
        let conflicting = BTreeMap::from([(0, "red".to_owned()), (1, "blue".to_owned())]);
        assert!(fills_match_pattern(&fills, &matching));
        assert!(!fills_match_pattern(&fills, &conflicting));
    }

    #[test]
    fn fuzzy_match_ignores_whitespace() {
        assert!(fuzzy_matches("ru'", "r u r' u'"));
        assert!(!fuzzy_matches("xyz", "r u r' u'"));
    }

    #[test]
    fn saved_queries_round_trip_and_ignore_missing_defaults() {
        let query = crate::models::SavedSearchQuery {
            query: "T perm".to_owned(),
            cells: BTreeMap::from([(2, "red".to_owned())]),
            all_angles: false,
        };
        let saved = SavedSearchQueries {
            queries: BTreeMap::from([("T".to_owned(), query.clone())]),
            default_query: Some("T".to_owned()),
        };
        let mut settings = serde_json::json!({"baseSize": 1.2});
        store_saved_queries(&mut settings, "pll", saved);

        let loaded = saved_queries(&settings, "pll");
        assert_eq!(loaded.queries["T"], query);
        assert_eq!(loaded.default_query.as_deref(), Some("T"));

        settings[SAVED_SEARCHES_SETTING]["pll"]["defaultQuery"] = serde_json::json!("missing");
        assert_eq!(saved_queries(&settings, "pll").default_query, None);
    }
}
