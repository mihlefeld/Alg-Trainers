use crate::models::{ResultSync, StoredResult};
use js_sys::Math;

pub fn format_time(ms: i64) -> String {
    if ms >= 60_000 {
        format!(
            "{}:{:02}.{:02}",
            ms / 60_000,
            (ms / 1000) % 60,
            (ms / 10) % 100
        )
    } else {
        format!("{}.{:02}", ms / 1000, (ms / 10) % 100)
    }
}

pub fn exponential_moving_average(values: &[f64], factor: f64) -> Vec<f64> {
    values
        .iter()
        .copied()
        .scan(None, |previous, value| {
            let smoothed = match *previous {
                Some(previous) => previous * (1.0 - factor) + value * factor,
                None => value,
            };
            *previous = Some(smoothed);
            Some(smoothed)
        })
        .collect()
}

pub fn parse_stored_results(key: &str, value: &str) -> Option<(Vec<StoredResult>, bool)> {
    let mut results = serde_json::from_str::<Vec<StoredResult>>(value).ok()?;
    let mut changed = false;
    for (index, result) in &mut results.iter_mut().enumerate() {
        if result.id.is_empty() {
            result.id = stable_legacy_uuid(key, index, result.ms, &result.case);
            changed = true;
        }
        if result.datetime.is_empty() {
            result.datetime = legacy_result_datetime(index);
            changed = true;
        }
        if result.time.is_empty() {
            result.time = format_time(result.ms);
            changed = true;
        }
    }
    results.sort_by(compare_results);
    Some((results, changed))
}

pub fn stable_legacy_uuid(key: &str, index: usize, milliseconds: i64, case: &str) -> String {
    let canonical = format!("{}\0{}\0{}\0{}", key, index, milliseconds, case);
    let mut hash = 0x811c9dc5_u32;
    for byte in canonical.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    let mut bytes = [0_u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        hash ^= (index as u32).wrapping_add(0x9e3779b9);
        hash = hash.wrapping_mul(0x01000193);
        *byte = hash as u8;
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

pub fn legacy_result_datetime(index: usize) -> String {
    let seconds = (index % 86_400) as i64;
    let (year, month, day) = civil_date_from_days_since_1970(10_957 + index as i64 / 86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.000Z",
        seconds / 3_600,
        seconds % 3_600 / 60,
        seconds % 60
    )
}

fn civil_date_from_days_since_1970(mut days: i64) -> (i64, i64, i64) {
    days += 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    (year, month, day)
}

pub fn compare_results(left: &StoredResult, right: &StoredResult) -> std::cmp::Ordering {
    left.datetime
        .cmp(&right.datetime)
        .then_with(|| left.id.cmp(&right.id))
}

pub fn merge_remote_result(mut merged: Vec<StoredResult>, result: ResultSync) -> Vec<StoredResult> {
    if result.deleted {
        merged.retain(|existing| existing.id != result.id);
    } else {
        let stored = StoredResult {
            id: result.id,
            datetime: result.datetime,
            ms: result.result_ms,
            case: result.case_id,
            time: format_time(result.result_ms),
            deleted: false,
        };
        if let Some(existing) = merged.iter_mut().find(|existing| existing.id == stored.id) {
            *existing = stored;
        } else {
            merged.push(stored);
        }
    }
    merged.sort_by(compare_results);
    merged
}

pub fn make_uuid() -> String {
    let mut bytes = [0_u8; 16];
    for byte in &mut bytes {
        *byte = (Math::random() * 256.0) as u8;
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaseStatsSummary {
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub histogram_counts: Vec<usize>,
    pub max_histogram_count: usize,
    pub plot_points: String,
    pub trend_points: String,
}

pub fn compute_case_stats(case_stats: &[&StoredResult]) -> CaseStatsSummary {
    let times = case_stats
        .iter()
        .map(|result| result.ms as f64 / 1000.0)
        .collect::<Vec<_>>();
    let mean = if times.is_empty() {
        0.0
    } else {
        times.iter().sum::<f64>() / times.len() as f64
    };
    let raw_min = times.iter().copied().fold(f64::INFINITY, f64::min);
    let raw_max = times.iter().copied().fold(0.0_f64, f64::max);
    let range = if raw_max > raw_min {
        raw_max - raw_min
    } else {
        1.0
    };
    let bin_count = 10usize.min(times.len().max(1));
    let mut counts = vec![0usize; bin_count];
    for time in &times {
        let index =
            (((*time - raw_min) / range * bin_count as f64).floor() as usize).min(bin_count - 1);
        counts[index] += 1;
    }
    let max_count = counts.iter().copied().max().unwrap_or(1).max(1);
    let point_width = 280.0;
    let plot_max = if raw_max > 0.0 { raw_max } else { 1.0 };
    let to_polyline = |values: &[f64]| {
        values
            .iter()
            .enumerate()
            .map(|(index, time)| {
                let x = if times.len() > 1 {
                    index as f64 * point_width / (times.len() - 1) as f64
                } else {
                    point_width / 2.0
                };
                format!("{x:.1},{:.1}", 120.0 - (*time / plot_max * 100.0))
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let plot_points = to_polyline(&times);
    let trend = exponential_moving_average(&times, 0.2);
    let trend_points = to_polyline(&trend);

    CaseStatsSummary {
        count: times.len(),
        mean,
        min: if times.is_empty() { 0.0 } else { raw_min },
        max: if times.is_empty() { 0.0 } else { raw_max },
        histogram_counts: counts,
        max_histogram_count: max_count,
        plot_points,
        trend_points,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_result_identity_and_sequence_are_deterministic() {
        let value = r#"[{"ms":3100,"case":7},{"ms":4890,"case":4}]"#;
        let (first, _) = parse_stored_results("sq1cpepTimesArray", value).expect("first parse");
        let (second, _) = parse_stored_results("sq1cpepTimesArray", value).expect("second parse");

        assert_eq!(first[0].id, "01612ed6-71dd-46f2-89d1-56d6e9fd4ef2");
        assert_eq!(
            first.iter().map(|result| &result.id).collect::<Vec<_>>(),
            second.iter().map(|result| &result.id).collect::<Vec<_>>()
        );
        assert_eq!(first[0].datetime, "2000-01-01T00:00:00.000Z");
        assert_eq!(first[1].datetime, "2000-01-01T00:00:01.000Z");
        assert_eq!(legacy_result_datetime(86_400), "2000-01-02T00:00:00.000Z");
    }

    #[test]
    fn legacy_result_identity_uses_canonical_fields() {
        let first = r#"[{"ms":3100,"case":7,"details":"first","index":0}]"#;
        let second =
            r#"[{"ms":3100,"case":"7","time":"3.10","details":"second","id":"","datetime":""}]"#;
        let (first, _) = parse_stored_results("results", first).expect("first parse");
        let (second, _) = parse_stored_results("results", second).expect("second parse");

        assert_eq!(first[0].id, second[0].id);
    }

    #[test]
    fn stored_results_sort_oldest_first_with_id_tiebreaker() {
        let mut results = vec![
            StoredResult {
                id: "b".to_owned(),
                datetime: "2026-01-02T00:00:00Z".to_owned(),
                ms: 2,
                case: "1".to_owned(),
                time: "0.002".to_owned(),
                deleted: false,
            },
            StoredResult {
                id: "a".to_owned(),
                datetime: "2026-01-01T00:00:00Z".to_owned(),
                ms: 1,
                case: "1".to_owned(),
                time: "0.001".to_owned(),
                deleted: false,
            },
        ];
        results.sort_by(compare_results);

        assert_eq!(
            results
                .iter()
                .map(|result| result.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
    }

    #[test]
    fn remote_tombstones_remove_local_results() {
        let result = StoredResult {
            id: "result".to_owned(),
            datetime: "2026-01-01T00:00:00Z".to_owned(),
            ms: 1000,
            case: "1".to_owned(),
            time: "1.00".to_owned(),
            deleted: false,
        };
        let tombstone = ResultSync {
            id: result.id.clone(),
            datetime: result.datetime.clone(),
            result_ms: result.ms,
            trainer_id: "trainer".to_owned(),
            case_id: result.case.clone(),
            updated_at: "2026-01-02T00:00:00Z".to_owned(),
            deleted: true,
        };

        assert!(merge_remote_result(vec![result], tombstone).is_empty());
    }

    #[test]
    fn reads_legacy_results_with_numeric_case_ids() {
        let result = serde_json::from_str::<StoredResult>(
            r#"{"time":"3.10","ms":3100,"details":"scramble","index":0,"case":7}"#,
        )
        .expect("legacy result");
        assert_eq!(result.ms, 3100);
        assert_eq!(result.case, "7");
        assert_eq!(result.time, "3.10");
        assert!(result.id.is_empty());
        assert!(result.datetime.is_empty());
    }

    #[test]
    fn computes_case_stats_summary() {
        let items = vec![
            StoredResult {
                id: "1".to_owned(),
                datetime: "2026-01-01T00:00:00Z".to_owned(),
                ms: 1000,
                case: "c".to_owned(),
                time: "1.00".to_owned(),
                deleted: false,
            },
            StoredResult {
                id: "2".to_owned(),
                datetime: "2026-01-01T00:00:01Z".to_owned(),
                ms: 3000,
                case: "c".to_owned(),
                time: "3.00".to_owned(),
                deleted: false,
            },
        ];
        let refs = items.iter().collect::<Vec<_>>();
        let summary = compute_case_stats(&refs);
        assert_eq!(summary.count, 2);
        assert!((summary.mean - 2.0).abs() < 1e-9);
        assert!((summary.min - 1.0).abs() < 1e-9);
        assert!((summary.max - 3.0).abs() < 1e-9);
        assert_eq!(summary.histogram_counts, vec![1, 1]);
        assert!(!summary.plot_points.is_empty());
        assert!(!summary.trend_points.is_empty());
    }
}
