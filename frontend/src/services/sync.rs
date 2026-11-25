use crate::logic::{algs, results};
use crate::models::{
    MeResponse, ResultSync, SelectionSync, SettingsSync, SyncBundle, TrainerSummary, UserAlgState,
    UserAlgSync,
};
use crate::services::api::save_cached_account;
use crate::services::storage::{
    get_stored, put_stored, storageAll, storageClearDirtyIfUnchanged, SETTINGS_KEY,
};
use gloo_net::http::Request;
use gloo_timers::callback::{Interval, Timeout};
use js_sys::Date;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

pub const SYNC_RESULT_BATCH_SIZE: usize = 10_000;

thread_local! {
    static SYNC_STARTED: RefCell<bool> = const { RefCell::new(false) };
    static SYNC_INTERVAL: RefCell<Option<Interval>> = const { RefCell::new(None) };
    static SYNC_DEBOUNCE: RefCell<Option<Timeout>> = const { RefCell::new(None) };
    static SYNC_IN_FLIGHT: RefCell<bool> = const { RefCell::new(false) };
    static SYNC_PENDING: RefCell<Option<bool>> = const { RefCell::new(None) };
    static LAST_SYNC: RefCell<f64> = const { RefCell::new(0.0) };
    static LAST_SYNC_COMPLETED: RefCell<Option<f64>> = const { RefCell::new(None) };
}

pub fn last_sync_label() -> String {
    LAST_SYNC_COMPLETED
        .with(|last| *last.borrow())
        .map(|timestamp| {
            Date::new(&JsValue::from_f64(timestamp))
                .to_locale_string("en-GB", &JsValue::UNDEFINED)
                .as_string()
                .unwrap_or_else(|| "Unknown".to_owned())
        })
        .unwrap_or_else(|| "Not yet".to_owned())
}

pub fn split_sync_batches(mut bundle: SyncBundle) -> Vec<SyncBundle> {
    let items = std::mem::take(&mut bundle.results);
    if items.is_empty() {
        return vec![bundle];
    }
    items
        .chunks(SYNC_RESULT_BATCH_SIZE)
        .enumerate()
        .map(|(index, chunk)| {
            let mut batch = if index == 0 {
                bundle.clone()
            } else {
                SyncBundle::default()
            };
            batch.results = chunk.to_vec();
            batch
        })
        .collect()
}

pub fn start_sync_engine() {
    SYNC_STARTED.with(|started| {
        if *started.borrow() {
            return;
        }
        *started.borrow_mut() = true;
        let sync_interval = Interval::new(60_000, || {
            spawn_local(async move {
                sync_now(false).await;
            })
        });
        SYNC_INTERVAL.with(|timer| *timer.borrow_mut() = Some(sync_interval));
        Timeout::new(0, || {
            spawn_local(async move {
                sync_now(false).await;
            })
        })
        .forget();
        if let Some(window) = web_sys::window() {
            let closure = Closure::<dyn FnMut()>::new(|| {
                SYNC_DEBOUNCE.with(|timer| {
                    if let Some(existing) = timer.borrow_mut().take() {
                        existing.cancel();
                    }
                    *timer.borrow_mut() = Some(Timeout::new(2_000, || {
                        spawn_local(async move {
                            sync_now(false).await;
                        });
                    }));
                });
            });
            let _ = window.add_event_listener_with_callback(
                "alg-trainers-change",
                closure.as_ref().unchecked_ref(),
            );
            closure.forget();

            let online_closure = Closure::<dyn FnMut()>::new(|| {
                spawn_local(async move {
                    sync_now(true).await;
                });
            });
            let _ = window.add_event_listener_with_callback(
                "online",
                online_closure.as_ref().unchecked_ref(),
            );
            online_closure.forget();

            if let Some(document) = window.document() {
                let visibility_document = document.clone();
                let visibility_closure = Closure::<dyn FnMut()>::new(move || {
                    if !visibility_document.hidden() {
                        spawn_local(async move {
                            sync_now(false).await;
                        });
                    }
                });
                let _ = document.add_event_listener_with_callback(
                    "visibilitychange",
                    visibility_closure.as_ref().unchecked_ref(),
                );
                visibility_closure.forget();
            }
        }
    });
}

fn record_sync_completed() {
    LAST_SYNC_COMPLETED.with(|last| *last.borrow_mut() = Some(Date::now()));
    if let Some(window) = web_sys::window() {
        if let Ok(event) = web_sys::Event::new("alg-trainers-sync") {
            let _ = window.dispatch_event(&event);
        }
    }
}

pub async fn sync_now(force: bool) {
    let already_running = SYNC_IN_FLIGHT.with(|in_flight| {
        if *in_flight.borrow() {
            SYNC_PENDING.with(|pending| {
                let mut pending = pending.borrow_mut();
                *pending = Some(pending.unwrap_or(false) || force);
            });
            true
        } else {
            *in_flight.borrow_mut() = true;
            false
        }
    });
    if already_running {
        return;
    }

    let mut current_force = force;
    loop {
        sync_once(current_force).await;
        let next = SYNC_PENDING.with(|pending| pending.borrow_mut().take());
        match next {
            Some(next_force) => current_force = next_force,
            None => break,
        }
    }

    SYNC_IN_FLIGHT.with(|in_flight| *in_flight.borrow_mut() = false);
}

async fn sync_once(force: bool) {
    let now = Date::now();
    let too_soon = !force && LAST_SYNC.with(|last| now - *last.borrow() < 10_000.0);
    if too_soon {
        return;
    }
    LAST_SYNC.with(|last| *last.borrow_mut() = now);
    let Ok(me_response) = Request::get("/api/me").send().await else {
        return;
    };
    if !me_response.ok() {
        return;
    }
    let Ok(me) = me_response.json::<MeResponse>().await else {
        return;
    };
    save_cached_account(&me).await;
    if !me.authenticated {
        return;
    }
    let Ok(promise) = storageAll("dirty") else {
        return;
    };
    let Ok(dirty_value) = JsFuture::from(promise).await else {
        return;
    };
    let Ok(mut dirty) = serde_wasm_bindgen::from_value::<BTreeMap<String, f64>>(dirty_value) else {
        return;
    };
    if dirty.is_empty() {
        if !normalize_legacy_results_in_storage().await {
            if pull_remote().await {
                record_sync_completed();
            }
            return;
        }
        let Ok(promise) = storageAll("dirty") else {
            return;
        };
        let Ok(dirty_value) = JsFuture::from(promise).await else {
            return;
        };
        let Ok(refreshed) = serde_wasm_bindgen::from_value::<BTreeMap<String, f64>>(dirty_value)
        else {
            return;
        };
        dirty = refreshed;
        if dirty.is_empty() {
            if pull_remote().await {
                record_sync_completed();
            }
            return;
        }
    }
    let trainers = match Request::get("/api/trainers").send().await {
        Ok(response) if response.ok() => match response.json::<Vec<TrainerSummary>>().await {
            Ok(trainers) if !trainers.is_empty() => trainers,
            _ => return,
        },
        _ => return,
    };
    let Ok(promise) = storageAll("data") else {
        return;
    };
    let Ok(data_value) = JsFuture::from(promise).await else {
        return;
    };
    let Ok(data) = serde_wasm_bindgen::from_value::<BTreeMap<String, String>>(data_value) else {
        return;
    };
    let mut bundle = SyncBundle::default();
    for key in dirty.keys() {
        let updated_at = Date::new(&JsValue::from_f64(*dirty.get(key).unwrap_or(&0.0)))
            .to_iso_string()
            .as_string()
            .unwrap_or_else(|| {
                Date::new_0()
                    .to_iso_string()
                    .as_string()
                    .unwrap_or_default()
            });
        let Some(value) = data.get(key) else {
            continue;
        };
        if key == SETTINGS_KEY {
            if let Ok(settings_json) = serde_json::from_str(value) {
                bundle.settings = Some(SettingsSync {
                    settings_json,
                    updated_at: updated_at.clone(),
                });
            }
        } else if let Some(trainer) = trainers.iter().find(|trainer| {
            trainer.selection_key == *key
                || format!("{}Presets", trainer.selection_key) == *key
                || format!("{}Algsets", trainer.selection_key) == *key
        }) {
            let cases = get_stored(&trainer.selection_key)
                .await
                .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
                .unwrap_or_else(|| serde_json::json!([]));
            let presets = get_stored(&format!("{}Presets", trainer.selection_key))
                .await
                .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
                .unwrap_or_else(|| serde_json::json!({}));
            let algsets = get_stored(&format!("{}Algsets", trainer.selection_key))
                .await
                .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
                .unwrap_or_else(|| serde_json::json!({}));
            bundle.selections.push(SelectionSync {
                trainer_id: trainer.trainer_id.clone(),
                selection_json: serde_json::json!({
                    "cases": cases,
                    "presets": presets,
                    "algsets": algsets
                }),
                updated_at: updated_at.clone(),
            });
        } else if let Some(trainer) = trainers
            .iter()
            .find(|trainer| format!("{}customAlgs", trainer.selection_key) == *key)
        {
            if let Ok(parsed) = serde_json::from_str::<BTreeMap<String, serde_json::Value>>(value) {
                for (case_id, value) in parsed {
                    for alg in algs::parse_custom_algs(&value, &case_id) {
                        bundle.user_algs.push(UserAlgSync {
                            trainer_id: trainer.trainer_id.clone(),
                            case_id: case_id.clone(),
                            alg_id: alg.alg_id,
                            alg: alg.alg,
                            position: alg.position,
                            hidden: alg.hidden,
                            deleted: alg.deleted,
                            updated_at: updated_at.clone(),
                        });
                    }
                }
            }
        } else if let Some(trainer) = trainers.iter().find(|trainer| trainer.times_key == *key) {
            if let Some((parsed_results, changed)) = results::parse_stored_results(key, value) {
                if changed {
                    put_stored(
                        key.clone(),
                        serde_json::to_string(&parsed_results).unwrap_or_else(|_| "[]".to_owned()),
                    )
                    .await;
                }
                bundle
                    .results
                    .extend(parsed_results.into_iter().map(|result| ResultSync {
                        id: result.id,
                        datetime: result.datetime,
                        result_ms: result.ms,
                        trainer_id: trainer.trainer_id.clone(),
                        case_id: result.case,
                        updated_at: updated_at.clone(),
                        deleted: result.deleted,
                    }));
            }
        }
    }
    for batch in split_sync_batches(bundle) {
        let Ok(response) = Request::put("/api/sync")
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&batch).unwrap_or_default())
            .unwrap()
            .send()
            .await
        else {
            return;
        };
        if !response.ok() {
            return;
        }
    }
    if let Ok(expected) = serde_wasm_bindgen::to_value(&dirty) {
        if let Ok(promise) = storageClearDirtyIfUnchanged(&expected) {
            let _ = JsFuture::from(promise).await;
        }
    }
    let _ = pull_remote().await;
    record_sync_completed();
}

async fn normalize_legacy_results_in_storage() -> bool {
    let Ok(promise) = storageAll("data") else {
        return false;
    };
    let Ok(data_value) = JsFuture::from(promise).await else {
        return false;
    };
    let Ok(data) = serde_wasm_bindgen::from_value::<BTreeMap<String, String>>(data_value) else {
        return false;
    };
    let mut changed = false;
    for (key, value) in data {
        if let Some((parsed, result_changed)) = results::parse_stored_results(&key, &value) {
            if result_changed {
                put_stored(
                    key,
                    serde_json::to_string(&parsed).unwrap_or_else(|_| "[]".to_owned()),
                )
                .await;
                changed = true;
            }
        }
    }
    changed
}

async fn pull_remote() -> bool {
    let dirty = match storageAll("dirty") {
        Ok(promise) => match JsFuture::from(promise).await {
            Ok(value) => {
                serde_wasm_bindgen::from_value::<BTreeMap<String, f64>>(value).unwrap_or_default()
            }
            Err(_) => BTreeMap::new(),
        },
        Err(_) => BTreeMap::new(),
    };
    let trainers = match Request::get("/api/trainers").send().await {
        Ok(response) if response.ok() => match response.json::<Vec<TrainerSummary>>().await {
            Ok(trainers) if !trainers.is_empty() => trainers,
            _ => return false,
        },
        _ => return false,
    };
    let Ok(response) = Request::get("/api/sync").send().await else {
        return false;
    };
    let Ok(mut bundle) = response.json::<SyncBundle>().await else {
        return false;
    };
    let mut seen_cursors = BTreeMap::<String, ()>::new();
    loop {
        let Some(cursor) = bundle.next_results.take() else {
            break;
        };
        let cursor_key = format!("{}|{}", cursor.datetime, cursor.id);
        if seen_cursors.insert(cursor_key, ()).is_some() {
            return false;
        }
        let url = format!(
            "/api/sync?before_datetime={}&before_id={}",
            js_sys::encode_uri_component(&cursor.datetime),
            js_sys::encode_uri_component(&cursor.id)
        );
        let Ok(response) = Request::get(&url).send().await else {
            return false;
        };
        let Ok(page) = response.json::<SyncBundle>().await else {
            return false;
        };
        bundle.results.extend(page.results);
        bundle.next_results = page.next_results;
    }
    let mut imported = Vec::new();
    if let Some(settings) = bundle.settings {
        let key = SETTINGS_KEY.to_owned();
        if !dirty.contains_key(&key) {
            put_stored(key.clone(), settings.settings_json.to_string()).await;
            imported.push(key);
        }
    }
    for selection in bundle.selections {
        let key = trainers
            .iter()
            .find(|trainer| trainer.trainer_id == selection.trainer_id)
            .map(|trainer| trainer.selection_key.clone())
            .unwrap_or_else(|| format!("{}Selection", selection.trainer_id));
        let cases = selection
            .selection_json
            .get("cases")
            .cloned()
            .unwrap_or_else(|| selection.selection_json.clone());
        let presets = selection.selection_json.get("presets").cloned();
        let algsets = selection.selection_json.get("algsets").cloned();
        if !dirty.contains_key(&key) {
            put_stored(key.clone(), cases.to_string()).await;
            imported.push(key.clone());
        }
        let preset_key = format!("{key}Presets");
        if let Some(presets) = presets {
            if !dirty.contains_key(&preset_key) {
                put_stored(preset_key.clone(), presets.to_string()).await;
                imported.push(preset_key);
            }
        }
        let algset_key = format!("{key}Algsets");
        if let Some(algsets) = algsets {
            if !dirty.contains_key(&algset_key) {
                put_stored(algset_key.clone(), algsets.to_string()).await;
                imported.push(algset_key);
            }
        }
    }
    let mut remote_algs = BTreeMap::<String, BTreeMap<String, Vec<UserAlgState>>>::new();
    for alg in bundle.user_algs {
        let values = if alg.alg_id.starts_with("legacy:") && alg.alg.contains('\n') {
            alg.alg
                .lines()
                .enumerate()
                .map(|(position, value)| UserAlgState {
                    alg_id: format!("{}:{position}", alg.alg_id),
                    alg: value.to_owned(),
                    position: position as i32,
                    hidden: alg.hidden,
                    deleted: alg.deleted,
                })
                .collect()
        } else {
            vec![UserAlgState {
                alg_id: alg.alg_id,
                alg: alg.alg,
                position: alg.position,
                hidden: alg.hidden,
                deleted: alg.deleted,
            }]
        };
        remote_algs
            .entry(format!(
                "{}customAlgs",
                trainers
                    .iter()
                    .find(|trainer| trainer.trainer_id == alg.trainer_id)
                    .map(|trainer| trainer.selection_key.as_str())
                    .unwrap_or(&alg.trainer_id)
            ))
            .or_default()
            .entry(alg.case_id)
            .or_default()
            .extend(values);
    }
    for (key, case_algs) in remote_algs {
        if !dirty.contains_key(&key) {
            put_stored(
                key.clone(),
                serde_json::Value::Object(algs::custom_algs_payload(&case_algs)).to_string(),
            )
            .await;
            imported.push(key);
        }
    }
    let mut remote_results = BTreeMap::<String, Vec<ResultSync>>::new();
    for result in bundle.results {
        remote_results
            .entry(
                trainers
                    .iter()
                    .find(|trainer| trainer.trainer_id == result.trainer_id)
                    .map(|trainer| trainer.times_key.clone())
                    .unwrap_or_else(|| format!("{}Times", result.trainer_id)),
            )
            .or_default()
            .push(result);
    }
    for (key, items) in remote_results {
        if !dirty.contains_key(&key) {
            let mut merged = get_stored(&key)
                .await
                .and_then(|value| {
                    results::parse_stored_results(&key, &value).map(|(parsed, _)| parsed)
                })
                .unwrap_or_default();
            for result in items {
                merged = results::merge_remote_result(merged, result);
            }
            put_stored(
                key.clone(),
                serde_json::to_string(&merged).unwrap_or_else(|_| "[]".to_owned()),
            )
            .await;
            imported.push(key);
        }
    }
    if !imported.is_empty() {
        let imported = imported.into_iter().collect::<BTreeSet<_>>();
        if let Ok(promise) = storageAll("dirty") {
            if let Ok(value) = JsFuture::from(promise).await {
                let mut current = serde_wasm_bindgen::from_value::<BTreeMap<String, f64>>(value)
                    .unwrap_or_default();
                current.retain(|key, _| imported.contains(key));
                if let Ok(expected) = serde_wasm_bindgen::to_value(&current) {
                    if let Ok(promise) = storageClearDirtyIfUnchanged(&expected) {
                        let _ = JsFuture::from(promise).await;
                    }
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_pushes_results_in_server_sized_batches() {
        let result = ResultSync {
            id: "00000000-0000-4000-8000-000000000000".to_owned(),
            datetime: "2026-01-01T00:00:00Z".to_owned(),
            result_ms: 1,
            trainer_id: "trainer".to_owned(),
            case_id: "case".to_owned(),
            updated_at: "2026-01-01T00:00:00Z".to_owned(),
            deleted: false,
        };
        let mut bundle = SyncBundle::default();
        bundle.results = vec![result; SYNC_RESULT_BATCH_SIZE + 1];

        let batches = split_sync_batches(bundle);

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].results.len(), SYNC_RESULT_BATCH_SIZE);
        assert_eq!(batches[1].results.len(), 1);
    }
}
