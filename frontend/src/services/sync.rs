use crate::logic::{algs, results};
use crate::models::{
    MeResponse, ResultSync, SelectionSync, SettingsSync, StoredResult, SyncBundle, TrainerSummary,
    UserAlgState, UserAlgSync,
};
use crate::services::api::save_cached_account;
use crate::services::storage::{
    get_stored, metadataGet, metadataPut, put_stored, put_stored_if_clean, storageAll,
    storageClearDirtyIfUnchanged, syncable_settings, SETTINGS_KEY,
};
use gloo_net::http::Request;
use gloo_timers::callback::{Interval, Timeout};
use js_sys::Date;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

pub const SYNC_RESULT_BATCH_SIZE: usize = 2_000;
const SYNC_CURSOR_KEY: &str = "syncCursor";
const PUSH_WATERMARK_OVERLAP_MS: f64 = 5_000.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    Synced,
    InProgress,
    Throttled,
    Unauthenticated,
    Offline,
    RateLimited,
    PayloadRejected(u16),
    ServerError(u16),
    HttpError(u16),
    StorageError(&'static str),
    DecodeError(&'static str),
}

impl SyncOutcome {
    pub fn from_http_status(status: u16) -> Self {
        match status {
            401 | 403 => Self::Unauthenticated,
            413 | 422 => Self::PayloadRejected(status),
            429 => Self::RateLimited,
            500..=599 => Self::ServerError(status),
            other => Self::HttpError(other),
        }
    }

    pub fn user_message(&self) -> Option<String> {
        match self {
            Self::Synced | Self::Throttled => None,
            Self::InProgress => Some("A sync is already in progress.".to_owned()),
            Self::Unauthenticated => {
                Some("Your session expired. Please sign in again.".to_owned())
            }
            Self::Offline => {
                Some("Unable to reach the server. Please try again when online.".to_owned())
            }
            Self::RateLimited => {
                Some("Too many sync requests. Please wait a moment and try again.".to_owned())
            }
            Self::PayloadRejected(status) => Some(format!(
                "Sync data was rejected by the server (HTTP {status})."
            )),
            Self::ServerError(status) => Some(format!(
                "Server error during sync (HTTP {status}). Please try again later."
            )),
            Self::HttpError(status) => Some(format!("Sync failed with HTTP {status}.")),
            Self::StorageError(detail) => Some(format!("Local browser storage error ({detail}).")),
            Self::DecodeError(detail) => {
                Some(format!("Unexpected response from server ({detail})."))
            }
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct SyncCursorState {
    account: String,
    #[serde(default)]
    pull_cursor: Option<String>,
    #[serde(default)]
    last_push_at: Option<String>,
}

async fn load_sync_cursor(account: &str) -> SyncCursorState {
    let stored = match metadataGet(SYNC_CURSOR_KEY) {
        Ok(promise) => JsFuture::from(promise)
            .await
            .ok()
            .and_then(|value| value.as_string())
            .and_then(|text| serde_json::from_str::<SyncCursorState>(&text).ok()),
        Err(_) => None,
    };
    match stored {
        Some(state) if state.account == account => state,
        _ => SyncCursorState {
            account: account.to_owned(),
            pull_cursor: None,
            last_push_at: None,
        },
    }
}

async fn save_sync_cursor(state: &SyncCursorState) {
    if let Ok(serialized) = serde_json::to_string(state) {
        if let Ok(promise) = metadataPut(SYNC_CURSOR_KEY, &serialized) {
            let _ = JsFuture::from(promise).await;
        }
    }
}

pub fn filter_changed_results(
    items: Vec<StoredResult>,
    last_push_at: Option<&str>,
) -> Vec<StoredResult> {
    let Some(cutoff) = last_push_at else {
        return items;
    };
    items
        .into_iter()
        .filter(|item| match item.updated_at.as_deref() {
            None => true,
            Some(ts) => ts >= cutoff,
        })
        .collect()
}

pub fn merge_custom_algs(
    mut existing: BTreeMap<String, Vec<UserAlgState>>,
    incoming: BTreeMap<String, Vec<UserAlgState>>,
) -> BTreeMap<String, Vec<UserAlgState>> {
    for (case_id, incoming_items) in incoming {
        let entry = existing.entry(case_id).or_default();
        for item in incoming_items {
            if let Some(slot) = entry.iter_mut().find(|cur| cur.alg_id == item.alg_id) {
                *slot = item;
            } else {
                entry.push(item);
            }
        }
        entry.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then_with(|| left.alg_id.cmp(&right.alg_id))
        });
    }
    existing
}

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

pub async fn sync_now(force: bool) -> SyncOutcome {
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
        return SyncOutcome::InProgress;
    }

    let mut current_force = force;
    let mut outcome;
    loop {
        outcome = sync_once(current_force).await;
        let next = SYNC_PENDING.with(|pending| pending.borrow_mut().take());
        match next {
            Some(next_force) => current_force = next_force,
            None => break,
        }
    }

    SYNC_IN_FLIGHT.with(|in_flight| *in_flight.borrow_mut() = false);
    outcome
}

async fn sync_once(force: bool) -> SyncOutcome {
    let now = Date::now();
    let too_soon = !force && LAST_SYNC.with(|last| now - *last.borrow() < 10_000.0);
    if too_soon {
        return SyncOutcome::Throttled;
    }
    LAST_SYNC.with(|last| *last.borrow_mut() = now);
    let me_response = match Request::get("/api/me").send().await {
        Ok(response) => response,
        Err(_) => return SyncOutcome::Offline,
    };
    if !me_response.ok() {
        return SyncOutcome::from_http_status(me_response.status());
    }
    let Ok(me) = me_response.json::<MeResponse>().await else {
        return SyncOutcome::DecodeError("account status");
    };
    save_cached_account(&me).await;
    if !me.authenticated {
        return SyncOutcome::Unauthenticated;
    }
    let account_email = me.email.as_deref().unwrap_or("");
    let push_started_ms = Date::now();
    let push_started_iso = Date::new(&JsValue::from_f64(push_started_ms))
        .to_iso_string()
        .as_string()
        .unwrap_or_default();
    let push_watermark_iso =
        Date::new(&JsValue::from_f64((push_started_ms - PUSH_WATERMARK_OVERLAP_MS).max(0.0)))
            .to_iso_string()
            .as_string()
            .unwrap_or_else(|| push_started_iso.clone());

    let Ok(promise) = storageAll("dirty") else {
        return SyncOutcome::StorageError("read dirty keys");
    };
    let Ok(dirty_value) = JsFuture::from(promise).await else {
        return SyncOutcome::StorageError("read dirty keys");
    };
    let Ok(mut dirty) = serde_wasm_bindgen::from_value::<BTreeMap<String, f64>>(dirty_value) else {
        return SyncOutcome::StorageError("decode dirty keys");
    };
    if dirty.is_empty() {
        if !normalize_legacy_results_in_storage().await {
            let outcome = pull_remote(account_email).await;
            if outcome == SyncOutcome::Synced {
                record_sync_completed();
            }
            return outcome;
        }
        let Ok(promise) = storageAll("dirty") else {
            return SyncOutcome::StorageError("refresh dirty keys");
        };
        let Ok(dirty_value) = JsFuture::from(promise).await else {
            return SyncOutcome::StorageError("refresh dirty keys");
        };
        let Ok(refreshed) = serde_wasm_bindgen::from_value::<BTreeMap<String, f64>>(dirty_value)
        else {
            return SyncOutcome::StorageError("decode dirty keys");
        };
        dirty = refreshed;
        if dirty.is_empty() {
            let outcome = pull_remote(account_email).await;
            if outcome == SyncOutcome::Synced {
                record_sync_completed();
            }
            return outcome;
        }
    }
    let trainers = match Request::get("/api/trainers").send().await {
        Ok(response) if response.ok() => match response.json::<Vec<TrainerSummary>>().await {
            Ok(trainers) if !trainers.is_empty() => trainers,
            _ => return SyncOutcome::DecodeError("trainer catalog"),
        },
        Ok(response) => return SyncOutcome::from_http_status(response.status()),
        Err(_) => return SyncOutcome::Offline,
    };
    let Ok(promise) = storageAll("data") else {
        return SyncOutcome::StorageError("read local data");
    };
    let Ok(data_value) = JsFuture::from(promise).await else {
        return SyncOutcome::StorageError("read local data");
    };
    let Ok(data) = serde_wasm_bindgen::from_value::<BTreeMap<String, String>>(data_value) else {
        return SyncOutcome::StorageError("decode local data");
    };
    let mut cursor_state = load_sync_cursor(account_email).await;
    let mut bundle = SyncBundle::default();
    let mut pushed_selection_trainers = BTreeSet::new();
    for key in dirty.keys() {
        let updated_at = Date::new(&JsValue::from_f64(*dirty.get(key).unwrap_or(&0.0)))
            .to_iso_string()
            .as_string()
            .unwrap_or_else(|| push_started_iso.clone());
        let Some(value) = data.get(key) else {
            continue;
        };
        if key == SETTINGS_KEY {
            if let Ok(settings_json) = serde_json::from_str(value) {
                bundle.settings = Some(SettingsSync {
                    settings_json: syncable_settings(settings_json),
                    updated_at: updated_at.clone(),
                });
            }
        } else if let Some(trainer) = trainers.iter().find(|trainer| {
            trainer.selection_key == *key
                || format!("{}Presets", trainer.selection_key) == *key
                || format!("{}Algsets", trainer.selection_key) == *key
        }) {
            if !pushed_selection_trainers.insert(trainer.trainer_id.clone()) {
                continue;
            }
            let preset_key = format!("{}Presets", trainer.selection_key);
            let algset_key = format!("{}Algsets", trainer.selection_key);
            let latest_ts = [
                dirty.get(&trainer.selection_key).copied(),
                dirty.get(&preset_key).copied(),
                dirty.get(&algset_key).copied(),
            ]
            .into_iter()
            .flatten()
            .fold(0.0_f64, f64::max);
            let selection_updated_at = if latest_ts > 0.0 {
                Date::new(&JsValue::from_f64(latest_ts))
                    .to_iso_string()
                    .as_string()
                    .unwrap_or_else(|| updated_at.clone())
            } else {
                updated_at.clone()
            };
            let cases = data
                .get(&trainer.selection_key)
                .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
                .unwrap_or_else(|| serde_json::json!([]));
            let presets = data
                .get(&preset_key)
                .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
                .unwrap_or_else(|| serde_json::json!({}));
            let algsets = data
                .get(&algset_key)
                .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
                .unwrap_or_else(|| serde_json::json!({}));
            bundle.selections.push(SelectionSync {
                trainer_id: trainer.trainer_id.clone(),
                selection_json: serde_json::json!({
                    "cases": cases,
                    "presets": presets,
                    "algsets": algsets
                }),
                updated_at: selection_updated_at,
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
                let to_push =
                    filter_changed_results(parsed_results, cursor_state.last_push_at.as_deref());
                bundle
                    .results
                    .extend(to_push.into_iter().map(|result| ResultSync {
                        id: result.id,
                        datetime: result.datetime,
                        result_ms: result.ms,
                        trainer_id: trainer.trainer_id.clone(),
                        case_id: result.case,
                        updated_at: result.updated_at.unwrap_or_else(|| updated_at.clone()),
                        deleted: result.deleted,
                    }));
            }
        }
    }
    let has_payload = bundle.settings.is_some()
        || !bundle.selections.is_empty()
        || !bundle.user_algs.is_empty()
        || !bundle.results.is_empty();
    if has_payload {
        for batch in split_sync_batches(bundle) {
            let response = match Request::put("/api/sync")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&batch).unwrap_or_default())
                .unwrap()
                .send()
                .await
            {
                Ok(response) => response,
                Err(_) => return SyncOutcome::Offline,
            };
            if !response.ok() {
                return SyncOutcome::from_http_status(response.status());
            }
        }
    }
    cursor_state.last_push_at = Some(push_watermark_iso);
    save_sync_cursor(&cursor_state).await;
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    if let Ok(expected) = dirty.serialize(&serializer) {
        if let Ok(promise) = storageClearDirtyIfUnchanged(&expected) {
            let _ = JsFuture::from(promise).await;
        }
    }
    let outcome = pull_remote(account_email).await;
    if outcome == SyncOutcome::Synced {
        record_sync_completed();
    }
    outcome
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

async fn pull_remote(account_email: &str) -> SyncOutcome {
    let mut cursor_state = load_sync_cursor(account_email).await;
    let since = cursor_state.pull_cursor.clone();
    let trainers = match Request::get("/api/trainers").send().await {
        Ok(response) if response.ok() => match response.json::<Vec<TrainerSummary>>().await {
            Ok(trainers) if !trainers.is_empty() => trainers,
            _ => return SyncOutcome::DecodeError("trainer catalog"),
        },
        Ok(response) => return SyncOutcome::from_http_status(response.status()),
        Err(_) => return SyncOutcome::Offline,
    };
    let initial_url = match since.as_deref() {
        Some(since) => format!("/api/sync?since={}", js_sys::encode_uri_component(since)),
        None => "/api/sync".to_owned(),
    };
    let response = match Request::get(&initial_url).send().await {
        Ok(response) => response,
        Err(_) => return SyncOutcome::Offline,
    };
    if !response.ok() {
        return SyncOutcome::from_http_status(response.status());
    }
    let Ok(mut bundle) = response.json::<SyncBundle>().await else {
        return SyncOutcome::DecodeError("sync bundle");
    };
    let server_time = bundle.server_time.clone();
    let mut seen_cursors = BTreeMap::<String, ()>::new();
    loop {
        let Some(cursor) = bundle.next_results.take() else {
            break;
        };
        let cursor_key = format!("{}|{}", cursor.datetime, cursor.id);
        if seen_cursors.insert(cursor_key, ()).is_some() {
            return SyncOutcome::DecodeError("pagination cursor loop");
        }
        let url = match since.as_deref() {
            Some(since) => format!(
                "/api/sync?since={}&before_datetime={}&before_id={}",
                js_sys::encode_uri_component(since),
                js_sys::encode_uri_component(&cursor.datetime),
                js_sys::encode_uri_component(&cursor.id)
            ),
            None => format!(
                "/api/sync?before_datetime={}&before_id={}",
                js_sys::encode_uri_component(&cursor.datetime),
                js_sys::encode_uri_component(&cursor.id)
            ),
        };
        let response = match Request::get(&url).send().await {
            Ok(response) => response,
            Err(_) => return SyncOutcome::Offline,
        };
        if !response.ok() {
            return SyncOutcome::from_http_status(response.status());
        }
        let Ok(page) = response.json::<SyncBundle>().await else {
            return SyncOutcome::DecodeError("sync page");
        };
        bundle.results.extend(page.results);
        bundle.next_results = page.next_results;
    }
    let mut all_clean = true;
    if let Some(settings) = bundle.settings {
        let key = SETTINGS_KEY.to_owned();
        if !put_stored_if_clean(key, settings.settings_json.to_string()).await {
            all_clean = false;
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
        if !put_stored_if_clean(key.clone(), cases.to_string()).await {
            all_clean = false;
        }
        let preset_key = format!("{key}Presets");
        if let Some(presets) = presets {
            if !put_stored_if_clean(preset_key, presets.to_string()).await {
                all_clean = false;
            }
        }
        let algset_key = format!("{key}Algsets");
        if let Some(algsets) = algsets {
            if !put_stored_if_clean(algset_key, algsets.to_string()).await {
                all_clean = false;
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
        let merged_algs = if since.is_some() {
            let existing = get_stored(&key)
                .await
                .and_then(|value| {
                    serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&value).ok()
                })
                .map(|parsed| {
                    parsed
                        .into_iter()
                        .map(|(case_id, value)| {
                            let items = algs::parse_custom_algs(&value, &case_id);
                            (case_id, items)
                        })
                        .collect()
                })
                .unwrap_or_default();
            merge_custom_algs(existing, case_algs)
        } else {
            case_algs
        };
        if !put_stored_if_clean(
            key,
            serde_json::Value::Object(algs::custom_algs_payload(&merged_algs)).to_string(),
        )
        .await
        {
            all_clean = false;
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
        let existing = get_stored(&key)
            .await
            .and_then(|value| results::parse_stored_results(&key, &value).map(|(parsed, _)| parsed))
            .unwrap_or_default();
        let merged = results::merge_remote_results(existing, items);
        if !put_stored_if_clean(
            key,
            serde_json::to_string(&merged).unwrap_or_else(|_| "[]".to_owned()),
        )
        .await
        {
            all_clean = false;
        }
    }
    if all_clean {
        if let Some(server_time) = server_time {
            cursor_state.pull_cursor = Some(server_time);
            save_sync_cursor(&cursor_state).await;
        }
    }
    SyncOutcome::Synced
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

    #[test]
    fn incremental_push_skips_results_unchanged_since_last_push() {
        let items = vec![
            StoredResult {
                id: "old".to_owned(),
                datetime: "2026-01-01T00:00:00Z".to_owned(),
                ms: 1000,
                case: "1".to_owned(),
                time: "1.00".to_owned(),
                deleted: false,
                updated_at: Some("2026-01-01T00:00:00Z".to_owned()),
            },
            StoredResult {
                id: "new".to_owned(),
                datetime: "2026-01-02T00:00:00Z".to_owned(),
                ms: 1200,
                case: "1".to_owned(),
                time: "1.20".to_owned(),
                deleted: false,
                updated_at: Some("2026-01-03T00:00:00Z".to_owned()),
            },
            StoredResult {
                id: "legacy".to_owned(),
                datetime: "2026-01-01T00:00:01Z".to_owned(),
                ms: 1500,
                case: "1".to_owned(),
                time: "1.50".to_owned(),
                deleted: false,
                updated_at: None,
            },
        ];

        let filtered = filter_changed_results(items, Some("2026-01-02T00:00:00Z"));
        assert_eq!(
            filtered.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
            vec!["new", "legacy"]
        );
    }

    #[test]
    fn incremental_pull_merges_custom_algs_without_dropping_untouched_cases() {
        let mut existing = BTreeMap::new();
        existing.insert(
            "case-1".to_owned(),
            vec![UserAlgState {
                alg_id: "user:1".to_owned(),
                alg: "R U R'".to_owned(),
                position: 0,
                hidden: false,
                deleted: false,
            }],
        );
        existing.insert(
            "case-2".to_owned(),
            vec![UserAlgState {
                alg_id: "user:2".to_owned(),
                alg: "F R U R' U' F'".to_owned(),
                position: 0,
                hidden: false,
                deleted: false,
            }],
        );

        let mut incoming = BTreeMap::new();
        incoming.insert(
            "case-1".to_owned(),
            vec![UserAlgState {
                alg_id: "user:1".to_owned(),
                alg: "R U2 R'".to_owned(),
                position: 0,
                hidden: false,
                deleted: false,
            }],
        );

        let merged = merge_custom_algs(existing, incoming);
        assert_eq!(merged["case-1"][0].alg, "R U2 R'");
        assert_eq!(merged["case-2"][0].alg, "F R U R' U' F'");
    }
}
