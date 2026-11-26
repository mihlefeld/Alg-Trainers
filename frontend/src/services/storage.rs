use crate::logic::{algs, cube_schemes, results};
use crate::models::{SelectionPreset, StoredResult, TrainerCase, UserAlgState};
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

pub const SETTINGS_KEY: &str = "ALLGSettings";

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
export function isMobileLayout() {
  if (!window.matchMedia) return false;
  return window.matchMedia("(max-width: 700px)").matches ||
    window.matchMedia("(pointer: coarse)").matches;
}
export function isFirefoxAndroid() {
  if (!navigator.userAgent) return false;
  return /Android/i.test(navigator.userAgent) && /Firefox/i.test(navigator.userAgent);
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
export async function metadataGet(key) {
  const db = await openStorage();
  return await new Promise((resolve, reject) => {
    const request = db.transaction("metadata", "readonly").objectStore("metadata").get(key);
    request.onsuccess = () => resolve(request.result ?? null);
    request.onerror = () => reject(request.error);
  });
}
export async function metadataPut(key, value) {
  const db = await openStorage();
  await new Promise((resolve, reject) => {
    const tx = db.transaction("metadata", "readwrite");
    tx.objectStore("metadata").put(value, key);
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
}
export async function metadataDelete(key) {
  const db = await openStorage();
  await new Promise((resolve, reject) => {
    const tx = db.transaction("metadata", "readwrite");
    tx.objectStore("metadata").delete(key);
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    pub fn storageGet(key: &str) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn storagePut(key: &str, value: &str) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn storageAll(store_name: &str) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn storageClearDirtyIfUnchanged(expected: &JsValue) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn metadataGet(key: &str) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn metadataPut(key: &str, value: &str) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn metadataDelete(key: &str) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn prefersLightTheme() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn isMobileLayout() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn isFirefoxAndroid() -> Result<bool, JsValue>;
    #[wasm_bindgen(catch)]
    pub fn colorInputStyle(value: &str, background: &str) -> Result<String, JsValue>;
    pub fn downloadText(filename: &str, text: &str);
    #[wasm_bindgen(catch)]
    pub fn readFileText(file: &web_sys::File) -> Result<js_sys::Promise, JsValue>;
    pub fn randomAlgId() -> String;
    #[wasm_bindgen(catch)]
    pub fn importStorageText(text: &str) -> Result<js_sys::Promise, JsValue>;
}

pub async fn get_stored(key: &str) -> Option<String> {
    let result = match storageGet(key) {
        Ok(promise) => JsFuture::from(promise).await,
        Err(error) => Err(error),
    };
    match result {
        Ok(value) => value.as_string(),
        Err(error) => {
            web_sys::console::error_1(&error);
            if key == SETTINGS_KEY {
                if let Some(window) = web_sys::window() {
                    let _ =
                        window.alert_with_message("Unable to load settings from browser storage.");
                }
            }
            None
        }
    }
}

pub async fn put_stored(key: String, value: String) {
    let result = match storagePut(&key, &value) {
        Ok(promise) => JsFuture::from(promise).await.map(|_| ()),
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        web_sys::console::error_1(&error);
        if key == SETTINGS_KEY {
            if let Some(window) = web_sys::window() {
                let _ = window.alert_with_message("Unable to save settings to browser storage.");
            }
        }
    }
}

pub async fn put_stored_results(
    key: String,
    incoming: Vec<StoredResult>,
    deleted_ids: Vec<String>,
) {
    let mut merged = get_stored(&key)
        .await
        .and_then(|value| results::parse_stored_results(&key, &value).map(|(items, _)| items))
        .unwrap_or_default();
    let deleted_ids = deleted_ids.into_iter().collect::<BTreeSet<_>>();
    for result in &mut merged {
        if deleted_ids.contains(&result.id) {
            result.deleted = true;
        }
    }
    for mut result in incoming {
        if deleted_ids.contains(&result.id) {
            result.deleted = true;
        }
        if let Some(existing) = merged.iter_mut().find(|existing| existing.id == result.id) {
            *existing = result;
        } else {
            merged.push(result);
        }
    }
    merged.sort_by(results::compare_results);
    put_stored(
        key,
        serde_json::to_string(&merged).unwrap_or_else(|_| "[]".to_owned()),
    )
    .await;
}

pub async fn load_settings(persist_migration: bool) -> serde_json::Value {
    let mut settings = serde_json::json!({});
    if let Some(value) = get_stored(SETTINGS_KEY).await {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&value) {
            settings = parsed;
            if cube_schemes::migrate(&mut settings) && persist_migration {
                put_stored(SETTINGS_KEY.to_owned(), settings.to_string()).await;
            }
        }
    }
    if settings.get("cubeColorSchemes").is_none() {
        cube_schemes::migrate(&mut settings);
    }
    settings
}

pub fn save_settings_async(settings: serde_json::Value) {
    spawn_local(async move {
        put_stored(SETTINGS_KEY.to_owned(), settings.to_string()).await;
    });
}

pub async fn load_custom_algs(
    selection_key: &str,
    cases: &[TrainerCase],
) -> BTreeMap<String, Vec<UserAlgState>> {
    let key = format!("{selection_key}customAlgs");
    let Some(value) = get_stored(&key).await else {
        return BTreeMap::new();
    };
    let Ok(parsed) = serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&value) else {
        return BTreeMap::new();
    };
    parsed
        .into_iter()
        .map(|(case_id, value)| {
            let mut items = algs::parse_custom_algs(&value, &case_id);
            if let Some(case) = cases.iter().find(|case| case.case_id == case_id) {
                items = algs::upgrade_legacy_custom_algs(case, items);
            }
            (case_id, items)
        })
        .collect()
}

pub fn save_custom_algs(selection_key: &str, custom_algs: &BTreeMap<String, Vec<UserAlgState>>) {
    let key = format!("{selection_key}customAlgs");
    let payload = serde_json::Value::Object(algs::custom_algs_payload(custom_algs)).to_string();
    spawn_local(async move {
        put_stored(key, payload).await;
    });
}

pub fn save_selected_cases(selection_key: &str, cases: Vec<String>) {
    let key = selection_key.to_owned();
    spawn_local(async move {
        put_stored(
            key,
            serde_json::to_string(&cases).unwrap_or_else(|_| "[]".to_owned()),
        )
        .await;
    });
}

pub fn save_selection_and_algsets(
    selection_key: &str,
    cases: Vec<String>,
    algsets: BTreeMap<String, bool>,
) {
    let key = selection_key.to_owned();
    spawn_local(async move {
        put_stored(
            key.clone(),
            serde_json::to_string(&cases).unwrap_or_else(|_| "[]".to_owned()),
        )
        .await;
        put_stored(
            format!("{key}Algsets"),
            serde_json::to_string(&algsets).unwrap_or_else(|_| "{}".to_owned()),
        )
        .await;
    });
}

pub fn save_presets(selection_key: &str, presets: BTreeMap<String, SelectionPreset>) {
    let key = format!("{selection_key}Presets");
    spawn_local(async move {
        put_stored(
            key,
            serde_json::to_string(&presets).unwrap_or_else(|_| "{}".to_owned()),
        )
        .await;
    });
}

pub async fn export_backup() {
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
}

pub async fn import_backup_file(file: &web_sys::File) -> Result<(), &'static str> {
    let promise = readFileText(file).map_err(|_| "Unable to read backup file")?;
    let text = JsFuture::from(promise)
        .await
        .map_err(|_| "Unable to read backup file")?
        .as_string()
        .ok_or("Unable to read backup file")?;
    let promise = importStorageText(&text).map_err(|_| "Invalid trainer backup")?;
    let _ = JsFuture::from(promise)
        .await
        .map_err(|_| "Unable to import trainer backup")?;
    Ok(())
}

pub async fn read_file_text(
    file: &web_sys::File,
    err_msg: &'static str,
) -> Result<String, &'static str> {
    let promise = readFileText(file).map_err(|_| err_msg)?;
    JsFuture::from(promise)
        .await
        .map_err(|_| err_msg)?
        .as_string()
        .ok_or(err_msg)
}
