mod cube_schemes;
mod cube_settings;

use gloo_net::http::Request;
use gloo_timers::callback::{Interval, Timeout};
use js_sys::{Date, Math};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{DragEvent, Element, KeyboardEvent, MouseEvent};
use yew::prelude::*;
use yew_router::prelude::*;

#[wasm_bindgen(inline_js = r#"
function openStorage() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("alg-trainers", 2);
    request.onupgradeneeded = () => {
      if (!request.result.objectStoreNames.contains("data")) {
        request.result.createObjectStore("data");
      }
      if (!request.result.objectStoreNames.contains("metadata")) {
        request.result.createObjectStore("metadata");
      }
      if (!request.result.objectStoreNames.contains("dirty")) {
        request.result.createObjectStore("dirty");
      }
    };
    request.onsuccess = () => {
      request.result.onversionchange = () => request.result.close();
      resolve(request.result);
    };
    request.onerror = () => reject(request.error);
    request.onblocked = () => reject(new Error("Browser storage upgrade blocked by another tab."));
  });
}
export async function storageGet(key) {
  const db = await openStorage();
  if (key === "ALLGSettings") {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(["data", "metadata", "dirty"], "readwrite");
      const data = tx.objectStore("data");
      const metadata = tx.objectStore("metadata");
      const stored = data.get(key);
      const legacyMarker = metadata.get("localStorageMigrated");
      const settingsMarker = metadata.get("yewSettingsMigrated");
      let result = null;
      settingsMarker.onsuccess = () => {
        try {
          result = stored.result ?? null;
          if (!settingsMarker.result && !legacyMarker.result && result === null) {
            result = localStorage.getItem(key);
            if (result !== null) {
              data.put(result, key);
              tx.objectStore("dirty").put(Date.now(), key);
            }
          }
          metadata.put(true, "yewSettingsMigrated");
        } catch (error) {
          tx.abort();
          reject(error);
        }
      };
      tx.oncomplete = () => resolve(result);
      tx.onerror = () => reject(tx.error);
      tx.onabort = () => reject(tx.error || new Error("Unable to migrate settings"));
    });
  }
  return await new Promise((resolve, reject) => {
    const request = db.transaction("data", "readonly").objectStore("data").get(key);
    request.onsuccess = () => resolve(request.result ?? null);
    request.onerror = () => reject(request.error);
  });
}
export async function storagePut(key, value) {
  const db = await openStorage();
  await new Promise((resolve, reject) => {
    const tx = db.transaction(["data", "dirty"], "readwrite");
    tx.objectStore("data").put(value, key);
    tx.objectStore("dirty").put(Date.now(), key);
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
  window.dispatchEvent(new CustomEvent("alg-trainers-change", { detail: { key, value } }));
}
export async function storageAll(storeName) {
  const db = await openStorage();
  return await new Promise((resolve, reject) => {
    const tx = db.transaction(storeName, "readonly");
    const values = tx.objectStore(storeName).getAll();
    const keys = tx.objectStore(storeName).getAllKeys();
    tx.oncomplete = () => resolve(Object.fromEntries(keys.result.map((key, i) => [key, values.result[i]])));
    tx.onerror = () => reject(tx.error);
  });
}
export async function storageClearDirtyIfUnchanged(expected) {
  const db = await openStorage();
  await new Promise((resolve, reject) => {
    const tx = db.transaction("dirty", "readwrite");
    const store = tx.objectStore("dirty");
    for (const [key, timestamp] of Object.entries(expected)) {
      const request = store.get(key);
      request.onsuccess = () => {
        if (request.result === timestamp) store.delete(key);
      };
    }
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
}
export function prefersLightTheme() {
  return window.matchMedia && window.matchMedia("(prefers-color-scheme: light)").matches;
}
export function colorInputStyle(value, background) {
  if (!CSS.supports("color", value) || !CSS.supports("color", background)) {
    throw new Error("Please enter a valid CSS color.");
  }
  const probe = document.createElement("span");
  probe.style.color = value;
  document.body.appendChild(probe);
  const resolved = getComputedStyle(probe).color;
  probe.style.color = background;
  const base = getComputedStyle(probe).color;
  probe.remove();
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = 1;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) throw new Error("Unable to calculate color contrast.");
  context.fillStyle = "white";
  context.fillRect(0, 0, 1, 1);
  context.fillStyle = base;
  context.fillRect(0, 0, 1, 1);
  context.fillStyle = resolved;
  context.fillRect(0, 0, 1, 1);
  const rgb = context.getImageData(0, 0, 1, 1).data;
  const linear = channel => {
    const c = channel / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  const luminance = 0.2126 * linear(rgb[0]) + 0.7152 * linear(rgb[1]) + 0.0722 * linear(rgb[2]);
  const text = (luminance + 0.05) / 0.05 >= 1.05 / (luminance + 0.05) ? "black" : "white";
  return `background-color:${value};color:${text}`;
}
export function downloadText(filename, text) {
  const url = URL.createObjectURL(new Blob([text], { type: "application/json" }));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}
export async function readFileText(file) {
  return await file.text();
}
export function randomAlgId() {
  if (crypto.randomUUID) return `user:${crypto.randomUUID()}`;
  const random = crypto.getRandomValues(new Uint32Array(4));
  return `user:${Array.from(random, value => value.toString(16).padStart(8, "0")).join("")}`;
}
function stableLegacyUuid(key, index, result) {
  const input = `${key}\0${index}\0${Number(result.ms)}\0${String(result.case)}`;
  const bytes = new Uint8Array(16);
  let hash = 0x811c9dc5;
  for (const byte of new TextEncoder().encode(input)) {
    hash ^= byte;
    hash = Math.imul(hash, 0x01000193);
  }
  for (let offset = 0; offset < bytes.length; offset++) {
    hash ^= offset + 0x9e3779b9;
    hash = Math.imul(hash, 0x01000193);
    bytes[offset] = hash & 0xff;
  }
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = Array.from(bytes, value => value.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}
export async function importStorageText(text) {
  const parsed = JSON.parse(text);
  const values = parsed.data && typeof parsed.data === "object" ? parsed.data : parsed;
  if (!values || typeof values !== "object" || Array.isArray(values)) {
    throw new Error("Invalid trainer backup");
  }
  const db = await openStorage();
  await new Promise((resolve, reject) => {
    const tx = db.transaction(["data", "dirty"], "readwrite");
    const data = tx.objectStore("data");
    try {
      data.clear();
      tx.objectStore("dirty").clear();
      for (const [key, value] of Object.entries(values)) {
        if (typeof value !== "string") throw new Error("Invalid trainer backup value");
        let stored = value;
        try {
          const parsed = JSON.parse(value);
          if (Array.isArray(parsed) && parsed.every(result =>
            result && typeof result === "object" && "ms" in result && "case" in result
          )) {
            stored = JSON.stringify(parsed.map((result, index) => ({
              ...result,
              id: typeof result.id === "string" && result.id ? result.id : stableLegacyUuid(key, index, result),
              datetime: typeof result.datetime === "string" && result.datetime
                ? result.datetime
                : new Date(Date.UTC(2000, 0, 1) + index * 1000).toISOString(),
              case: String(result.case),
              time: typeof result.time === "string" && result.time ? result.time : `${Math.trunc(result.ms / 1000)}.${String(result.ms % 1000).padStart(3, "0")}`,
              deleted: result.deleted === true,
            })));
          }
        } catch {
          // Other stored values retain their original representation.
        }
        data.put(stored, key);
        tx.objectStore("dirty").put(Date.now(), key);
      }
    } catch (error) {
      tx.abort();
      reject(error);
      return;
    }
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    fn storageGet(key: &str) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    fn storagePut(key: &str, value: &str) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    fn storageAll(store_name: &str) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    fn storageClearDirtyIfUnchanged(expected: &JsValue) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    fn prefersLightTheme() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch)]
    fn colorInputStyle(value: &str, background: &str) -> Result<String, JsValue>;
    fn downloadText(filename: &str, text: &str);
    #[wasm_bindgen(catch)]
    fn readFileText(file: &web_sys::File) -> Result<js_sys::Promise, JsValue>;
    fn randomAlgId() -> String;
    #[wasm_bindgen(catch)]
    fn importStorageText(text: &str) -> Result<js_sys::Promise, JsValue>;
}

async fn get_stored(key: &str) -> Option<String> {
    let result = match storageGet(key) {
        Ok(promise) => JsFuture::from(promise).await,
        Err(error) => Err(error),
    };
    match result {
        Ok(value) => value.as_string(),
        Err(error) => {
            web_sys::console::error_1(&error);
            if key == "ALLGSettings" {
                if let Some(window) = web_sys::window() {
                    let _ =
                        window.alert_with_message("Unable to load settings from browser storage.");
                }
            }
            None
        }
    }
}

async fn put_stored(key: String, value: String) {
    let result = match storagePut(&key, &value) {
        Ok(promise) => JsFuture::from(promise).await.map(|_| ()),
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        web_sys::console::error_1(&error);
        if key == "ALLGSettings" {
            if let Some(window) = web_sys::window() {
                let _ = window.alert_with_message("Unable to save settings to browser storage.");
            }
        }
    }
}

async fn put_stored_results(key: String, results: Vec<StoredResult>, deleted_ids: Vec<String>) {
    let mut merged = get_stored(&key)
        .await
        .and_then(|value| parse_stored_results(&key, &value).map(|(results, _)| results))
        .unwrap_or_default();
    let deleted_ids = deleted_ids
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    for result in &mut merged {
        if deleted_ids.contains(&result.id) {
            result.deleted = true;
        }
    }
    for mut result in results {
        if deleted_ids.contains(&result.id) {
            result.deleted = true;
        }
        if let Some(existing) = merged.iter_mut().find(|existing| existing.id == result.id) {
            *existing = result;
        } else {
            merged.push(result);
        }
    }
    merged.sort_by(compare_results);
    put_stored(
        key,
        serde_json::to_string(&merged).unwrap_or_else(|_| "[]".to_owned()),
    )
    .await;
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/trainer/:id")]
    Trainer { id: String },
    #[at("/algsheet")]
    AlgSheet,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct MeResponse {
    authenticated: bool,
    email: Option<String>,
    wca_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TrainerSummary {
    trainer_id: String,
    puzzle: String,
    method: String,
    name: String,
    order: i32,
    max_algs_per_row: i32,
    selection_key: String,
    times_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TrainerCase {
    case_id: String,
    name: String,
    setup: Option<String>,
    image_svg: Option<String>,
    algs: Vec<TrainerAlg>,
    scrambles: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrainerAlg {
    alg_id: String,
    alg: String,
    position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserAlgState {
    alg_id: String,
    alg: String,
    position: i32,
    #[serde(default)]
    hidden: bool,
    #[serde(default)]
    deleted: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct DisplayAlg {
    alg_id: String,
    alg: String,
    position: i32,
    user: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct TrainerGroup {
    name: String,
    cases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TrainerAlgset {
    name: String,
    groups: Vec<String>,
    selected: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TrainerBundle {
    trainer_id: String,
    title: String,
    puzzle: String,
    method: String,
    cases: Vec<TrainerCase>,
    groups: Vec<TrainerGroup>,
    algsets: Vec<TrainerAlgset>,
    pre_rotations: Vec<String>,
    post_rotations: Vec<String>,
    pre_moves: Vec<String>,
    post_moves: Vec<String>,
    max_algs_per_row: i32,
    #[serde(default)]
    u_move_cycles: Vec<Vec<usize>>,
    selection_key: String,
    times_key: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
struct StoredResult {
    #[serde(default)]
    id: String,
    #[serde(default)]
    datetime: String,
    ms: i64,
    #[serde(deserialize_with = "deserialize_case_id")]
    case: String,
    #[serde(default)]
    time: String,
    #[serde(default)]
    deleted: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SelectionPreset {
    sel_cases: Vec<String>,
    #[serde(default)]
    selected_alg_sets: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct SyncBundle {
    settings: Option<SettingsSync>,
    selections: Vec<SelectionSync>,
    user_algs: Vec<UserAlgSync>,
    results: Vec<ResultSync>,
    #[serde(default)]
    next_results: Option<ResultCursor>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SettingsSync {
    settings_json: serde_json::Value,
    updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SelectionSync {
    trainer_id: String,
    selection_json: serde_json::Value,
    updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct UserAlgSync {
    trainer_id: String,
    case_id: String,
    alg_id: String,
    alg: String,
    position: i32,
    hidden: bool,
    deleted: bool,
    updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ResultSync {
    id: String,
    datetime: String,
    result_ms: i64,
    trainer_id: String,
    case_id: String,
    updated_at: String,
    deleted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ResultCursor {
    datetime: String,
    id: String,
}

fn parse_custom_algs(value: &serde_json::Value, case_id: &str) -> Vec<UserAlgState> {
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

fn deserialize_case_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(value) => Ok(value),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        _ => Err(serde::de::Error::custom("case must be a string or number")),
    }
}

fn parse_stored_results(key: &str, value: &str) -> Option<(Vec<StoredResult>, bool)> {
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

fn stable_legacy_uuid(key: &str, index: usize, milliseconds: i64, case: &str) -> String {
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

fn legacy_result_datetime(index: usize) -> String {
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

fn compare_results(left: &StoredResult, right: &StoredResult) -> std::cmp::Ordering {
    left.datetime
        .cmp(&right.datetime)
        .then_with(|| left.id.cmp(&right.id))
}

fn merge_remote_result(mut merged: Vec<StoredResult>, result: ResultSync) -> Vec<StoredResult> {
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

fn split_sync_batches(mut bundle: SyncBundle) -> Vec<SyncBundle> {
    let results = std::mem::take(&mut bundle.results);
    if results.is_empty() {
        return vec![bundle];
    }
    results
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

fn custom_algs_payload(
    custom_algs: &BTreeMap<String, Vec<UserAlgState>>,
) -> serde_json::Map<String, serde_json::Value> {
    custom_algs
        .iter()
        .map(|(case_id, items)| (case_id.clone(), serde_json::json!({ "items": items })))
        .collect()
}

fn display_algs(
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
            })
        })
        .collect::<Vec<_>>();
    let builtin_ids = case
        .algs
        .iter()
        .map(|alg| alg.alg_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
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
            }),
    );
    result.sort_by_key(|item| item.position);
    result
}

fn upgrade_legacy_custom_algs(case: &TrainerCase, items: Vec<UserAlgState>) -> Vec<UserAlgState> {
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

fn first_display_alg(
    case: &TrainerCase,
    custom_algs: &BTreeMap<String, Vec<UserAlgState>>,
) -> String {
    display_algs(case, custom_algs)
        .first()
        .map(|item| item.alg.clone())
        .unwrap_or_default()
}

fn save_custom_algs(selection_key: &str, custom_algs: &BTreeMap<String, Vec<UserAlgState>>) {
    let key = format!("{selection_key}customAlgs");
    let payload = serde_json::Value::Object(custom_algs_payload(custom_algs)).to_string();
    spawn_local(async move {
        put_stored(key, payload).await;
    });
}

#[function_component(App)]
fn app() -> Html {
    start_sync_engine();
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <Home /> },
        Route::Trainer { id } => html! { <TrainerPage id={id} /> },
        Route::AlgSheet => html! { <AlgSheetPage /> },
        Route::NotFound => html! {
            <main class="colFlex pad"><h1>{"Page not found"}</h1></main>
        },
    }
}

#[function_component(AlgSheetPage)]
fn alg_sheet_page() -> Html {
    let bundle = use_state(|| None::<TrainerBundle>);
    let query = use_state(String::new);
    let custom_algs = use_state(BTreeMap::<String, Vec<UserAlgState>>::new);
    let settings = use_state(|| {
        let mut settings = serde_json::json!({});
        cube_schemes::migrate(&mut settings);
        settings
    });
    {
        let settings = settings.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(value) = get_stored("ALLGSettings").await {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&value) {
                        settings.set(parsed);
                    }
                }
            });
            || ()
        });
    }
    {
        let bundle = bundle.clone();
        use_effect_with((), move |_| {
            let id = web_sys::window()
                .and_then(|window| window.location().search().ok())
                .unwrap_or_default()
                .strip_prefix("?algset=")
                .unwrap_or("")
                .to_owned();
            spawn_local(async move {
                if let Ok(response) = Request::get(&format!("/api/trainers/{id}")).send().await {
                    if let Ok(value) = response.json::<TrainerBundle>().await {
                        bundle.set(Some(value));
                    }
                }
            });
            || ()
        });
    }
    {
        let custom_algs = custom_algs.clone();
        let loaded = bundle.clone();
        let dependency = bundle
            .as_ref()
            .map(|value| value.selection_key.clone())
            .unwrap_or_default();
        use_effect_with(dependency, move |_| {
            if let Some(bundle) = loaded.as_ref() {
                let key = format!("{}customAlgs", bundle.selection_key);
                let cases = bundle.cases.clone();
                spawn_local(async move {
                    let Some(value) = get_stored(&key).await else {
                        return;
                    };
                    let Ok(parsed) =
                        serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&value)
                    else {
                        return;
                    };
                    custom_algs.set(
                        parsed
                            .into_iter()
                            .map(|(case_id, value)| {
                                let mut items = parse_custom_algs(&value, &case_id);
                                if let Some(case) =
                                    cases.iter().find(|case| case.case_id == case_id)
                                {
                                    items = upgrade_legacy_custom_algs(case, items);
                                }
                                (case_id, items)
                            })
                            .collect(),
                    );
                });
            }
            || ()
        });
    }
    let Some(bundle) = bundle.as_ref() else {
        return html! { <main class="colFlex pad"><h1>{"Loading algorithm sheet..."}</h1></main> };
    };
    let needle = query.trim().to_lowercase();
    let cube_colors = cube_schemes::active_colors(&settings, &bundle.trainer_id, &bundle.puzzle);
    let cases = bundle
        .cases
        .iter()
        .map(|case| (case.case_id.clone(), case))
        .collect::<BTreeMap<_, _>>();
    html! {
        <main class="colFlex" style="align-items:center;width:100%;padding:0.5em;">
            <div class="rowFlex center" style="justify-content:space-between;width:min(80vw,62em);">
                <h1>{format!("{} algorithms", bundle.title)}</h1>
                <a class="abutton" href={format!("/trainer/{}", bundle.trainer_id)}>{"Back"}</a>
            </div>
            <div class="borderedContainer pad rowFlex" style="width:min(80vw,62em);">
                <input type="search" placeholder="Search algorithms" style="width:100%;"
                    value={(*query).clone()}
                    oninput={{
                        let query = query.clone();
                        Callback::from(move |event: InputEvent| query.set(event.target_unchecked_into::<web_sys::HtmlInputElement>().value()))
                    }} />
            </div>
            <div class="algsheetGrid">
                {for bundle.algsets.iter().map(|algset| {
                    let needle = needle.clone();
                    let cases = cases.clone();
                    let custom_algs = custom_algs.clone();
                    html! {
                        <>
                            <div class="borderedContainer itemUnsel pad groupNameDiv fullSpan"><b>{&algset.name}</b></div>
                            {for algset.groups.iter().filter_map(|group_name| bundle.groups.iter().find(|group| &group.name == group_name)).map(|group| {
                                let visible_cases = group.cases.iter()
                                    .filter_map(|id| cases.get(id).copied())
                                    .filter(|case| case_matches_search(case, bundle, &needle, &BTreeMap::new(), false))
                                    .collect::<Vec<_>>();
                                if visible_cases.is_empty() {
                                    return Html::default();
                                }
                                html! {
                                    <>
                                        <div class="borderedContainer itemUnsel pad groupNameDiv fullSpan">{&group.name}</div>
                                        {for visible_cases.into_iter().map(|case| html! {
                                            <section class="algsheetCase colFlex">
                                                <div class="itemUnsel borderedContainer algsheetCaseImage">
                                                    <b class="caseName">{&case.name}</b>
                                                    <span class="caseSetup">{case.setup.as_deref().unwrap_or("")}</span>
                                                    if let Some(svg) = case.image_svg.as_ref() {
                                                        <img class="caseImage" src={case_image_data(svg, &cube_colors)} />
                                                    }
                                                </div>
                                                <div class="colFlex algsheetAlgs">
                                                    {for display_algs(case, &custom_algs).into_iter().map(|alg| html! {
                                                        <div class="borderedContainer pad">{alg.alg}</div>
                                                    })}
                                                </div>
                                            </section>
                                        })}
                                    </>
                                }
                            })}
                        </>
                    }
                })}
            </div>
        </main>
    }
}

#[function_component(Home)]
fn home() -> Html {
    let trainers = use_state(Vec::<TrainerSummary>::new);
    let me = use_state(MeResponse::default);
    let error = use_state(|| None::<String>);
    {
        let trainers = trainers.clone();
        let me = me.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(value) = get_stored("ALLGSettings").await {
                    if let Ok(settings) = serde_json::from_str(&value) {
                        apply_settings(&settings);
                    }
                } else {
                    apply_settings(&serde_json::Value::Null);
                }
            });
            spawn_local(async move {
                match Request::get("/api/trainers").send().await {
                    Ok(response) if response.ok() => {
                        match response.json::<Vec<TrainerSummary>>().await {
                            Ok(value) => trainers.set(value),
                            Err(cause) => {
                                error.set(Some(format!("Unable to load trainers: {cause}")))
                            }
                        }
                    }
                    _ => error.set(Some("Unable to load trainers.".to_owned())),
                }
            });
            spawn_local(async move {
                if let Ok(response) = Request::get("/api/me").send().await {
                    if let Ok(value) = response.json::<MeResponse>().await {
                        me.set(value);
                    }
                }
            });
            || ()
        });
    }

    let mut puzzles = Vec::<(String, Vec<TrainerSummary>)>::new();
    for trainer in trainers.iter() {
        match puzzles
            .iter_mut()
            .find(|(puzzle, _)| puzzle == &trainer.puzzle)
        {
            Some((_, entries)) => entries.push(trainer.clone()),
            None => puzzles.push((trainer.puzzle.clone(), vec![trainer.clone()])),
        }
    }
    html! {
        <main class="colFlex" style="background-color: var(--backgroundDarker); align-items:center; width:100%; min-height:100%;">
            <div class="colFlex pad"><h3>{"Welcome to ALLG-Trainer"}</h3></div>
            <div class="rowFlex pad" style="justify-content: center;">
                if me.authenticated {
                    <span class="borderedContainer pad">{format!("Signed in as {}", me.email.as_deref().unwrap_or("WCA user"))}</span>
                    <button class="abutton" onclick={Callback::from(|_| {
                        spawn_local(async move {
                            let _ = Request::post("/api/auth/logout").send().await;
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().reload();
                            }
                        });
                    })}>{"Sign out"}</button>
                } else {
                    <a class="abutton" href="/api/auth/wca">{"Sign in with the WCA"}</a>
                }
            </div>
            <div id="algtrainer-selections" class="rowFlex pad">
                { for puzzles.into_iter().map(|(puzzle, entries)| {
                    let icon = format!("/catalog/{}/icon.png", entries[0].trainer_id);
                    html! {
                        <div class="borderedContainer pad colFlex">
                            <div class="puzzle-header borderedContainer pad" style="background-color: var(--backgroundDarker);">
                                <span>{puzzle}</span>
                                <img style="height: 1.4em;" src={icon} />
                            </div>
                            <div class="rowFlex">
                                { for entries.into_iter().map(|entry| html! {
                                    <Link<Route> classes="abutton grow" to={Route::Trainer { id: entry.trainer_id }}>
                                        <b>{entry.name}</b>
                                    </Link<Route>>
                                }) }
                            </div>
                        </div>
                    }
                }) }
                if let Some(message) = error.as_ref() {
                    <div class="borderedContainer pad">{message}</div>
                }
            </div>
            <span class="version" style="text-align: center;">{"v2.0.0"}</span>
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct TrainerPageProps {
    id: String,
}

#[function_component(TrainerPage)]
fn trainer_page(props: &TrainerPageProps) -> Html {
    let bundle = use_state(|| None::<TrainerBundle>);
    let error = use_state(|| None::<String>);
    {
        let bundle = bundle.clone();
        let error = error.clone();
        let id = props.id.clone();
        use_effect_with(id.clone(), move |_| {
            spawn_local(async move {
                match Request::get(&format!("/api/trainers/{id}")).send().await {
                    Ok(response) if response.ok() => match response.json::<TrainerBundle>().await {
                        Ok(value) => bundle.set(Some(value)),
                        Err(cause) => error.set(Some(format!("Unable to decode trainer: {cause}"))),
                    },
                    _ => error.set(Some("Unable to load this trainer.".to_owned())),
                }
            });
            || ()
        });
    }

    match bundle.as_ref() {
        Some(bundle) => html! { <LoadedTrainer bundle={bundle.clone()} /> },
        None => html! {
            <main class="colFlex pad">
                <h1>{error.as_deref().unwrap_or("Loading trainer...")}</h1>
                <a href="/">{"Back"}</a>
            </main>
        },
    }
}

#[derive(Properties, PartialEq)]
struct LoadedTrainerProps {
    bundle: TrainerBundle,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Select,
    Practice,
}

#[derive(Clone, Copy, PartialEq)]
enum Dialog {
    None,
    Settings,
    Hotkeys,
    Credits,
    Stats,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum AlgsetClickAction {
    Disable,
    SelectAll,
    SelectNone,
}

fn algset_click_action(enabled: bool, selected_count: usize, total: usize) -> AlgsetClickAction {
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

fn last_sync_label() -> String {
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

#[function_component(AccountSettings)]
fn account_settings() -> Html {
    let account = use_state(MeResponse::default);
    let syncing = use_state(|| false);
    let sync_time = use_state(last_sync_label);
    let error = use_state(|| None::<String>);

    {
        let account = account.clone();
        let sync_time = sync_time.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            let account_request = account.clone();
            let request_error = error.clone();
            spawn_local(async move {
                match Request::get("/api/me").send().await {
                    Ok(response) if response.ok() => match response.json::<MeResponse>().await {
                        Ok(value) => account_request.set(value),
                        Err(_) => request_error.set(Some("Unable to load account.".to_owned())),
                    },
                    _ => request_error.set(Some("Unable to load account.".to_owned())),
                }
            });
            let sync_time_listener = sync_time.clone();
            let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                sync_time_listener.set(last_sync_label());
            });
            let window = web_sys::window();
            if let Some(window) = window.as_ref() {
                let _ = window.add_event_listener_with_callback(
                    "alg-trainers-sync",
                    listener.as_ref().unchecked_ref(),
                );
            }
            move || {
                if let Some(window) = window {
                    let _ = window.remove_event_listener_with_callback(
                        "alg-trainers-sync",
                        listener.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    let force_sync = {
        let syncing = syncing.clone();
        let sync_time = sync_time.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let syncing = syncing.clone();
            let sync_time = sync_time.clone();
            let error = error.clone();
            syncing.set(true);
            error.set(None);
            spawn_local(async move {
                let before = last_sync_label();
                sync_now(true).await;
                let after = last_sync_label();
                if before == after {
                    error.set(Some("Sync did not complete. Please try again.".to_owned()));
                }
                sync_time.set(after);
                syncing.set(false);
            });
        })
    };
    let logout = {
        let account = account.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let account = account.clone();
            let error = error.clone();
            spawn_local(async move {
                match Request::post("/api/auth/logout").send().await {
                    Ok(response) if response.ok() => account.set(MeResponse::default()),
                    _ => error.set(Some("Unable to sign out.".to_owned())),
                }
            });
        })
    };

    html! {
        <details class="settingsDetails accountDetails" name="trainer-settings">
            <summary class="settingsSummary">{"Account"}</summary>
            <div class="colFlex accountSection">
                if account.authenticated {
                    <div class="settingsEntry accountIdentity">
                        <b>{account.wca_id.as_deref().unwrap_or("WCA user")}</b>
                        <span>{account.email.as_deref().unwrap_or("")}</span>
                    </div>
                    <div class="settingsEntry accountSync">
                        <div class="colFlex">
                            <b>{"Last sync"}</b>
                            <span>{(*sync_time).clone()}</span>
                        </div>
                        <button class="abutton" disabled={*syncing} onclick={force_sync}>
                            {if *syncing { "Syncing..." } else { "Sync now" }}
                        </button>
                    </div>
                    <div class="settingsEntry">
                        <div></div>
                        <button class="abutton" onclick={logout}>{"Sign out"}</button>
                    </div>
                } else {
                    <div class="settingsEntry accountLogin">
                        <div class="colFlex">
                            <b>{"Not signed in"}</b>
                            <span>{"Sign in to sync settings and results."}</span>
                        </div>
                        <a class="abutton" href="/api/auth/wca">{"Sign in with WCA"}</a>
                    </div>
                }
                if let Some(message) = error.as_ref() {
                    <div class="accountError" role="alert">{message}</div>
                }
            </div>
        </details>
    }
}

#[function_component(LoadedTrainer)]
fn loaded_trainer(props: &LoadedTrainerProps) -> Html {
    let bundle = &props.bundle;
    let selected = use_state(Vec::<String>::new);
    let selected_algsets = use_state(|| {
        bundle
            .algsets
            .iter()
            .map(|algset| (algset.name.clone(), algset.selected))
            .collect::<BTreeMap<_, _>>()
    });
    let recap_remaining = use_state(Vec::<String>::new);
    let recap_active = use_state(|| false);
    let active_case = use_state(|| None::<TrainerCase>);
    let mode = use_state(|| Mode::Select);
    let results = use_state(Vec::<StoredResult>::new);
    let timer_running = use_state(|| false);
    let started_at = use_mut_ref(|| 0.0_f64);
    let elapsed_ms = use_state(|| 0_i64);
    let dialog = use_state(|| Dialog::None);
    let search_open = use_state(|| false);
    let search_query = use_state(String::new);
    let search_cells = use_state(BTreeMap::<usize, String>::new);
    let search_color = use_state(|| search_color_names()[0].to_owned());
    let search_all_angles = use_state(|| true);
    let settings = use_state(|| {
        let mut settings = serde_json::json!({});
        cube_schemes::migrate(&mut settings);
        settings
    });
    let settings_ready = use_state(|| false);
    let presets = use_state(BTreeMap::<String, SelectionPreset>::new);
    let preset_name = use_state(String::new);
    let current_scramble = use_state(String::new);
    let last_case = use_state(|| None::<TrainerCase>);
    let last_scramble = use_state(String::new);
    let hint_open = use_state(|| false);
    let hint_case = use_state(|| None::<TrainerCase>);
    let stats_case_id = use_state(String::new);
    let custom_algs = use_state(BTreeMap::<String, Vec<UserAlgState>>::new);
    let edited_alg = use_state(String::new);
    let alg_editor = use_state(|| None::<(Option<String>, String)>);
    let dragged_alg = use_state(String::new);
    let timer_ready = use_state(|| false);
    let timer_stopped = use_state(|| false);
    let hold_timeout = use_mut_ref(|| None::<Timeout>);
    let interval = use_mut_ref(|| None::<Interval>);
    let stopped_timeout = use_mut_ref(|| None::<Timeout>);
    let trainer_id = bundle.trainer_id.clone();
    let selection_storage_key = bundle.selection_key.clone();
    let times_storage_key = bundle.times_key.clone();

    {
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let results = results.clone();
        let settings = settings.clone();
        let settings_ready = settings_ready.clone();
        let presets = presets.clone();
        let custom_algs = custom_algs.clone();
        let trainer_id = trainer_id.clone();
        let selection_key = selection_storage_key.clone();
        let times_key = times_storage_key.clone();
        let bundle = bundle.clone();
        let recap_active = recap_active.clone();
        let recap_remaining = recap_remaining.clone();
        let active_case = active_case.clone();
        let current_scramble = current_scramble.clone();
        use_effect_with(trainer_id.clone(), move |_| {
            spawn_local(async move {
                let mut loaded_selected = vec!["1".to_owned(), "2".to_owned()];
                if let Some(value) = get_stored(&selection_key).await {
                    if let Ok(parsed) = serde_json::from_str::<Vec<String>>(&value) {
                        loaded_selected = parsed;
                        selected.set(loaded_selected.clone());
                    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&value) {
                        if let Some(cases) =
                            parsed.get("cases").and_then(serde_json::Value::as_array)
                        {
                            loaded_selected = cases
                                .iter()
                                .filter_map(serde_json::Value::as_str)
                                .map(str::to_owned)
                                .collect();
                            selected.set(loaded_selected.clone());
                        }
                        if let Some(algsets) =
                            parsed.get("algsets").and_then(serde_json::Value::as_object)
                        {
                            selected_algsets.set(
                                algsets
                                    .iter()
                                    .filter_map(|(name, value)| {
                                        value.as_bool().map(|value| (name.clone(), value))
                                    })
                                    .collect(),
                            );
                        }
                    }
                } else {
                    selected.set(loaded_selected.clone());
                }
                if let Some(value) = get_stored(&format!("{selection_key}Algsets")).await {
                    if let Ok(parsed) = serde_json::from_str::<BTreeMap<String, bool>>(&value) {
                        selected_algsets.set(parsed);
                    }
                }
                let mut loaded_results = Vec::new();
                if let Some(value) = get_stored(&times_key).await {
                    if let Some((parsed, changed)) = parse_stored_results(&times_key, &value) {
                        if changed {
                            put_stored(
                                times_key.clone(),
                                serde_json::to_string(&parsed).unwrap_or_else(|_| "[]".to_owned()),
                            )
                            .await;
                        }
                        loaded_results = parsed
                            .into_iter()
                            .filter(|result| !result.deleted)
                            .collect();
                        results.set(loaded_results.clone());
                    }
                }
                let mut loaded_settings = serde_json::json!({});
                if let Some(value) = get_stored("ALLGSettings").await {
                    if let Ok(parsed) = serde_json::from_str(&value) {
                        loaded_settings = parsed;
                        if cube_schemes::migrate(&mut loaded_settings) {
                            put_stored("ALLGSettings".to_owned(), loaded_settings.to_string())
                                .await;
                        }
                        settings.set(loaded_settings.clone());
                    }
                }
                if loaded_settings.get("cubeColorSchemes").is_none() {
                    cube_schemes::migrate(&mut loaded_settings);
                    settings.set(loaded_settings.clone());
                }
                settings_ready.set(true);
                if let Some(value) = get_stored(&format!("{selection_key}Presets")).await {
                    if let Ok(parsed) =
                        serde_json::from_str::<BTreeMap<String, SelectionPreset>>(&value)
                    {
                        presets.set(parsed);
                    }
                }
                if let Some(value) = get_stored(&format!("{selection_key}customAlgs")).await {
                    if let Ok(parsed) =
                        serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&value)
                    {
                        let parsed = parsed
                            .into_iter()
                            .map(|(case_id, value)| {
                                let mut items = parse_custom_algs(&value, &case_id);
                                if let Some(case) =
                                    bundle.cases.iter().find(|case| case.case_id == case_id)
                                {
                                    items = upgrade_legacy_custom_algs(case, items);
                                }
                                (case_id, items)
                            })
                            .collect();
                        custom_algs.set(parsed);
                    }
                }
                if let Some(window) = web_sys::window() {
                    let (case_id, remaining, active_recap) =
                        match window.location().search().unwrap_or_default().as_str() {
                            "?recap" => {
                                let mut recap = loaded_selected.clone();
                                let case_id = (!recap.is_empty()).then(|| {
                                    let index =
                                        (Math::random() * recap.len() as f64).floor() as usize;
                                    recap.remove(index)
                                });
                                (case_id, recap, true)
                            }
                            "?train" => (
                                weighted_case(&loaded_selected, &loaded_results, &loaded_settings),
                                Vec::new(),
                                false,
                            ),
                            _ => (None, Vec::new(), false),
                        };
                    recap_remaining.set(remaining);
                    recap_active.set(active_recap);
                    if let Some(case) = case_id
                        .and_then(|id| bundle.cases.iter().find(|case| case.case_id == id).cloned())
                    {
                        current_scramble.set(make_scramble(
                            &case,
                            &bundle,
                            &loaded_settings,
                            &bundle.selection_key,
                        ));
                        active_case.set(Some(case));
                    }
                }
            });
            || ()
        });
    }
    {
        let mode = mode.clone();
        let recap_active = recap_active.clone();
        let recap_remaining = recap_remaining.clone();
        let selected = selected.clone();
        let results = results.clone();
        let settings = settings.clone();
        let active_case = active_case.clone();
        let current_scramble = current_scramble.clone();
        let bundle = bundle.clone();
        use_effect_with((), move |_| {
            let set_from_location = Rc::new({
                let mode = mode.clone();
                let recap_active = recap_active.clone();
                let recap_remaining = recap_remaining.clone();
                let selected = selected.clone();
                let results = results.clone();
                let settings = settings.clone();
                let active_case = active_case.clone();
                let current_scramble = current_scramble.clone();
                let bundle = bundle.clone();
                move || {
                    if let Some(window) = web_sys::window() {
                        let state = window.location().search().unwrap_or_default();
                        let (case_id, remaining, is_recap) = match state.as_str() {
                            "?recap" => {
                                let mut recap = (*selected).clone();
                                let case_id = (!recap.is_empty()).then(|| {
                                    let index =
                                        (Math::random() * recap.len() as f64).floor() as usize;
                                    recap.remove(index)
                                });
                                (case_id, recap, true)
                            }
                            "?train" => (
                                weighted_case(&selected, &results, &settings),
                                Vec::new(),
                                false,
                            ),
                            _ => (None, Vec::new(), false),
                        };
                        if state == "?train" || state == "?recap" {
                            mode.set(Mode::Practice);
                            set_body_background("var(--backgroundDarker)");
                            recap_remaining.set(remaining);
                            recap_active.set(is_recap);
                            if let Some(case) = case_id.and_then(|id| {
                                bundle.cases.iter().find(|case| case.case_id == id).cloned()
                            }) {
                                current_scramble.set(make_scramble(
                                    &case,
                                    &bundle,
                                    &settings,
                                    &bundle.selection_key,
                                ));
                                active_case.set(Some(case));
                            }
                        } else {
                            mode.set(Mode::Select);
                            recap_active.set(false);
                            set_body_background("var(--background)");
                        }
                    }
                }
            }) as Rc<dyn Fn()>;
            set_from_location();
            let set_from_location = set_from_location.clone();
            let listener = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                set_from_location();
            }) as Box<dyn Fn()>);
            if let Some(window) = web_sys::window() {
                let _ = window.add_event_listener_with_callback(
                    "popstate",
                    listener.as_ref().unchecked_ref(),
                );
            }
            listener.forget();
            || ()
        });
    }

    let cases_by_id: Rc<BTreeMap<String, TrainerCase>> = Rc::new(
        bundle
            .cases
            .iter()
            .cloned()
            .map(|case| (case.case_id.clone(), case))
            .collect(),
    );
    let choose_case = {
        let cases_by_id = cases_by_id.clone();
        let selected = selected.clone();
        let recap_remaining = recap_remaining.clone();
        let recap_active = recap_active.clone();
        let active_case = active_case.clone();
        let current_scramble = current_scramble.clone();
        let results = results.clone();
        let settings = settings.clone();
        let bundle = bundle.clone();
        Callback::from(move |_| {
            if selected.is_empty() {
                return;
            }
            let mut recap = (*recap_remaining).clone();
            let id = if *recap_active && !recap.is_empty() {
                let index = (Math::random() * recap.len() as f64).floor() as usize;
                Some(recap.remove(index))
            } else {
                recap_active.set(false);
                weighted_case(&selected, &results, &settings)
            };
            recap_remaining.set(recap);
            let case = id.and_then(|id| cases_by_id.get(&id)).cloned();
            if let Some(case) = case.as_ref() {
                current_scramble.set(make_scramble(
                    case,
                    &bundle,
                    &settings,
                    &bundle.selection_key,
                ));
            }
            active_case.set(case);
        })
    };

    let toggle_case = {
        let selected = selected.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |case_id: String| {
            let mut next = (*selected).clone();
            if let Some(index) = next.iter().position(|id| id == &case_id) {
                next.remove(index);
            } else {
                next.push(case_id);
            }
            selected.set(next.clone());
            let key = selection_key.clone();
            spawn_local(async move {
                put_stored(key, serde_json::to_string(&next).unwrap()).await;
            });
        })
    };

    let start_training = {
        let mode = mode.clone();
        let selected = selected.clone();
        let recap_remaining = recap_remaining.clone();
        let recap_active = recap_active.clone();
        let cases_by_id = cases_by_id.clone();
        let active_case = active_case.clone();
        let current_scramble = current_scramble.clone();
        let settings = settings.clone();
        let bundle = bundle.clone();
        Callback::from(move |_| {
            if selected.is_empty() {
                return;
            }
            let mut recap = (*selected).clone();
            let index = (Math::random() * recap.len() as f64).floor() as usize;
            let case_id = recap.remove(index);
            recap_remaining.set(recap);
            recap_active.set(true);
            if let Some(case) = cases_by_id.get(&case_id).cloned() {
                current_scramble.set(make_scramble(
                    &case,
                    &bundle,
                    &settings,
                    &bundle.selection_key,
                ));
                active_case.set(Some(case));
            }
            set_body_background("var(--backgroundDarker)");
            mode.set(Mode::Practice);
            if let Some(window) = web_sys::window() {
                let history = window.history();
                if let Ok(history) = history {
                    let _ = history.push_state_with_url(
                        &JsValue::from_str("recap"),
                        "",
                        Some("?recap"),
                    );
                }
            }
            Timeout::new(0, || {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Ok(Some(main)) = document.query_selector("#trainer-main") {
                            let _ = main.unchecked_into::<web_sys::HtmlElement>().focus();
                        }
                    }
                }
            })
            .forget();
        })
    };
    let switch_to_train = {
        let recap_remaining = recap_remaining.clone();
        let recap_active = recap_active.clone();
        let selected = selected.clone();
        let results = results.clone();
        let settings = settings.clone();
        let cases_by_id = cases_by_id.clone();
        let active_case = active_case.clone();
        let current_scramble = current_scramble.clone();
        let bundle = bundle.clone();
        Callback::from(move |_| {
            recap_remaining.set(Vec::new());
            recap_active.set(false);
            if let Some(window) = web_sys::window() {
                if let Ok(history) = window.history() {
                    let _ = history.replace_state_with_url(
                        &JsValue::from_str("train"),
                        "",
                        Some("?train"),
                    );
                }
            }
            if let Some(case_id) = weighted_case(&selected, &results, &settings) {
                if let Some(case) = cases_by_id.get(&case_id).cloned() {
                    current_scramble.set(make_scramble(
                        &case,
                        &bundle,
                        &settings,
                        &bundle.selection_key,
                    ));
                    active_case.set(Some(case));
                }
            }
        })
    };

    let finish_timer = {
        let timer_running = timer_running.clone();
        let started_at = started_at.clone();
        let elapsed_ms = elapsed_ms.clone();
        let interval = interval.clone();
        let active_case = active_case.clone();
        let results = results.clone();
        let times_key = times_storage_key.clone();
        let choose_case = choose_case.clone();
        let timer_ready = timer_ready.clone();
        let last_case = last_case.clone();
        let last_scramble = last_scramble.clone();
        let current_scramble = current_scramble.clone();
        let timer_stopped = timer_stopped.clone();
        let stopped_timeout = stopped_timeout.clone();
        Callback::from(move |_| {
            if !*timer_running {
                return;
            }
            let ms = (Date::now() - *started_at.borrow()).round() as i64;
            elapsed_ms.set(ms);
            timer_running.set(false);
            timer_ready.set(false);
            timer_stopped.set(true);
            if let Some(timeout) = stopped_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            let timer_stopped = timer_stopped.clone();
            *stopped_timeout.borrow_mut() =
                Some(Timeout::new(100, move || timer_stopped.set(false)));
            if let Some(timer) = interval.borrow_mut().take() {
                timer.cancel();
            }
            if let Some(case) = active_case.as_ref() {
                last_case.set(Some(case.clone()));
                last_scramble.set((*current_scramble).clone());
                let mut next = (*results).clone();
                next.push(StoredResult {
                    id: make_uuid(),
                    datetime: Date::new_0()
                        .to_iso_string()
                        .as_string()
                        .unwrap_or_default(),
                    ms,
                    case: case.case_id.clone(),
                    time: format_time(ms),
                    deleted: false,
                });
                results.set(next.clone());
                let key = times_key.clone();
                spawn_local(async move {
                    put_stored_results(key, next, Vec::new()).await;
                });
            }
            choose_case.emit(());
        })
    };

    let abort_timer = {
        let timer_running = timer_running.clone();
        let timer_ready = timer_ready.clone();
        let timer_stopped = timer_stopped.clone();
        let interval = interval.clone();
        let hold_timeout = hold_timeout.clone();
        let elapsed_ms = elapsed_ms.clone();
        let stopped_timeout = stopped_timeout.clone();
        Callback::from(move |_| {
            timer_running.set(false);
            timer_ready.set(false);
            timer_stopped.set(false);
            elapsed_ms.set(0);
            if let Some(timeout) = stopped_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            if let Some(timer) = interval.borrow_mut().take() {
                timer.cancel();
            }
            if let Some(timeout) = hold_timeout.borrow_mut().take() {
                timeout.cancel();
            }
        })
    };

    let key_down = {
        let timer_running = timer_running.clone();
        let finish_timer = finish_timer.clone();
        let timer_ready = timer_ready.clone();
        let hold_timeout = hold_timeout.clone();
        let settings = settings.clone();
        let dialog = dialog.clone();
        let hint_open = hint_open.clone();
        let mode = mode.clone();
        let active_case = active_case.clone();
        let last_case = last_case.clone();
        let hint_case = hint_case.clone();
        let custom_algs = custom_algs.clone();
        let edited_alg = edited_alg.clone();
        let results = results.clone();
        let selected = selected.clone();
        let presets = presets.clone();
        let selection_key = selection_storage_key.clone();
        let start_training = start_training.clone();
        let switch_to_train = switch_to_train.clone();
        let recap_active = recap_active.clone();
        let times_key = times_storage_key.clone();
        let abort_timer = abort_timer.clone();
        let cases = bundle.cases.clone();
        Callback::from(move |event: KeyboardEvent| {
            let is_typing = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .map(|element| {
                    matches!(
                        element.tag_name().to_ascii_lowercase().as_str(),
                        "input" | "textarea" | "select"
                    )
                })
                .unwrap_or(false);
            if event.code() == "Escape" {
                if *dialog != Dialog::None {
                    dialog.set(Dialog::None);
                } else if *hint_open {
                    hint_open.set(false);
                } else if *timer_running {
                    abort_timer.emit(());
                }
                return;
            }
            if *timer_running && !is_typing && *dialog == Dialog::None && !*hint_open {
                event.prevent_default();
                finish_timer.emit(());
                return;
            }
            if *hint_open {
                match event.code().as_str() {
                    "ArrowLeft" => {
                        event.prevent_default();
                        if let Some(current) = hint_case.as_ref() {
                            if let Some(index) = cases
                                .iter()
                                .position(|case| case.case_id == current.case_id)
                            {
                                if index > 0 {
                                    let case = cases[index - 1].clone();
                                    edited_alg.set(first_display_alg(&case, &custom_algs));
                                    hint_case.set(Some(case));
                                }
                            }
                        }
                    }
                    "ArrowRight" => {
                        event.prevent_default();
                        if let Some(current) = hint_case.as_ref() {
                            if let Some(index) = cases
                                .iter()
                                .position(|case| case.case_id == current.case_id)
                            {
                                if let Some(case) = cases.get(index + 1).cloned() {
                                    edited_alg.set(first_display_alg(&case, &custom_algs));
                                    hint_case.set(Some(case));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            if is_typing || *dialog != Dialog::None || *hint_open {
                return;
            }
            if *timer_running {
                event.prevent_default();
                finish_timer.emit(());
                return;
            }
            if *mode == Mode::Practice && !*timer_running {
                match event.code().as_str() {
                    "KeyX" => {
                        if *recap_active {
                            switch_to_train.emit(());
                        } else {
                            start_training.emit(());
                        }
                        return;
                    }
                    "KeyH" => {
                        let case = if event.shift_key() {
                            last_case.as_ref()
                        } else {
                            active_case.as_ref()
                        };
                        if let Some(case) = case {
                            let alg = first_display_alg(&case, &custom_algs);
                            edited_alg.set(alg);
                            hint_case.set(Some(case.clone()));
                            hint_open.set(true);
                        }
                        return;
                    }
                    "KeyB" => {
                        if let Some(case) = last_case.as_ref() {
                            let mut next = (*presets).clone();
                            let preset = next.entry("Bookmarks".to_owned()).or_insert_with(|| {
                                SelectionPreset {
                                    sel_cases: Vec::new(),
                                    selected_alg_sets: BTreeMap::new(),
                                }
                            });
                            if let Some(index) =
                                preset.sel_cases.iter().position(|id| id == &case.case_id)
                            {
                                preset.sel_cases.remove(index);
                            } else {
                                preset.sel_cases.push(case.case_id.clone());
                            }
                            if preset.sel_cases.is_empty() {
                                next.remove("Bookmarks");
                            }
                            presets.set(next.clone());
                            let key = format!("{selection_key}Presets");
                            spawn_local(async move {
                                put_stored(key, serde_json::to_string(&next).unwrap()).await;
                            });
                        }
                        return;
                    }
                    "KeyR" => {
                        if let Some(case) = last_case.as_ref() {
                            let mut next = (*selected).clone();
                            if let Some(index) = next.iter().position(|id| id == &case.case_id) {
                                next.remove(index);
                            } else {
                                next.push(case.case_id.clone());
                            }
                            selected.set(next.clone());
                            let key = selection_key.clone();
                            spawn_local(async move {
                                put_stored(key, serde_json::to_string(&next).unwrap()).await;
                            });
                        }
                        return;
                    }
                    "Delete" => {
                        let mut next = (*results).clone();
                        let deleted_ids = if event.shift_key() {
                            let deleted = next.iter().map(|result| result.id.clone()).collect();
                            next.clear();
                            deleted
                        } else {
                            next.pop().map(|result| vec![result.id]).unwrap_or_default()
                        };
                        results.set(next.clone());
                        let times_key = times_key.clone();
                        spawn_local(async move {
                            put_stored_results(times_key, next, deleted_ids).await;
                        });
                        return;
                    }
                    _ => {}
                }
            }
            if event.code() != "Space" {
                return;
            }
            event.prevent_default();
            if *timer_running {
                finish_timer.emit(());
            } else if hold_timeout.borrow().is_none() && !*timer_ready {
                let timer_ready = timer_ready.clone();
                let delay = if settings
                    .get("enableHoldDelay")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true)
                {
                    200
                } else {
                    0
                };
                *hold_timeout.borrow_mut() = Some(Timeout::new(delay, move || {
                    timer_ready.set(true);
                }));
            }
        })
    };
    let key_up = {
        let timer_running = timer_running.clone();
        let timer_ready = timer_ready.clone();
        let hold_timeout = hold_timeout.clone();
        let started_at = started_at.clone();
        let elapsed_ms = elapsed_ms.clone();
        let interval = interval.clone();
        let timer_stopped = timer_stopped.clone();
        let stopped_timeout = stopped_timeout.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.code() != "Space" {
                return;
            }
            if *timer_running {
                return;
            }
            if let Some(timeout) = hold_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            if !*timer_ready {
                return;
            }
            timer_ready.set(false);
            timer_stopped.set(false);
            if let Some(timeout) = stopped_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            let start = Date::now();
            *started_at.borrow_mut() = start;
            timer_running.set(true);
            elapsed_ms.set(0);
            let elapsed_ms = elapsed_ms.clone();
            let started_at = started_at.clone();
            *interval.borrow_mut() = Some(Interval::new(20, move || {
                elapsed_ms.set((Date::now() - *started_at.borrow()).round() as i64);
            }));
        })
    };
    let pointer_down = {
        let timer_running = timer_running.clone();
        let finish_timer = finish_timer.clone();
        let timer_ready = timer_ready.clone();
        let hold_timeout = hold_timeout.clone();
        let settings = settings.clone();
        let timer_stopped = timer_stopped.clone();
        let stopped_timeout = stopped_timeout.clone();
        Callback::from(move |event: web_sys::PointerEvent| {
            if event.pointer_type() != "touch" && event.pointer_type() != "pen" {
                return;
            }
            event.prevent_default();
            if *timer_running {
                finish_timer.emit(());
                return;
            }
            if hold_timeout.borrow().is_none() && !*timer_ready {
                timer_stopped.set(false);
                if let Some(timeout) = stopped_timeout.borrow_mut().take() {
                    timeout.cancel();
                }
                let timer_ready = timer_ready.clone();
                let delay = if settings
                    .get("enableHoldDelay")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true)
                {
                    200
                } else {
                    0
                };
                *hold_timeout.borrow_mut() =
                    Some(Timeout::new(delay, move || timer_ready.set(true)));
            }
        })
    };
    let pointer_up = {
        let timer_running = timer_running.clone();
        let timer_ready = timer_ready.clone();
        let hold_timeout = hold_timeout.clone();
        let started_at = started_at.clone();
        let elapsed_ms = elapsed_ms.clone();
        let interval = interval.clone();
        let timer_stopped = timer_stopped.clone();
        let stopped_timeout = stopped_timeout.clone();
        Callback::from(move |event: web_sys::PointerEvent| {
            if event.pointer_type() != "touch" && event.pointer_type() != "pen" {
                return;
            }
            event.prevent_default();
            if *timer_running {
                return;
            }
            if let Some(timeout) = hold_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            if !*timer_ready {
                elapsed_ms.set(0);
                return;
            }
            timer_ready.set(false);
            timer_stopped.set(false);
            if let Some(timeout) = stopped_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            let start = Date::now();
            *started_at.borrow_mut() = start;
            timer_running.set(true);
            elapsed_ms.set(0);
            let elapsed_ms = elapsed_ms.clone();
            let started_at = started_at.clone();
            *interval.borrow_mut() = Some(Interval::new(20, move || {
                elapsed_ms.set((Date::now() - *started_at.borrow()).round() as i64);
            }));
        })
    };
    let pointer_cancel = {
        let timer_ready = timer_ready.clone();
        let hold_timeout = hold_timeout.clone();
        Callback::from(move |event: web_sys::PointerEvent| {
            event.prevent_default();
            if let Some(timeout) = hold_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            timer_ready.set(false);
        })
    };
    let open_hint = {
        let hint_open = hint_open.clone();
        let hint_case = hint_case.clone();
        let edited_alg = edited_alg.clone();
        let custom_algs = custom_algs.clone();
        let active_case = active_case.clone();
        Callback::from(move |_| {
            if let Some(case) = active_case.as_ref() {
                let alg = first_display_alg(&case, &custom_algs);
                edited_alg.set(alg);
                hint_case.set(Some(case.clone()));
                hint_open.set(true);
            }
        })
    };
    let open_hint_for_case = {
        let hint_open = hint_open.clone();
        let hint_case = hint_case.clone();
        let edited_alg = edited_alg.clone();
        let custom_algs = custom_algs.clone();
        Callback::from(move |case: TrainerCase| {
            let alg = first_display_alg(&case, &custom_algs);
            edited_alg.set(alg);
            hint_case.set(Some(case));
            hint_open.set(true);
        })
    };
    let open_last_hint = {
        let hint_open = hint_open.clone();
        let hint_case = hint_case.clone();
        let edited_alg = edited_alg.clone();
        let custom_algs = custom_algs.clone();
        let last_case = last_case.clone();
        Callback::from(move |_| {
            if let Some(case) = last_case.as_ref() {
                let alg = first_display_alg(case, &custom_algs);
                edited_alg.set(alg);
                hint_case.set(Some(case.clone()));
                hint_open.set(true);
            }
        })
    };
    let open_stats = {
        let dialog = dialog.clone();
        let stats_case_id = stats_case_id.clone();
        Callback::from(move |case_id: String| {
            stats_case_id.set(case_id);
            dialog.set(Dialog::Stats);
        })
    };
    let previous_hint = {
        let hint_case = hint_case.clone();
        let hint_open = hint_open.clone();
        let edited_alg = edited_alg.clone();
        let custom_algs = custom_algs.clone();
        let cases = bundle.cases.clone();
        Callback::from(move |_| {
            if let Some(case) = hint_case.as_ref() {
                if let Some(index) = cases.iter().position(|item| item.case_id == case.case_id) {
                    if index > 0 {
                        let previous = cases[index - 1].clone();
                        let alg = first_display_alg(&previous, &custom_algs);
                        edited_alg.set(alg);
                        hint_case.set(Some(previous));
                        hint_open.set(true);
                    }
                }
            }
        })
    };
    let next_hint = {
        let hint_case = hint_case.clone();
        let hint_open = hint_open.clone();
        let edited_alg = edited_alg.clone();
        let custom_algs = custom_algs.clone();
        let cases = bundle.cases.clone();
        Callback::from(move |_| {
            if let Some(case) = hint_case.as_ref() {
                if let Some(index) = cases.iter().position(|item| item.case_id == case.case_id) {
                    if index + 1 < cases.len() {
                        let next = cases[index + 1].clone();
                        let alg = first_display_alg(&next, &custom_algs);
                        edited_alg.set(alg);
                        hint_case.set(Some(next));
                        hint_open.set(true);
                    }
                }
            }
        })
    };
    let save_editor = {
        let custom_algs = custom_algs.clone();
        let hint_case = hint_case.clone();
        let alg_editor = alg_editor.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |_| {
            let Some(case) = hint_case.as_ref() else {
                return;
            };
            let Some((alg_id, value)) = alg_editor.as_ref() else {
                return;
            };
            let value = value.trim().to_owned();
            if value.is_empty() {
                return;
            }
            let mut next = (*custom_algs).clone();
            if let Some(alg_id) = alg_id {
                let current = next.entry(case.case_id.clone()).or_default();
                if let Some(item) = current.iter_mut().find(|item| &item.alg_id == alg_id) {
                    item.alg = value;
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
                        alg_id: randomAlgId(),
                        alg: value,
                        position,
                        hidden: false,
                        deleted: false,
                    });
            }
            custom_algs.set(next.clone());
            save_custom_algs(&selection_key, &next);
            alg_editor.set(None);
        })
    };
    let hide_alg = {
        let custom_algs = custom_algs.clone();
        let hint_case = hint_case.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |item: DisplayAlg| {
            let Some(case) = hint_case.as_ref() else {
                return;
            };
            let mut next = (*custom_algs).clone();
            let current = next.entry(case.case_id.clone()).or_default();
            if let Some(existing) = current.iter_mut().find(|alg| alg.alg_id == item.alg_id) {
                existing.hidden = true;
            } else {
                current.push(UserAlgState {
                    alg_id: item.alg_id,
                    alg: item.alg,
                    position: item.position,
                    hidden: true,
                    deleted: false,
                });
            }
            custom_algs.set(next.clone());
            save_custom_algs(&selection_key, &next);
        })
    };
    let delete_alg = {
        let custom_algs = custom_algs.clone();
        let hint_case = hint_case.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |alg_id: String| {
            let Some(case) = hint_case.as_ref() else {
                return;
            };
            let mut next = (*custom_algs).clone();
            if let Some(item) = next
                .entry(case.case_id.clone())
                .or_default()
                .iter_mut()
                .find(|item| item.alg_id == alg_id)
            {
                item.deleted = true;
            }
            custom_algs.set(next.clone());
            save_custom_algs(&selection_key, &next);
        })
    };
    let reorder_alg = {
        let custom_algs = custom_algs.clone();
        let hint_case = hint_case.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |(from, to): (String, String)| {
            let Some(case) = hint_case.as_ref() else {
                return;
            };
            let mut items = display_algs(case, &custom_algs);
            let Some(from_index) = items.iter().position(|item| item.alg_id == from) else {
                return;
            };
            let Some(to_index) = items.iter().position(|item| item.alg_id == to) else {
                return;
            };
            let moved = items.remove(from_index);
            items.insert(to_index, moved);
            let mut next = (*custom_algs).clone();
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
            custom_algs.set(next.clone());
            save_custom_algs(&selection_key, &next);
        })
    };
    let select_all = {
        let selected = selected.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |all: Vec<String>| {
            let mut next = (*selected).clone();
            if all.iter().all(|id| next.contains(id)) {
                next.retain(|id| !all.contains(id));
            } else {
                for id in all {
                    if !next.contains(&id) {
                        next.push(id);
                    }
                }
            }
            selected.set(next.clone());
            let key = selection_key.clone();
            spawn_local(async move {
                put_stored(key, serde_json::to_string(&next).unwrap()).await;
            });
        })
    };
    let toggle_group = {
        let selected = selected.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |case_ids: Vec<String>| {
            let mut next = (*selected).clone();
            let all_selected = case_ids.iter().all(|id| next.contains(id));
            if all_selected {
                next.retain(|id| !case_ids.contains(id));
            } else {
                for id in case_ids {
                    if !next.contains(&id) {
                        next.push(id);
                    }
                }
            }
            selected.set(next.clone());
            let key = selection_key.clone();
            spawn_local(async move {
                put_stored(key, serde_json::to_string(&next).unwrap()).await;
            });
        })
    };
    let toggle_algset = {
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |(name, group_case_ids): (String, Vec<String>)| {
            let mut algsets = (*selected_algsets).clone();
            let mut cases = (*selected).clone();
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
                        if !cases.contains(&id) {
                            cases.push(id);
                        }
                    }
                }
                AlgsetClickAction::SelectNone => {
                    cases.retain(|id| !group_case_ids.contains(id));
                }
            }
            selected.set(cases.clone());
            selected_algsets.set(algsets.clone());
            let key = selection_key.clone();
            spawn_local(async move {
                put_stored(key.clone(), serde_json::to_string(&cases).unwrap()).await;
                put_stored(
                    format!("{key}Algsets"),
                    serde_json::to_string(&algsets).unwrap(),
                )
                .await;
            });
        })
    };
    let use_preset = {
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |preset: SelectionPreset| {
            let case_ids = preset.sel_cases;
            let algsets = preset.selected_alg_sets;
            selected.set(case_ids.clone());
            selected_algsets.set(algsets.clone());
            let key = selection_key.clone();
            spawn_local(async move {
                put_stored(key.clone(), serde_json::to_string(&case_ids).unwrap()).await;
                put_stored(
                    format!("{key}Algsets"),
                    serde_json::to_string(&algsets).unwrap(),
                )
                .await;
            });
        })
    };
    let add_preset_to_selection = {
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |preset: SelectionPreset| {
            let mut cases = (*selected).clone();
            for case_id in preset.sel_cases {
                if !cases.contains(&case_id) {
                    cases.push(case_id);
                }
            }
            let mut algsets = (*selected_algsets).clone();
            algsets.extend(preset.selected_alg_sets);
            selected.set(cases.clone());
            selected_algsets.set(algsets.clone());
            let key = selection_key.clone();
            spawn_local(async move {
                put_stored(key.clone(), serde_json::to_string(&cases).unwrap()).await;
                put_stored(
                    format!("{key}Algsets"),
                    serde_json::to_string(&algsets).unwrap(),
                )
                .await;
            });
        })
    };
    let remove_preset_from_selection = {
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |preset: SelectionPreset| {
            let cases = (*selected)
                .clone()
                .into_iter()
                .filter(|id| !preset.sel_cases.contains(id))
                .collect::<Vec<_>>();
            let mut algsets = (*selected_algsets).clone();
            for (name, value) in preset.selected_alg_sets {
                if !value {
                    algsets.insert(name, false);
                }
            }
            selected.set(cases.clone());
            selected_algsets.set(algsets.clone());
            let key = selection_key.clone();
            spawn_local(async move {
                put_stored(key.clone(), serde_json::to_string(&cases).unwrap()).await;
                put_stored(
                    format!("{key}Algsets"),
                    serde_json::to_string(&algsets).unwrap(),
                )
                .await;
            });
        })
    };
    let add_preset = {
        let presets = presets.clone();
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let preset_name = preset_name.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |_| {
            let name = preset_name.trim().to_owned();
            if name.is_empty() {
                return;
            }
            let mut next = (*presets).clone();
            next.insert(
                name,
                SelectionPreset {
                    sel_cases: (*selected).clone(),
                    selected_alg_sets: (*selected_algsets).clone(),
                },
            );
            presets.set(next.clone());
            preset_name.set(String::new());
            let key = format!("{selection_key}Presets");
            spawn_local(async move {
                put_stored(key, serde_json::to_string(&next).unwrap()).await;
            });
        })
    };
    let update_preset = {
        let presets = presets.clone();
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |name: String| {
            let mut next = (*presets).clone();
            next.insert(
                name,
                SelectionPreset {
                    sel_cases: (*selected).clone(),
                    selected_alg_sets: (*selected_algsets).clone(),
                },
            );
            presets.set(next.clone());
            let key = format!("{selection_key}Presets");
            spawn_local(async move {
                put_stored(key, serde_json::to_string(&next).unwrap()).await;
            });
        })
    };
    let delete_preset = {
        let presets = presets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |name: String| {
            let mut next = (*presets).clone();
            next.remove(&name);
            presets.set(next.clone());
            let key = format!("{selection_key}Presets");
            spawn_local(async move {
                put_stored(key, serde_json::to_string(&next).unwrap()).await;
            });
        })
    };
    let open_dialog = {
        let dialog = dialog.clone();
        Callback::from(move |next: Dialog| dialog.set(next))
    };
    let close_dialog = {
        let dialog = dialog.clone();
        Callback::from(move |_| dialog.set(Dialog::None))
    };
    let close_dialog_background = {
        let dialog = dialog.clone();
        Callback::from(move |event: web_sys::MouseEvent| {
            if event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
                .is_some_and(|target| target.tag_name() == "DIALOG")
            {
                dialog.set(Dialog::None);
            }
        })
    };
    let close_hint_background = {
        let hint_open = hint_open.clone();
        Callback::from(move |event: web_sys::MouseEvent| {
            if event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
                .is_some_and(|target| target.tag_name() == "DIALOG")
            {
                hint_open.set(false);
            }
        })
    };
    let open_settings = {
        let open_dialog = open_dialog.clone();
        Callback::from(move |_| open_dialog.emit(Dialog::Settings))
    };
    let export_backup = Callback::from(move |_| {
        spawn_local(async move {
            let Ok(promise) = storageAll("data") else {
                return;
            };
            let Ok(value) = JsFuture::from(promise).await else {
                return;
            };
            let Ok(data) = serde_wasm_bindgen::from_value::<BTreeMap<String, String>>(value) else {
                return;
            };
            let payload = serde_json::json!({ "version": 1, "data": data });
            downloadText(
                "alg-trainers-backup.json",
                &serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_owned()),
            );
        });
    });
    let import_backup = Callback::from(move |event: Event| {
        let input = event.target_unchecked_into::<web_sys::HtmlInputElement>();
        let Some(file) = input.files().and_then(|files| files.get(0)) else {
            return;
        };
        spawn_local(async move {
            let result = async {
                let promise = readFileText(&file).map_err(|_| "Unable to read backup file")?;
                let text = JsFuture::from(promise)
                    .await
                    .map_err(|_| "Unable to read backup file")?
                    .as_string()
                    .ok_or("Unable to read backup file")?;
                let promise = importStorageText(&text).map_err(|_| "Invalid trainer backup")?;
                let _ = JsFuture::from(promise)
                    .await
                    .map_err(|_| "Unable to import trainer backup")?;
                Ok::<(), &str>(())
            }
            .await;
            if let Some(window) = web_sys::window() {
                match result {
                    Ok(()) => {
                        let _ = window.alert_with_message(
                            "Backup imported. Reload the page to load the restored data.",
                        );
                    }
                    Err(message) => {
                        let _ = window.alert_with_message(message);
                    }
                }
            }
        });
    });
    let import_letter_pairs = {
        let settings = settings.clone();
        Callback::from(move |event: Event| {
            let input = event.target_unchecked_into::<web_sys::HtmlInputElement>();
            let Some(file) = input.files().and_then(|files| files.get(0)) else {
                return;
            };
            let settings = settings.clone();
            spawn_local(async move {
                let result = async {
                    let promise =
                        readFileText(&file).map_err(|_| "Unable to read letter-pair file")?;
                    let text = JsFuture::from(promise)
                        .await
                        .map_err(|_| "Unable to read letter-pair file")?
                        .as_string()
                        .ok_or("Unable to read letter-pair file")?;
                    let pairs = serde_json::from_str::<BTreeMap<String, String>>(&text)
                        .map_err(|_| "Invalid letter-pair JSON")?;
                    let mut next = (*settings).clone();
                    next["letterpairs"] = serde_json::to_value(pairs).unwrap();
                    settings.set(next.clone());
                    put_stored("ALLGSettings".to_owned(), next.to_string()).await;
                    Ok::<(), &str>(())
                }
                .await;
                if let Some(window) = web_sys::window() {
                    let _ = match result {
                        Ok(()) => window.alert_with_message("Letter-pair words imported."),
                        Err(message) => window.alert_with_message(message),
                    };
                }
            });
        })
    };
    let remove_last_time = {
        let results = results.clone();
        let times_key = times_storage_key.clone();
        Callback::from(move |_| {
            let mut next = (*results).clone();
            let deleted_ids = next.pop().map(|result| vec![result.id]).unwrap_or_default();
            results.set(next.clone());
            let times_key = times_key.clone();
            spawn_local(async move {
                put_stored_results(times_key, next, deleted_ids).await;
            });
        })
    };
    let remove_last_case = {
        let selected = selected.clone();
        let last_case = last_case.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |_| {
            let Some(case) = last_case.as_ref() else {
                return;
            };
            let mut next = (*selected).clone();
            if let Some(index) = next.iter().position(|id| id == &case.case_id) {
                next.remove(index);
            } else {
                next.push(case.case_id.clone());
            }
            selected.set(next.clone());
            let selection_key = selection_key.clone();
            spawn_local(async move {
                put_stored(
                    selection_key,
                    serde_json::to_string(&next).unwrap_or_else(|_| "[]".to_owned()),
                )
                .await;
            });
        })
    };
    let bookmark_last_case = {
        let presets = presets.clone();
        let last_case = last_case.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |_| {
            let Some(case) = last_case.as_ref() else {
                return;
            };
            let mut next = (*presets).clone();
            let preset = next
                .entry("Bookmarks".to_owned())
                .or_insert_with(|| SelectionPreset {
                    sel_cases: Vec::new(),
                    selected_alg_sets: BTreeMap::new(),
                });
            if let Some(index) = preset.sel_cases.iter().position(|id| id == &case.case_id) {
                preset.sel_cases.remove(index);
            } else {
                preset.sel_cases.push(case.case_id.clone());
            }
            if preset.sel_cases.is_empty() {
                next.remove("Bookmarks");
            }
            presets.set(next.clone());
            let key = format!("{selection_key}Presets");
            spawn_local(async move {
                put_stored(
                    key,
                    serde_json::to_string(&next).unwrap_or_else(|_| "{}".to_owned()),
                )
                .await;
            });
        })
    };
    let change_setting = {
        let settings = settings.clone();
        Callback::from(move |(key, value): (String, bool)| {
            let mut next = (*settings).clone();
            next[&key] = value.into();
            settings.set(next.clone());
            spawn_local(async move {
                put_stored("ALLGSettings".to_owned(), next.to_string()).await;
            });
        })
    };
    let adjust_setting = {
        let settings = settings.clone();
        Callback::from(move |(key, amount): (&'static str, f64)| {
            let mut next = (*settings).clone();
            let current = next
                .get(key)
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_else(|| match key {
                    "timerSize" => 5.0,
                    "scrambleSize" => 2.0,
                    _ => 1.0,
                });
            next[key] = serde_json::json!((current + amount).max(0.5));
            settings.set(next.clone());
            spawn_local(async move {
                put_stored("ALLGSettings".to_owned(), next.to_string()).await;
            });
        })
    };
    let change_setting_value = {
        let settings = settings.clone();
        Callback::from(move |(key, value): (String, serde_json::Value)| {
            let mut next = (*settings).clone();
            next[&key] = value;
            settings.set(next.clone());
            spawn_local(async move {
                put_stored("ALLGSettings".to_owned(), next.to_string()).await;
            });
        })
    };
    let reset_theme = {
        let settings = settings.clone();
        Callback::from(move |theme: &'static str| {
            let mut next = (*settings).clone();
            let colors = default_theme_colors(theme);
            next["colors"] = serde_json::to_value(colors).unwrap();
            settings.set(next.clone());
            spawn_local(async move {
                put_stored("ALLGSettings".to_owned(), next.to_string()).await;
            });
        })
    };
    let change_color = {
        let settings = settings.clone();
        Callback::from(move |(key, value): (String, String)| {
            let mut next = (*settings).clone();
            if next
                .get("colors")
                .and_then(serde_json::Value::as_object)
                .is_none()
            {
                next["colors"] = serde_json::json!({});
            }
            next["colors"][&key] = value.into();
            settings.set(next.clone());
            spawn_local(async move {
                put_stored("ALLGSettings".to_owned(), next.to_string()).await;
            });
        })
    };
    let change_cube_settings = {
        let settings = settings.clone();
        Callback::from(move |next: serde_json::Value| {
            settings.set(next.clone());
            spawn_local(async move {
                put_stored("ALLGSettings".to_owned(), next.to_string()).await;
            });
        })
    };
    let change_scramble_override = {
        let settings = settings.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |(option, value): (&'static str, String)| {
            let mut next = (*settings).clone();
            if next
                .get("trainerScrambleOverrides")
                .and_then(serde_json::Value::as_object)
                .is_none()
            {
                next["trainerScrambleOverrides"] = serde_json::json!({});
            }
            if next["trainerScrambleOverrides"]
                .get(&selection_key)
                .and_then(serde_json::Value::as_object)
                .is_none()
            {
                next["trainerScrambleOverrides"][&selection_key] = serde_json::json!({});
            }
            let options = value
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>();
            if options.is_empty() {
                if let Some(overrides) =
                    next["trainerScrambleOverrides"][&selection_key].as_object_mut()
                {
                    overrides.remove(option);
                }
            } else {
                next["trainerScrambleOverrides"][&selection_key][option] =
                    serde_json::to_value(options).unwrap();
            }
            settings.set(next.clone());
            spawn_local(async move {
                put_stored("ALLGSettings".to_owned(), next.to_string()).await;
            });
        })
    };
    {
        let settings = settings.clone();
        use_effect_with(settings.clone(), move |settings| {
            apply_settings(settings);
            || ()
        });
    }

    let search_needle = search_query.trim().to_lowercase();
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
        .collect::<std::collections::BTreeSet<_>>();
    let visible_case_ids = bundle
        .cases
        .iter()
        .filter(|case| enabled_case_ids.contains(&case.case_id))
        .filter(|case| {
            case_matches_search(
                case,
                bundle,
                &search_needle,
                &search_cells,
                *search_all_angles,
            )
        })
        .map(|case| case.case_id.clone())
        .collect::<Vec<_>>();
    let hidden_cases = bundle.cases.len().saturating_sub(visible_case_ids.len());
    let hidden_selected = selected
        .iter()
        .filter(|id| !visible_case_ids.contains(id))
        .count();
    let visible_case_id_set = visible_case_ids
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
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
    let probabilities = case_probabilities(&selected, &results, &settings);
    let cube_colors = cube_schemes::active_colors(&settings, &bundle.trainer_id, &bundle.puzzle);

    html! {
        <main id="trainer-main" tabindex="0" style="display:flex; flex-direction:column; align-items:center; width:100%;" onkeydown={key_down} onkeyup={key_up}>
            if *mode == Mode::Select {
                <div id="selectionLayout" class="colFlex scroller" style={format!("--selection-columns:{selection_columns}; margin: 0.5em; max-width: 70em; width: fit-content; display:flex;")}>
                    <div class="colFlex">
                        <div class="rowFlex center" style="justify-content: space-between;">
                            <h1 id="trainerTitle">{&bundle.title}<span>{" | "}</span><a href="/">{"Back"}</a></h1>
                            <div class="colFlex">
                                <div id="settingsButton">
                                    <button onclick={open_settings.clone()} title="Settings" aria-label="Settings" class="material-symbols-outlined abutton largeButton">{"settings"}</button>
                                </div>
                                <div id="modeButtons" class="colFlex">
                                    <button onclick={{
                                        let start_training = start_training.clone();
                                        Callback::from(move |_| start_training.emit(()))
                                    }}
                                        class="material-symbols-outlined abutton largeButton startPracticeButton"
                                        disabled={selected.is_empty()} title="Start recap" aria-label="Start recap">
                                        {"send"}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="borderedContainer colFlex pad center" style="background-color: var(--backgroundDarker);">
                        <div id="presetsSettings" class="colFlex" style="width:fit-content; max-width:100%;">
                            if !presets.contains_key("Default") {
                                <div class="settingsEntry presetRow">
                                    <span class="presetName">{"Default"}</span>
                                    <div class="plusMinus presetActions">
                                        <button class="abutton" onclick={{
                                            let add_to_selection = add_preset_to_selection.clone();
                                            let algsets = selected_algsets.clone();
                                            Callback::from(move |_| add_to_selection.emit(SelectionPreset {
                                                sel_cases: vec!["1".to_owned(), "2".to_owned()],
                                                selected_alg_sets: (*algsets).clone(),
                                            }))
                                        }}>{"+"}</button>
                                        <button class="abutton" disabled=true>{"-"}</button>
                                        <button class="abutton" disabled=true>{"Del"}</button>
                                        <button class="abutton" disabled=true>{"Set"}</button>
                                        <button class="abutton" onclick={{
                                            let use_preset = use_preset.clone();
                                            let algsets = selected_algsets.clone();
                                            Callback::from(move |_| use_preset.emit(SelectionPreset {
                                                sel_cases: vec!["1".to_owned(), "2".to_owned()],
                                                selected_alg_sets: (*algsets).clone(),
                                            }))
                                        }}>{"Use"}</button>
                                    </div>
                                </div>
                            }
                            { for presets.iter().map(|(name, preset)| {
                                let use_preset = use_preset.clone();
                                let add_to_selection = add_preset_to_selection.clone();
                                let remove_from_selection = remove_preset_from_selection.clone();
                                let update_preset = update_preset.clone();
                                let delete_preset = delete_preset.clone();
                                let preset = preset.clone();
                                let preset_name = name.clone();
                                html! {
                                    <div class="settingsEntry presetRow">
                                        <span class="presetName">{&preset_name}</span>
                                        <div class="plusMinus presetActions">
                                            <button class="abutton" aria-label={format!("Add {} to selection", preset_name)} onclick={{
                                                let preset = preset.clone();
                                                Callback::from(move |_| add_to_selection.emit(preset.clone()))
                                            }}>{"+"}</button>
                                            <button class="abutton" aria-label={format!("Remove {} from selection", preset_name)} onclick={{
                                                let preset = preset.clone();
                                                Callback::from(move |_| remove_from_selection.emit(preset.clone()))
                                            }}>{"-"}</button>
                                            <button class="abutton" onclick={{
                                                let name = preset_name.clone();
                                                Callback::from(move |_| delete_preset.emit(name.clone()))
                                            }}>{"Del"}</button>
                                            <button class="abutton" onclick={{
                                                let name = preset_name.clone();
                                                Callback::from(move |_| update_preset.emit(name.clone()))
                                            }}>{"Set"}</button>
                                            <button class="abutton" onclick={Callback::from(move |_| use_preset.emit(preset.clone()))}>{"Use"}</button>
                                        </div>
                                    </div>
                                }
                            }) }
                            <div class="settingsEntry">
                                <input id="newPresetInput" type="text" placeholder="New Preset"
                                    value={(*preset_name).clone()}
                                    oninput={{
                                        let preset_name = preset_name.clone();
                                        Callback::from(move |event: InputEvent| {
                                            preset_name.set(event.target_unchecked_into::<web_sys::HtmlInputElement>().value());
                                        })
                                    }} />
                                <button class="abutton" onclick={add_preset}>{"Add"}</button>
                            </div>
                        </div>
                        <div style="width:100%; display:flex; justify-content:space-between; align-items:end;">
                            <button class="linkButton" onclick={{
                                let open_dialog = open_dialog.clone();
                                Callback::from(move |_| open_dialog.emit(Dialog::Hotkeys))
                            }}>{"Hotkeys"}</button>
                            <a href={format!("/algsheet?algset={}", bundle.trainer_id)}>{"Algs lookup"}</a>
                            <button class="linkButton" onclick={{
                                let open_dialog = open_dialog.clone();
                                Callback::from(move |_| open_dialog.emit(Dialog::Credits))
                            }}>{"Credits"}</button>
                        </div>
                        <span class="version">{"v2.0.0"}</span>
                    </div>
                    <div id="cases_selection" class="colFlex" style="width: 100%;">
                        if bundle.algsets.len() > 1 {
                            <div class="rowFlex algsetSelectors" style="flex-wrap:wrap;">
                                { for bundle.algsets.iter().map(|algset| {
                                    let selected_algsets = selected_algsets.clone();
                                    let toggle_algset = toggle_algset.clone();
                                    let cases_by_id = cases_by_id.clone();
                                    let case_ids = bundle.groups.iter()
                                        .filter(|group| algset.groups.contains(&group.name))
                                        .flat_map(|group| group.cases.clone())
                                        .filter(|id| cases_by_id.contains_key(id))
                                        .collect::<std::collections::BTreeSet<_>>()
                                        .into_iter()
                                        .collect::<Vec<_>>();
                                    let enabled = selected_algsets.get(&algset.name).copied().unwrap_or(algset.selected);
                                    let selected_count = case_ids.iter().filter(|id| selected.contains(id)).count();
                                    let count = if enabled { selected_count.to_string() } else { "-".to_owned() };
                                    let fully_selected = enabled
                                        && !case_ids.is_empty()
                                        && selected_count == case_ids.len();
                                    let label = format!("{} ({}/{})", algset.name, count, case_ids.len());
                                    let name = algset.name.clone();
                                    html! {
                                        <button class={classes!(
                                            "borderedContainer",
                                            "pad",
                                            if fully_selected { "itemSel" } else { "itemUnsel" },
                                            (!enabled).then_some("algsetDisabled")
                                        )}
                                            onclick={Callback::from(move |_| toggle_algset.emit((name.clone(), case_ids.clone())))}>{label}</button>
                                    }
                                }) }
                            </div>
                        }
                        <div class="rowFlex" style="align-items:stretch;">
                            <div id="allSelector"
                                 class={classes!("borderedContainer", "pad", if !visible_case_ids.is_empty() && visible_case_ids.iter().all(|id| selected.contains(id)) { "itemSel" } else { "itemUnsel" })}
                                 style="flex-grow:1; text-align:center;" onclick={{
                                     let select_all = select_all.clone();
                                     let visible_case_ids = visible_case_ids.clone();
                                     Callback::from(move |_| select_all.emit(visible_case_ids.clone()))
                                 }}>
                                <b>{format!("{} ({}/{})", if search_active { "Search Results" } else { "All Cases" }, visible_case_ids.iter().filter(|id| selected.contains(id)).count(), visible_case_ids.len())}</b>
                                if search_active || enabled_case_ids.len() < bundle.cases.len() {
                                    <span class="hiddenCaseCount">{format!("Hidden: {hidden_cases} ({hidden_selected} selected)")}</span>
                                }
                            </div>
                            <button class={classes!("borderedContainer", "pad", "searchToggle",
                                    (*search_open).then_some("searchToggleOpen"),
                                    search_active.then_some("searchToggleActive"))}
                                 style="display:flex; flex-direction:column; justify-content:center; padding-left:1em; padding-right:1em;"
                                 title="Search" aria-label="Search cases"
                                 onclick={{
                                     let search_open = search_open.clone();
                                     Callback::from(move |_| search_open.set(!*search_open))
                                 }}>
                                <span class="material-symbols-outlined">{"search"}</span>
                            </button>
                        </div>
                        if *search_open {
                            <div class="searchPanel borderedContainer pad colFlex">
                                if let Some(template) = search_template_svg(bundle, &search_cells, &cube_colors) {
                                    <div class="searchDiagram borderedContainer itemUnsel pad" onclick={{
                                        let search_cells = search_cells.clone();
                                        let search_color = search_color.clone();
                                        Callback::from(move |event: MouseEvent| {
                                            let Some(element) = event.target_dyn_into::<Element>() else {
                                                return;
                                            };
                                            let Some(index) = element.get_attribute("data-search-index").and_then(|value| value.parse::<usize>().ok()) else {
                                                return;
                                            };
                                            let mut cells = (*search_cells).clone();
                                            let color = (*search_color).clone();
                                            if cells.get(&index) == Some(&color) {
                                                cells.remove(&index);
                                            } else {
                                                cells.insert(index, color);
                                            }
                                            search_cells.set(cells);
                                        })
                                    }}>
                                        {Html::from_html_unchecked(template.into())}
                                        <button class="material-symbols-outlined searchDiagramClear compactIconButton"
                                            title="Clear image search" aria-label="Clear image search"
                                            onclick={{
                                                let search_cells = search_cells.clone();
                                                Callback::from(move |_| search_cells.set(BTreeMap::new()))
                                            }}>{"close"}</button>
                                    </div>
                                }
                                if !bundle.u_move_cycles.is_empty() {
                                    <label class="rowFlex searchAngleToggle">
                                        <span>{"Search all angles"}</span>
                                        <input type="checkbox" checked={*search_all_angles}
                                            onchange={{
                                                let search_all_angles = search_all_angles.clone();
                                                Callback::from(move |event: Event| {
                                                    search_all_angles.set(event.target_unchecked_into::<web_sys::HtmlInputElement>().checked());
                                                })
                                            }} />
                                    </label>
                                }
                                <div class="rowFlex searchPalette" aria-label="Search colors">
                                    {for search_color_names().into_iter().map(|name| {
                                        let selected_color = search_color.clone();
                                        let active = *selected_color == name;
                                        let color = cube_color(&cube_colors, name);
                                        html! {
                                            <button class={classes!("searchSwatch", active.then_some("active"))}
                                                title={name} aria-label={format!("Use {name} search color")}
                                                aria-pressed={active.to_string()}
                                                style={format!("background:{color}")}
                                                onclick={Callback::from(move |_| selected_color.set(name.to_owned()))}>
                                            </button>
                                        }
                                    })}
                                </div>
                                <div class="rowFlex searchTextRow">
                                    <span class="material-symbols-outlined">{"search"}</span>
                                    <input id="searchTextInput" type="search"
                                        placeholder="Search algset, group, name, setup or algorithm"
                                        value={(*search_query).clone()}
                                        oninput={{
                                            let search_query = search_query.clone();
                                            Callback::from(move |event: InputEvent| search_query.set(event.target_unchecked_into::<web_sys::HtmlInputElement>().value()))
                                        }} />
                                    <button class="material-symbols-outlined abutton compactButton"
                                        title="Clear search" aria-label="Clear search"
                                        onclick={{
                                            let search_query = search_query.clone();
                                            let search_cells = search_cells.clone();
                                            Callback::from(move |_| {
                                                search_query.set(String::new());
                                                search_cells.set(BTreeMap::new());
                                            })
                                        }}>{"close"}</button>
                                </div>
                            </div>
                        }
                        { {
                            let mut displayed_algsets = std::collections::BTreeSet::new();
                            bundle.groups.iter().map(|group| {
                            let selected = selected.clone();
                            let cases_by_id = cases_by_id.clone();
                            let toggle_case = toggle_case.clone();
                            let toggle_group = toggle_group.clone();
                            let group_cases = group.cases.clone();
                            let needle = search_needle.clone();
                            let visible_cases = group.cases.iter()
                                .filter_map(|id| cases_by_id.get(id))
                                .filter(|case| case_matches_search(case, bundle, &needle, &search_cells, *search_all_angles))
                                .map(|case| case.case_id.clone())
                                .collect::<Vec<_>>();
                            if search_active && visible_cases.is_empty() {
                                return Html::default();
                            }
                            let group_enabled = bundle.algsets.iter().any(|algset| {
                                algset.groups.contains(&group.name)
                                    && selected_algsets.get(&algset.name).copied().unwrap_or(algset.selected)
                            });
                            if !group_enabled {
                                return Html::default();
                            }
                            let visible_toggle_cases = visible_cases.clone();
                            let group_selected = group.cases.iter().all(|id| selected.contains(id));
                            let algset_headers = bundle.algsets.iter()
                                .filter(|algset| algset.groups.contains(&group.name)
                                    && selected_algsets.get(&algset.name).copied().unwrap_or(algset.selected))
                                .filter_map(|algset| displayed_algsets.insert(algset.name.clone()).then(|| algset.name.clone()))
                                .collect::<Vec<_>>();
                            html! {
                                <>
                                    {for algset_headers.iter().map(|name| html! {
                                        <div class="borderedContainer pad algsetNameHeader"><b>{name}</b></div>
                                    })}
                                    <section class={classes!("caseGroup", (visible_cases.len() == widest_group_cases).then_some("widestCaseGroup"))}>
                                        <div class={classes!("borderedContainer", "pad", "groupNameDiv", if group_selected { "itemSel" } else { "itemUnsel" })}
                                             style="text-align:center;" onclick={Callback::from(move |_| {
                                                 if !search_active { toggle_group.emit(group_cases.clone()) }
                                                 else { toggle_group.emit(visible_toggle_cases.clone()) }
                                             })}>
                                            <b>{&group.name}</b>
                                        </div>
                                        <div class="rowFlex" style="flex-wrap: wrap;">
                                        { for visible_cases.iter().filter_map(|id| cases_by_id.get(id)).map(|case| {
                                            let is_selected = selected.contains(&case.case_id);
                                            let case_id = case.case_id.clone();
                                            let toggle_case = toggle_case.clone();
                                            let open_hint_for_case = open_hint_for_case.clone();
                                            let hint_case = case.clone();
                                            let hint_case_mobile = case.clone();
                                            let hint_timeout = Rc::new(RefCell::new(None::<Timeout>));
                                            let image = case.image_svg.as_ref()
                                                                .map(|svg| case_image_data(svg, &cube_colors))
                                                .unwrap_or_default();
                                            let include_name = bundle.title.contains("PLL")
                                                || bundle.title.contains("BLD")
                                                || bundle.title.contains("Square-1");
                                            html! {
                                                <div id={format!("itemTd{}", case.case_id)}
                                                     class={classes!("borderedContainer", "caseItem", if is_selected { "itemSel" } else { "itemUnsel" })}
                                                     onclick={Callback::from(move |_| toggle_case.emit(case_id.clone()))}
                                                     ondblclick={Callback::from({
                                                         let open_hint_for_case = open_hint_for_case.clone();
                                                         let hint_case = hint_case.clone();
                                                         move |event: web_sys::MouseEvent| {
                                                             event.prevent_default();
                                                             open_hint_for_case.emit(hint_case.clone());
                                                         }
                                                     })}
                                                     oncontextmenu={Callback::from({
                                                         let open_hint_for_case = open_hint_for_case.clone();
                                                         let hint_case = hint_case.clone();
                                                         move |event: web_sys::MouseEvent| {
                                                             event.prevent_default();
                                                             open_hint_for_case.emit(hint_case.clone());
                                                         }
                                                     })}
                                                     onpointerdown={Callback::from({
                                                         let hint_timeout = hint_timeout.clone();
                                                         let open_hint_for_case = open_hint_for_case.clone();
                                                         let hint_case_mobile = hint_case_mobile.clone();
                                                         move |event: web_sys::PointerEvent| {
                                                             if event.pointer_type() != "touch" {
                                                                 return;
                                                             }
                                                             let open_hint_for_case = open_hint_for_case.clone();
                                                             let hint_case_mobile = hint_case_mobile.clone();
                                                             *hint_timeout.borrow_mut() = Some(Timeout::new(600, move || {
                                                                 open_hint_for_case.emit(hint_case_mobile);
                                                             }));
                                                         }
                                                     })}
                                                     onpointerup={Callback::from({
                                                         let hint_timeout = hint_timeout.clone();
                                                         move |event: web_sys::PointerEvent| {
                                                             if event.pointer_type() == "touch" {
                                                                 if let Some(timeout) = hint_timeout.borrow_mut().take() {
                                                                     timeout.cancel();
                                                                 }
                                                             }
                                                         }
                                                     })}>
                                                    if include_name {
                                                        <span class="caseSpan">{&case.name}</span>
                                                    }
                                                    <img class="caseImage" src={image} />
                                                    if settings.get("showProbabilities").and_then(serde_json::Value::as_bool).unwrap_or(false) {
                                                        <span class="caseProbability">
                                                            {probabilities.get(&case.case_id)
                                                                .map(|value| format!("{:.1}%", value * 100.0))
                                                                .unwrap_or_default()}
                                                        </span>
                                                    }
                                                </div>
                                            }
                                        }) }
                                        </div>
                                    </section>
                                </>
                            }
                            }).collect::<Vec<_>>()
                        } }
                    </div>
                </div>
            } else {
                <div id="practiceLayout" class="colFlex" style="display:flex;">
                    <div id="casesBox" class="borderedContainer pad rowFlex center">
                        <button class="material-symbols-outlined abutton mbutton" onclick={Callback::from(move |_| {
                            set_body_background("var(--background)");
                            mode.set(Mode::Select);
                            if let Some(window) = web_sys::window() {
                                if let Ok(history) = window.history() {
                                    let _ = history.push_state_with_url(
                                        &JsValue::from_str("select"),
                                        "",
                                        Some("?select"),
                                    );
                                }
                            }
                        })}>{"arrow_back"}</button>
                        <span id="selInfo" style="font-size:2em;">
                            <b>
                                if !*recap_active {
                                    <a onclick={{
                                        let start_training = start_training.clone();
                                        Callback::from(move |_| start_training.emit(()))
                                    }}>{"Train"}</a>
                                    {format!(" {} Cases", selected.len())}
                                } else {
                                    <a onclick={{
                                        let switch_to_train = switch_to_train.clone();
                                        Callback::from(move |_| switch_to_train.emit(()))
                                    }}>{"Recap"}</a>
                                    {format!(" {} Cases", recap_remaining.len() + 1)}
                                }
                            </b>
                        </span>
                        <button class="material-symbols-outlined abutton" title="Settings" aria-label="Settings" onclick={open_settings}>{"settings"}</button>
                    </div>
                    <div class="borderedContainer pad center rowFlex practiceScramble" style="justify-content:space-between;">
                        <div id="scramble" onclick={open_hint.clone()}
                             style={format!("font-size:{}em;", settings.get("scrambleSize").and_then(serde_json::Value::as_f64).unwrap_or(2.0))}>
                            {if current_scramble.is_empty() { "Select at least one case".to_owned() } else { (*current_scramble).clone() }}
                            <span class="inlineButton" style="font-size:1em !important;">{"?"}</span>
                        </div>
                    </div>
                    <div id="timer_and_stats" class="rowFlex grow">
                        <div id="timerDiv" class="borderedContainer" onclick={{
                            let finish_timer = finish_timer.clone();
                            Callback::from(move |_| finish_timer.emit(()))
                        }} onpointerdown={pointer_down} onpointerup={pointer_up}
                            onpointermove={Callback::from(|event: web_sys::PointerEvent| event.prevent_default())}
                            onpointercancel={pointer_cancel}>
                            <span id="timer" style={format!("text-align:center; color:{}; font-size:{}em;",
                                if *timer_ready { "#008500" } else if *timer_running { "inherit" } else if *timer_stopped { "#850000" } else { "inherit" },
                                settings.get("timerSize").and_then(serde_json::Value::as_f64).unwrap_or(5.0))}>
                                {if *timer_running || *elapsed_ms > 0 { format_time(*elapsed_ms) } else { "ready".to_owned() }}
                            </span>
                        </div>
                        <div id="stats" class="borderedContainer colFlex pad">
                            <div class="timeEntry rowFlex" style="justify-content:space-between;">
                                <div class="colFlex">
                                    <span>{format!("Times {}", results.len())}</span>
                                    <span>{format!("Cases {}", results.iter().map(|result| &result.case).collect::<std::collections::BTreeSet<_>>().len())}</span>
                                </div>
                                <button class="material-symbols-outlined inlineButton" onclick={{
                                    let results = results.clone();
                                    let times_key = times_storage_key.clone();
                                    Callback::from(move |_| {
                                        let deleted_ids = (*results).iter().map(|result| result.id.clone()).collect();
                                        results.set(Vec::new());
                                        let key = times_key.clone();
                                        spawn_local(async move {
                                            put_stored_results(key, Vec::new(), deleted_ids).await;
                                        });
                                    })
                                }}>{"delete"}</button>
                            </div>
                            <div id="times" class="colFlex">
                                { for cases_by_id.values().filter_map(|case| {
                                    let case_results = results.iter()
                                        .filter(|result| result.case == case.case_id)
                                        .collect::<Vec<_>>();
                                    if case_results.is_empty() {
                                        return None;
                                    }
                                    let mean = case_results.iter().map(|result| result.ms as f64).sum::<f64>()
                                        / case_results.len() as f64;
                                    let open_hint_for_case = open_hint_for_case.clone();
                                    let hint_case = case.clone();
                                    let open_stats = open_stats.clone();
                                    let case_id = case.case_id.clone();
                                    Some(html! {
                                        <div class="timeEntry">
                                            <button class="caseNameStats" onclick={Callback::from(move |_| {
                                                open_hint_for_case.emit(hint_case.clone());
                                            })}>{&case.name}</button>
                                            <button class="caseNameStats" title={format!("View {} statistics", &case.name)}
                                                onclick={Callback::from(move |_| open_stats.emit(case_id.clone()))}>
                                                {format!("(#{}; avg {})", case_results.len(), format_time(mean.round() as i64))}
                                            </button>
                                        </div>
                                    })
                                }) }
                            </div>
                        </div>
                    </div>
                    <div class="borderedContainer pad rowFlex center practiceFooter" style="justify-content:space-between;flex-direction:row-reverse;">
                        <div id="last_scramble" class="rowFlex" style="width:100%;">
                            <span class="lastScrambleText">
                                {"Last scramble: "}
                                if last_scramble.is_empty() { {"None yet"} } else { {&*last_scramble} }
                            </span>
                            if let Some(case) = last_case.as_ref() {
                                <button class="caseNameStats" onclick={{
                                    let open_last_hint = open_last_hint.clone();
                                    Callback::from(move |_| open_last_hint.emit(()))
                                }}>{format!("({})", case.name)}</button>
                            }
                            <span id="last-scramble-buttons">
                                <button class="material-symbols-outlined inlineButton" title="Bookmark last case" aria-label="Bookmark last case"
                                    onclick={bookmark_last_case}>
                                    {if last_case.as_ref().and_then(|case| presets.get("Bookmarks").map(|preset| preset.sel_cases.contains(&case.case_id))).unwrap_or(false) {"bookmark_remove"} else {"bookmark_add"}}
                                </button>
                                <button class="material-symbols-outlined inlineButton" title="Remove last case from selection" aria-label="Remove last case from selection"
                                    onclick={remove_last_case}>{"close"}</button>
                                <button class="material-symbols-outlined inlineButton" title="Delete last time" aria-label="Delete last time"
                                    onclick={remove_last_time}>{"undo"}</button>
                            </span>
                        </div>
                    </div>
                </div>
            }
            if *hint_open {
                <dialog id="hintWindow" class="colFlex" open=true onclick={close_hint_background}>
                    <div class="dialogDiv colFlex borderedContainer">
                        <button class="material-symbols-outlined abutton closeButton" onclick={Callback::from({
                            let hint_open = hint_open.clone();
                            move |_| hint_open.set(false)
                        })}>{"close"}</button>
                        if let Some(case) = hint_case.as_ref() {
                            <div class="borderedContainer rowFlex pad center hintCaseHeader" style="justify-content:space-between;">
                                <button class="material-symbols-outlined abutton prevNext" onclick={previous_hint.clone()} title="Previous case">{"arrow_back"}</button>
                                <b>{&case.name}</b>
                                <button class="material-symbols-outlined abutton prevNext" onclick={next_hint.clone()} title="Next case">{"arrow_forward"}</button>
                            </div>
                            <div class="borderedContainer colFlex center" style="background-color:var(--secondary);">
                                if let Some(svg) = case.image_svg.as_ref() {
                                    <img style="width:16em;" src={case_image_data(svg, &cube_colors)} />
                                }
                            </div>
                            <div class="borderedContainer grow rowFlex center pad">{case.setup.as_deref().unwrap_or("")}</div>
                            <div class="borderedContainer grow colFlex algList">
                                {for display_algs(case, &custom_algs).into_iter().map(|item| {
                                    let editing = alg_editor.as_ref().is_some_and(|(id, _)| id.as_deref() == Some(item.alg_id.as_str()));
                                    let dragged_alg_start = dragged_alg.clone();
                                    let dragged_alg_drop = dragged_alg.clone();
                                    let reorder_alg = reorder_alg.clone();
                                    let hide_alg = hide_alg.clone();
                                    let delete_alg = delete_alg.clone();
                                    let alg_editor = alg_editor.clone();
                                    let drag_id = item.alg_id.clone();
                                    let drop_id = item.alg_id.clone();
                                    html! {
                                        <div class="algItem borderedContainer"
                                            ondragover={Callback::from(|event: DragEvent| event.prevent_default())}
                                            ondrop={{
                                                let reorder_alg = reorder_alg.clone();
                                                Callback::from(move |event: DragEvent| {
                                                    event.prevent_default();
                                                    if !dragged_alg_drop.is_empty() && dragged_alg_drop.as_str() != drop_id.as_str() {
                                                        reorder_alg.emit((dragged_alg_drop.to_string(), drop_id.clone()));
                                                    }
                                                })
                                            }}>
                                            <button class="material-symbols-outlined algHandle" draggable="true"
                                                title="Drag to reorder" aria-label="Drag to reorder"
                                                ondragstart={Callback::from(move |_| dragged_alg_start.set(drag_id.clone()))}>
                                                {"drag_indicator"}
                                            </button>
                                            if editing {
                                                <textarea class="algEditorInput" rows="2"
                                                    value={alg_editor.as_ref().map(|(_, value)| value.clone()).unwrap_or_default()}
                                                    oninput={Callback::from({
                                                        let alg_editor = alg_editor.clone();
                                                        move |event: InputEvent| {
                                                            let value = event.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                                                            if let Some((id, _)) = alg_editor.as_ref() {
                                                                alg_editor.set(Some((id.clone(), value)));
                                                            }
                                                        }
                                                    })} />
                                                <button class="material-symbols-outlined algAction" title="Save algorithm"
                                                    onclick={save_editor.clone()}>{"check"}</button>
                                                <button class="material-symbols-outlined algAction" title="Cancel edit"
                                                    onclick={Callback::from({
                                                        let alg_editor = alg_editor.clone();
                                                        move |_| alg_editor.set(None)
                                                    })}>{"close"}</button>
                                            } else {
                                                <span class="algText">{&item.alg}</span>
                                                if item.user {
                                                    <button class="material-symbols-outlined algAction" title="Edit algorithm"
                                                        onclick={Callback::from({
                                                            let alg_editor = alg_editor.clone();
                                                            let id = item.alg_id.clone();
                                                            let value = item.alg.clone();
                                                            move |_| alg_editor.set(Some((Some(id.clone()), value.clone())))
                                                        })}>{"edit"}</button>
                                                    <button class="material-symbols-outlined algAction" title="Delete algorithm"
                                                        onclick={Callback::from({
                                                            let id = item.alg_id.clone();
                                                            move |_| delete_alg.emit(id.clone())
                                                        })}>{"close"}</button>
                                                } else {
                                                    <button class="material-symbols-outlined algAction" title="Hide algorithm"
                                                        onclick={Callback::from(move |_| hide_alg.emit(item.clone()))}>{"visibility_off"}</button>
                                                }
                                            }
                                        </div>
                                    }
                                })}
                                if let Some((None, value)) = alg_editor.as_ref() {
                                    <div class="algItem algNewItem borderedContainer">
                                        <textarea class="algEditorInput" rows="2" placeholder="Enter a new algorithm"
                                            value={value.clone()}
                                            oninput={Callback::from({
                                                let alg_editor = alg_editor.clone();
                                                move |event: InputEvent| alg_editor.set(Some((
                                                    None,
                                                    event.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value(),
                                                )))
                                            })} />
                                        <button class="material-symbols-outlined algAction" title="Add algorithm"
                                            onclick={save_editor.clone()}>{"check"}</button>
                                        <button class="material-symbols-outlined algAction" title="Cancel"
                                            onclick={Callback::from({
                                                let alg_editor = alg_editor.clone();
                                                move |_| alg_editor.set(None)
                                            })}>{"close"}</button>
                                    </div>
                                } else {
                                    <button class="material-symbols-outlined abutton algAddButton" title="Add algorithm"
                                        onclick={Callback::from({
                                            let alg_editor = alg_editor.clone();
                                            move |_| alg_editor.set(Some((None, String::new())))
                                        })}>{"add"}</button>
                                }
                            </div>
                        }
                    </div>
                </dialog>
            }
            if *dialog == Dialog::Stats {
                <dialog id="caseTimeDetails" class="colFlex" open=true onclick={close_dialog_background.clone()}>
                    <div class="dialogDiv colFlex borderedContainer">
                        <button class="material-symbols-outlined abutton closeButton" onclick={close_dialog.clone()} aria-label="Close">{"close"}</button>
                        {{
                            let case_id = (*stats_case_id).clone();
                            let case_stats = if case_id.is_empty() {
                                results.iter().collect::<Vec<_>>()
                            } else {
                                results.iter().filter(|result| result.case == case_id).collect::<Vec<_>>()
                            };
                            let title = if case_id.is_empty() {
                                "All cases".to_owned()
                            } else {
                                cases_by_id.get(&case_id).map(|case| case.name.clone()).unwrap_or_else(|| case_id.clone())
                            };
                            let times = case_stats.iter().map(|result| result.ms as f64 / 1000.0).collect::<Vec<_>>();
                            let mean = if times.is_empty() { 0.0 } else { times.iter().sum::<f64>() / times.len() as f64 };
                            let min = times.iter().copied().fold(f64::INFINITY, f64::min);
                            let max = times.iter().copied().fold(0.0_f64, f64::max);
                            let range = if max > min { max - min } else { 1.0 };
                            let bin_count = 10usize.min(times.len().max(1));
                            let mut counts = vec![0usize; bin_count];
                            for time in &times {
                                let index = (((*time - min) / range * bin_count as f64).floor() as usize).min(bin_count - 1);
                                counts[index] += 1;
                            }
                            let max_count = counts.iter().copied().max().unwrap_or(1).max(1);
                            let point_width = 280.0;
                            let plot_max = if max > 0.0 { max } else { 1.0 };
                            let points = times.iter().enumerate().map(|(index, time)| {
                                let x = if times.len() > 1 { index as f64 * point_width / (times.len() - 1) as f64 } else { point_width / 2.0 };
                                format!("{x:.1},{:.1}", 120.0 - (*time / plot_max * 100.0))
                            }).collect::<Vec<_>>().join(" ");
                            let trend_points = exponential_moving_average(&times, 0.2)
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
                                .join(" ");
                            html! {
                                <div class="colFlex borderedContainer pad" style="width:100%;box-sizing:border-box;">
                                    <h2 id="caseTimeDetailsCaseHeader">{title}</h2>
                                    <div class="rowFlex" style="justify-content:space-around;flex-wrap:wrap;">
                                        <b>{format!("Times: {}", times.len())}</b>
                                        <b>{format!("Mean: {:.2}s", mean)}</b>
                                        <b>{format!("Range: {:.2}-{:.2}s", if times.is_empty() { 0.0 } else { min }, if times.is_empty() { 0.0 } else { max })}</b>
                                    </div>
                                    <div id="timeHistogram" class="rowFlex" style="height:8em;align-items:end;padding:0.5em;">
                                        { for counts.iter().map(|count| html! {
                                            <div style={format!("flex:1;height:{}%;background:var(--primaryBG);min-width:0.5em;", *count as f64 * 100.0 / max_count as f64)} title={count.to_string()}></div>
                                        }) }
                                    </div>
                                    <svg id="timePlot" viewBox="0 0 280 130" style="width:100%;height:10em;background:var(--background);">
                                        <polyline points={trend_points} fill="none" stroke="var(--secondary)" stroke-width="2" />
                                        <polyline points={points} fill="none" stroke="var(--primary)" stroke-width="3" />
                                    </svg>
                                    <div id="caseTimeDetailsTimes" style="max-height:12em;overflow-y:auto;">
                                        { for case_stats.iter().rev().map(|result| html! {
                                            <div>{&result.time}{format!(" - {}", &result.datetime)}</div>
                                        }) }
                                    </div>
                                </div>
                            }
                        }}
                    </div>
                </dialog>
            }
            if *dialog == Dialog::Settings {
                <dialog id="settings" class="colFlex borderedContainer" open=true onclick={close_dialog_background.clone()}>
                    <div class="dialogDiv colFlex borderedContainer">
                        <button class="material-symbols-outlined abutton closeButton" onclick={close_dialog.clone()} aria-label="Close">{"close"}</button>
                        <details class="settingsDetails" name="trainer-settings" open=true>
                            <summary class="settingsSummary">{"Behavior"}</summary>
                            <div class="colFlex">
                                {for [
                                    ("enableHoldDelay", "Hold down delay"),
                                    ("weightedChoice", "Weighted choice (# of Solves)"),
                                    ("weightedChoiceTime", "Weighted choice (Mean time)"),
                                    ("weightedChoiceStd", "Weighted choice (Standard deviation)"),
                                    ("showProbabilities", "Show case probabilities"),
                                ].into_iter().map(|(key, label)| {
                                    let change_setting = change_setting.clone();
                                    let checked = settings.get(key).and_then(serde_json::Value::as_bool)
                                        .unwrap_or(key == "enableHoldDelay" || key == "weightedChoice");
                                    html! {
                                        <div class="settingsEntry">
                                            <div>{label}</div>
                                            <label class="switch">
                                                <input type="checkbox" checked={checked} onchange={{
                                                    let key = key.to_owned();
                                                    Callback::from(move |event: Event| {
                                                        change_setting.emit((key.clone(), event.target_unchecked_into::<web_sys::HtmlInputElement>().checked()));
                                                    })
                                                }} />
                                                <span class="slider round"></span>
                                            </label>
                                        </div>
                                    }
                                })}
                            </div>
                        </details>
                        <AccountSettings />
                        <details class="settingsDetails" name="trainer-settings">
                            <summary class="settingsSummary">{"Scramble options"}</summary>
                            <div class="colFlex">
                                <div class="trainerScrambleHelp">{"Enter one option per line. Leave blank to use the trainer defaults."}</div>
                                {for [
                                    ("preRotations", "Pre-rotations"),
                                    ("postRotations", "Post-rotations"),
                                    ("preMoves", "Pre-moves"),
                                    ("postMoves", "Post-moves"),
                                ].into_iter().map(|(key, label)| {
                                    let value = settings
                                        .get("trainerScrambleOverrides")
                                        .and_then(|value| value.get(&selection_storage_key))
                                        .and_then(|value| value.get(key))
                                        .and_then(serde_json::Value::as_array)
                                        .map(|items| items.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>().join("\n"))
                                        .unwrap_or_default();
                                    let change_scramble_override = change_scramble_override.clone();
                                    html! {
                                        <div class="settingsEntry trainerScrambleEntry">
                                            <label>{label}</label>
                                            <textarea rows="2" placeholder="Trainer default" value={value}
                                                onchange={{
                                                    Callback::from(move |event: Event| {
                                                        change_scramble_override.emit((key, event.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value()));
                                                    })
                                                }} />
                                        </div>
                                    }
                                })}
                            </div>
                        </details>
                        <details class="settingsDetails" name="trainer-settings">
                            <summary class="settingsSummary">{"Styling"}</summary>
                            <div class="colFlex">
                                {for [
                                    ("baseSize", "Base size"),
                                    ("timerSize", "Timer size"),
                                    ("scrambleSize", "Scramble size"),
                                ].into_iter().map(|(key, label)| {
                                    let adjust_setting = adjust_setting.clone();
                                    html! {
                                        <div class="settingsEntry">
                                            <div>{label}</div>
                                            <div class="plusMinus">
                                                <button onclick={{let adjust_setting = adjust_setting.clone(); Callback::from(move |_| adjust_setting.emit((key, 0.1)))}} class="abutton">{"+"}</button>
                                                <button onclick={Callback::from(move |_| adjust_setting.emit((key, -0.1)))} class="abutton">{"-"}</button>
                                            </div>
                                        </div>
                                    }
                                })}
                                <div class="settingsEntry">
                                    <div>{"Theme"}</div>
                                    <div class="plusMinus">
                                        <button class="material-symbols-outlined abutton" title="Light theme" onclick={{
                                            let reset_theme = reset_theme.clone();
                                            Callback::from(move |_| reset_theme.emit("light"))
                                        }}>{"light_mode"}</button>
                                        <button class="material-symbols-outlined abutton" title="Dark theme" onclick={{
                                            let reset_theme = reset_theme.clone();
                                            Callback::from(move |_| reset_theme.emit("dark"))
                                        }}>{"dark_mode"}</button>
                                    </div>
                                </div>
                                {for [
                                    ("--background", "Background"),
                                    ("--text", "Text"),
                                    ("--primary", "Primary"),
                                    ("--secondary", "Secondary"),
                                    ("--accent", "Accent"),
                                ].into_iter().map(|(key, label)| {
                                    let value = setting_color(&settings, key);
                                    let style = color_input_style(&value, &setting_color(&settings, "--background"));
                                    let change_color = change_color.clone();
                                    html! {
                                        <div class="settingsEntry colorSetting">
                                            <div>{label}</div>
                                            <input type="text" value={value.clone()} style={style}
                                                aria-label={format!("{} color", label)}
                                                onchange={{
                                                    Callback::from(move |event: Event| {
                                                        let input = event.target_unchecked_into::<web_sys::HtmlInputElement>();
                                                        let new_value = input.value();
                                                        if colorInputStyle(&new_value, "#fff").is_err() {
                                                            if let Some(window) = web_sys::window() {
                                                                let _ = window.alert_with_message("Please enter a valid CSS color.");
                                                            }
                                                            input.set_value(&value);
                                                            return;
                                                        }
                                                        change_color.emit((key.to_owned(), new_value));
                                                    })
                                                }} />
                                        </div>
                                    }
                                })}
                            </div>
                        </details>
                        <details class="settingsDetails" name="trainer-settings">
                            <summary class="settingsSummary">{"Cube colors"}</summary>
                            if *settings_ready {
                                <cube_settings::CubeSettings settings={(*settings).clone()} on_change={change_cube_settings} />
                            } else {
                                <div>{"Loading settings..."}</div>
                            }
                        </details>
                        <details class="settingsDetails" name="trainer-settings">
                            <summary class="settingsSummary">{"Blind trainer behavior"}</summary>
                            <div class="colFlex">
                                {for [
                                    ("letterSchemeCorners", "Corner letter scheme"),
                                    ("letterSchemeEdges", "Edge letter scheme"),
                                ].into_iter().map(|(key, label)| {
                                    let value = settings.get(key).and_then(serde_json::Value::as_str)
                                        .unwrap_or("abcdefghijklmnopqrstuvwx").to_owned();
                                    let change_setting_value = change_setting_value.clone();
                                    html! {
                                        <div class="settingsEntry">
                                            <div>{label}</div>
                                            <input type="text" value={value}
                                                onchange={{
                                                    Callback::from(move |event: Event| {
                                                        change_setting_value.emit((key.to_owned(), event.target_unchecked_into::<web_sys::HtmlInputElement>().value().into()));
                                                    })
                                                }} />
                                        </div>
                                    }
                                })}
                                {for [
                                    ("useCornersLetterPairs", "Corner letter-pair words"),
                                    ("useEdgesLetterPairs", "Edge letter-pair words"),
                                ].into_iter().map(|(key, label)| {
                                    let change_setting = change_setting.clone();
                                    let checked = settings.get(key).and_then(serde_json::Value::as_bool).unwrap_or(false);
                                    html! {
                                        <div class="settingsEntry">
                                            <div>{label}</div>
                                            <label class="switch">
                                                <input type="checkbox" checked={checked} onchange={{
                                                    Callback::from(move |event: Event| {
                                                        change_setting.emit((key.to_owned(), event.target_unchecked_into::<web_sys::HtmlInputElement>().checked()));
                                                    })
                                                }} />
                                                <span class="slider round"></span>
                                            </label>
                                        </div>
                                    }
                                })}
                                <div class="settingsEntry">
                                    <div>{"Letter-pair words"}</div>
                                    <label for="letterPairUpload" class="material-symbols-outlined abutton compactButton" title="Import letter-pair JSON">{"upload"}</label>
                                    <input id="letterPairUpload" type="file" accept="application/json,.json" style="display:none;" onchange={import_letter_pairs} />
                                </div>
                            </div>
                        </details>
                        <details class="settingsDetails" name="trainer-settings">
                            <summary class="settingsSummary">{"Import / Export"}</summary>
                            <div class="colFlex">
                                <div class="settingsEntry">
                                    <div>{"Backup settings and trainer data"}</div>
                                    <div class="plusMinus">
                                        <label for="trainerBackupUpload" class="material-symbols-outlined abutton compactButton" title="Import backup">{"upload"}</label>
                                        <input id="trainerBackupUpload" type="file" accept="application/json,.json" style="display:none;" onchange={import_backup} />
                                        <button class="material-symbols-outlined abutton compactButton" title="Export backup" onclick={export_backup}>{"download"}</button>
                                    </div>
                                </div>
                            </div>
                        </details>
                    </div>
                </dialog>
            }
            if *dialog == Dialog::Hotkeys {
                <dialog id="hotkeysDialog" open=true onclick={close_dialog_background.clone()}>
                    <div class="dialogDiv colFlex borderedContainer">
                        <button class="material-symbols-outlined abutton closeButton" onclick={close_dialog.clone()} aria-label="Close">{"close"}</button>
                        <h2>{"Hotkeys"}</h2>
                        {for [
                            ("Delete", "Delete last time"),
                            ("Shift + Delete", "Clear session"),
                            ("R", "Remove last case from selection"),
                            ("X", "Return to case selection"),
                            ("B", "Bookmark case"),
                            ("H", "Show hint for current case"),
                            ("Shift + H", "Show hint for last case"),
                            ("Space", "Start / stop timer"),
                            ("Escape", "Close dialog"),
                        ].into_iter().map(|(key, action)| html! {
                            <div class="borderedContainer pad rowFlex hotkeyRow">
                                <span class="hotKeyText">{action}</span>
                                <kbd class="hotkey">{key}</kbd>
                            </div>
                        })}
                    </div>
                </dialog>
            }
            if *dialog == Dialog::Credits {
                <dialog id="aboutDialog" open=true onclick={close_dialog_background}>
                    <div class="dialogDiv colFlex borderedContainer">
                        <button class="material-symbols-outlined abutton closeButton" onclick={close_dialog} aria-label="Close">{"close"}</button>
                        <h2>{"Credits and algorithm sources"}</h2>
                        <div class="creditsGrid">
                            <div class="colFlex">
                                <b>{"Algorithm sheets"}</b>
                                <a href="https://docs.google.com/spreadsheets/d/1j-SGlgZk20D3d2TyeiXhMWzeyTBuuix3Owgj9Q8rNbo/edit#gid=1373252438" target="_blank">{"Skewb Resources"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/175ZOo4a9MwglJCnoJ3bWkPOhLhG__kPX4uVSkJMHwio/edit#gid=0" target="_blank">{"Skewb Algs"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/12FH8MswpdBJxLJ_Eu_gopoxTCySPk9H_o7_yjN8_ECw/edit#gid=566431744" target="_blank">{"The Pyraminx Sheet"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/1lVODoMORbDdVXwTG4RUZPWCSrx8uPx2bvwvXg5tuQYE/edit#gid=165427832" target="_blank">{"James' Megaminx Algs"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/17Ym3zUyYXX9NzFcPw5iCs-3fXAjfqCezK-Dsofv8A2Y/edit#gid=288801998" target="_blank">{"Ore's Mega Dot OLLs"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/1OFXakCV85Mp2zsQBXMxiMX9a506JeAcLnUXZr8FgXAY/edit#gid=1684065307" target="_blank">{"Best 2x2 Algs"}</a>
                                <a href="https://crazymonkeyonspeed.github.io/" target="_blank">{"Frederik's FH Algs"}</a>
                                <a href="https://www.youtube.com/@BrinaSun" target="_blank">{"Brian Sun 3x3 Algs"}</a>
                                <a href="https://speedcubedb.com/" target="_blank">{"SpeedCubeDB"}</a>
                                <a href="https://docs.google.com/document/d/1nkgNFdAu9v6p871urGrdkKD-QGmftYhjqZjD75z4Wqo/edit#heading=h.zf1ib2cgt533" target="_blank">{"FTO TCP"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/1uc5V3G-kRH3qV5b6Lq1yzTjnqSMS3D3dZcsCIOiJqcI/edit?gid=1409661599#gid=1409661599" target="_blank">{"2H CMLL"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/1WHnksK4yyv63sv3Es-PuKfnFRtZGafqp6rEuEfOx-cg/edit?gid=1409661599#gid=1409661599" target="_blank">{"OH CMLL"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/e/2PACX-1vSKmPOKUtF0g1FUn7wEFtyZJFYegBXuYxlA4ZcUDYmoqC4xBQOXnsaKgAydGoohlI-c97HeRpMyec7d/pubhtml?gid=0&single=true" target="_blank">{"OH PLL"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/1-uwmZHf4vwJxFgeB3-TiF8MQ0RFSS30d5CUK96PoIwk/edit?gid=0#gid=0" target="_blank">{"Juju ZBLL"}</a>
                                <a href="https://docs.google.com/presentation/d/1cD9AdZ1GYT2Jxznlbtn1yNXOnAATZRgEW1I2qcyorok/edit?slide=id.p#slide=id.p" target="_blank">{"Anthony and Jonas OH OLL"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/1s8Q2VM2c1eV3oLIxFfM4exvABO0AcCeMCUdMiYKnzC0/edit?gid=0#gid=0" target="_blank">{"ZBLS Tao Yu"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/1VQNYNwdOLqqBkacHcfYtEBst22FOVhH9EAhTOYOZTgo/edit?gid=467747530#gid=467747530" target="_blank">{"Daniel's PBL Document"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/1txRjB-fAWWSM-1e1w-eJbDp9xhu0ZDwLWQdswNZhcew/edit?gid=804375502#gid=804375502" target="_blank">{"Elliot Full Floating"}</a>
                                <a href="https://zwegner.github.io/cubing/fto/lbt-algs.html" target="_blank">{"Zach Wegner's LBT Algs"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/1Dh4MEQzpCuTylqnkn09UxBEvEq9MNYMqMiWQgFsBuYE/edit?gid=418973378#gid=418973378" target="_blank">{"2-Look Last Triplet (Nautilus)"}</a>
                                <a href="https://docs.google.com/spreadsheets/d/1Z5tekV-6TZExwB9kylOtuQk7X0Zkt5llIfz11Ydkrk4/edit?gid=554409358#gid=554409358" target="_blank">{"FSLL Introduction and Algs"}</a>
                            </div>
                            <div class="colFlex">
                                <b>{"Other"}</b>
                                <a href="https://reinierschippers.nl/oll_trainer-master/" target="_blank">{"Reinier's OLL Trainer"}</a>
                                <a href="https://bestsiteever.ru/oll/" target="_blank">{"Roman's OLL Trainer"}</a>
                                <a href="https://trangium.github.io/BatchSolver/" target="_blank">{"Batch Solver"}</a>
                                <a href="https://github.com/mihlefeld/Alg-Trainers" target="_blank">{"GitHub"}</a>
                            </div>
                        </div>
                    </div>
                </dialog>
            }
        </main>
    }
}

fn format_time(ms: i64) -> String {
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

fn exponential_moving_average(values: &[f64], factor: f64) -> Vec<f64> {
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

fn set_body_background(value: &str) {
    if let Some(body) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.body())
    {
        let _ = body.style().set_property("background", value);
    }
}

fn apply_settings(settings: &serde_json::Value) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    let base_size = settings
        .get("baseSize")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(1.0);
    let style = body.style();
    let _ = style.set_property("font-size", &format!("{base_size}em"));
    let colors = settings
        .get("colors")
        .and_then(serde_json::Value::as_object);
    let default_theme = if prefersLightTheme().unwrap_or(false) {
        "light"
    } else {
        "dark"
    };
    for (key, fallback) in default_theme_colors(default_theme) {
        let value = colors
            .and_then(|colors| colors.get(key))
            .and_then(serde_json::Value::as_str)
            .unwrap_or(fallback);
        let _ = style.set_property(key, value);
    }
    let _ = style.set_property(
        "--backgroundDarker",
        &format!("color-mix(in oklch, var(--background), black 20%)"),
    );
    let derived_text = if default_theme == "light" {
        ("var(--text)", "var(--background)", "var(--text)")
    } else {
        (
            "var(--background)",
            "var(--background)",
            "var(--background)",
        )
    };
    let _ = style.set_property("--secondaryText", derived_text.0);
    let _ = style.set_property("--primaryText", derived_text.1);
    let _ = style.set_property("--accentText", derived_text.2);
    let primary = setting_color_with_theme(settings, "--primary", default_theme);
    let secondary = setting_color_with_theme(settings, "--secondary", default_theme);
    let accent = setting_color_with_theme(settings, "--accent", default_theme);
    let _ = style.set_property("--primaryBG", &primary);
    let _ = style.set_property(
        "--primaryBGHover",
        &format!("color-mix(in oklch, {primary}, var(--background) 18%)"),
    );
    let _ = style.set_property(
        "--secondaryHover",
        &format!("color-mix(in oklch, {secondary}, var(--background) 18%)"),
    );
    let _ = style.set_property(
        "--accentHover",
        &format!("color-mix(in oklch, {accent}, var(--background) 18%)"),
    );
}

fn default_theme_colors(theme: &str) -> BTreeMap<&'static str, &'static str> {
    if theme == "light" {
        BTreeMap::from([
            ("--text", "oklch(25% 0 0)"),
            ("--background", "oklch(96% 0.01 97)"),
            ("--secondary", "oklch(75% 0.09 99)"),
            ("--primary", "oklch(45% 0.08 136)"),
            ("--accent", "oklch(85% 0.16 98)"),
        ])
    } else {
        BTreeMap::from([
            ("--text", "oklch(99% 0 91)"),
            ("--background", "oklch(33% 0 0)"),
            ("--secondary", "oklch(69% 0.09 98)"),
            ("--primary", "oklch(82% 0.07 135)"),
            ("--accent", "oklch(91% 0.16 98)"),
        ])
    }
}

fn setting_color(settings: &serde_json::Value, key: &'static str) -> String {
    let theme = if prefersLightTheme().unwrap_or(false) {
        "light"
    } else {
        "dark"
    };
    setting_color_with_theme(settings, key, theme)
}

fn setting_color_with_theme(
    settings: &serde_json::Value,
    key: &'static str,
    theme: &str,
) -> String {
    settings
        .get("colors")
        .and_then(|colors| colors.get(key))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| default_theme_colors(theme)[key])
        .to_owned()
}

fn default_cube_colors() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("white", "#FAFAFA"),
        ("ignore", "#777777"),
        ("black", "#222222"),
        ("lightblue", "#A0CBE8"),
        ("blue", "#4E79A7"),
        ("red", "#E15759"),
        ("pink", "#FF9D9A"),
        ("beige", "#F4F3AA"),
        ("yellow", "#F1CE63"),
        ("green", "#59A14F"),
        ("lightgreen", "#8CD17D"),
        ("purple", "#B07AA1"),
        ("orange", "#F28E2B"),
    ])
}

fn color_input_style(value: &str, background: &str) -> String {
    match colorInputStyle(value, background) {
        Ok(style) => style,
        Err(error) => {
            web_sys::console::warn_1(&error);
            "background-color:var(--background);color:var(--text)".to_owned()
        }
    }
}

fn case_image_data(svg: &str, colors: &BTreeMap<String, String>) -> String {
    let originals = default_cube_colors();
    let mut rendered = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(index) = rest.find('#') {
        rendered.push_str(&rest[..index]);
        rest = &rest[index..];
        if let Some((name, original)) = originals
            .iter()
            .find(|(_, original)| rest.starts_with(**original))
        {
            rendered.push_str(colors.get(*name).map(String::as_str).unwrap_or(original));
            rest = &rest[original.len()..];
        } else {
            rendered.push('#');
            rest = &rest[1..];
        }
    }
    rendered.push_str(rest);
    format!("data:image/svg+xml;base64,{}", STANDARD.encode(rendered))
}

fn case_matches_search(
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

fn case_matches_text(case: &TrainerCase, bundle: &TrainerBundle, needle: &str) -> bool {
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

fn fuzzy_matches(needle: &str, haystack: &str) -> bool {
    let haystack = haystack
        .chars()
        .filter(|character| !character.is_whitespace());
    let mut remaining = haystack;
    needle
        .chars()
        .filter(|character| !character.is_whitespace())
        .all(|wanted| remaining.any(|candidate| candidate == wanted))
}

fn svg_polygon_fills(svg: &str) -> Vec<String> {
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

fn fills_match_pattern(fills: &[String], cells: &BTreeMap<usize, String>) -> bool {
    let mut label_colors = BTreeMap::<&str, &str>::new();
    let mut used_colors = std::collections::BTreeSet::new();
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

fn search_color_names() -> [&'static str; 12] {
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

fn cube_color(colors: &BTreeMap<String, String>, name: &str) -> String {
    colors
        .get(name)
        .map(String::as_str)
        .or_else(|| default_cube_colors().get(name).copied())
        .unwrap_or("#222222")
        .to_owned()
}

fn search_template_svg(
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

fn random_index(len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (Math::random() * len as f64).floor() as usize
    }
}

fn weighted_case(
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

fn case_probabilities(
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

fn make_scramble(
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
    } else if flip_slice != (parts.len() % 2 == 0) {
        if parts.len() >= 2 {
            let index = parts.len() - 2;
            parts[index].0 += 6;
            normalize_square_pair(&mut parts[index]);
            parts.insert(index + 1, (6, 0));
        }
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

fn make_uuid() -> String {
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

thread_local! {
    static SYNC_STARTED: RefCell<bool> = RefCell::new(false);
    static SYNC_INTERVAL: RefCell<Option<Interval>> = const { RefCell::new(None) };
    static SYNC_DEBOUNCE: RefCell<Option<Timeout>> = const { RefCell::new(None) };
    static LAST_SYNC: RefCell<f64> = RefCell::new(0.0);
    static LAST_SYNC_COMPLETED: RefCell<Option<f64>> = const { RefCell::new(None) };
}

const SYNC_RESULT_BATCH_SIZE: usize = 10_000;

fn start_sync_engine() {
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

async fn sync_now(force: bool) {
    let now = Date::now();
    let too_soon = !force && LAST_SYNC.with(|last| now - *last.borrow() < 10_000.0);
    if too_soon {
        return;
    }
    LAST_SYNC.with(|last| *last.borrow_mut() = now);
    let Ok(me) = Request::get("/api/me").send().await else {
        return;
    };
    let Ok(me) = me.json::<MeResponse>().await else {
        return;
    };
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
        if key == "ALLGSettings" {
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
            if let Ok(algs) = serde_json::from_str::<BTreeMap<String, serde_json::Value>>(value) {
                for (case_id, value) in algs {
                    for alg in parse_custom_algs(&value, &case_id) {
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
            if let Some((results, changed)) = parse_stored_results(key, value) {
                if changed {
                    put_stored(
                        key.clone(),
                        serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_owned()),
                    )
                    .await;
                }
                bundle
                    .results
                    .extend(results.into_iter().map(|result| ResultSync {
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
        if let Some((results, result_changed)) = parse_stored_results(&key, &value) {
            if result_changed {
                put_stored(
                    key,
                    serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_owned()),
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
        let key = "ALLGSettings".to_owned();
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
    for (key, algs) in remote_algs {
        if !dirty.contains_key(&key) {
            put_stored(
                key.clone(),
                serde_json::Value::Object(custom_algs_payload(&algs)).to_string(),
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
    for (key, results) in remote_results {
        if !dirty.contains_key(&key) {
            let mut merged = get_stored(&key)
                .await
                .and_then(|value| parse_stored_results(&key, &value).map(|(results, _)| results))
                .unwrap_or_default();
            for result in results {
                merged = merge_remote_result(merged, result);
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
        let imported = imported
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
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

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<App>::new().render();
}
use base64::{engine::general_purpose::STANDARD, Engine};

#[cfg(test)]
mod cube_image_tests {
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
    fn recoloring_does_not_replace_new_colors_twice() {
        let colors = BTreeMap::from([
            ("red".to_owned(), "#4E79A7".to_owned()),
            ("blue".to_owned(), "#123456".to_owned()),
        ]);
        let image = case_image_data("<svg fill='#E15759' stroke='#4E79A7'/>", &colors);
        let encoded = image.strip_prefix("data:image/svg+xml;base64,").unwrap();
        let decoded = String::from_utf8(STANDARD.decode(encoded).unwrap()).unwrap();
        assert_eq!(decoded, "<svg fill='#4E79A7' stroke='#123456'/>");
    }

    #[cfg(test)]
    mod algset_selection_tests {
        use super::{algset_click_action, AlgsetClickAction};

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
    }

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
}
