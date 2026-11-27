use crate::models::{MeResponse, TrainerBundle, TrainerSummary};
use crate::services::storage::{metadataDelete, metadataGet, metadataPut};
use gloo_net::http::Request;
use wasm_bindgen_futures::JsFuture;

const CACHED_ACCOUNT_KEY: &str = "cachedAccount";
const SYNC_CURSOR_KEY: &str = "syncCursor";

pub async fn load_cached_account() -> Option<MeResponse> {
    let promise = metadataGet(CACHED_ACCOUNT_KEY).ok()?;
    let value = JsFuture::from(promise).await.ok()?;
    let text = value.as_string()?;
    serde_json::from_str::<MeResponse>(&text)
        .ok()
        .filter(|account| account.authenticated)
}

pub async fn save_cached_account(account: &MeResponse) {
    if account.authenticated {
        if let Ok(serialized) = serde_json::to_string(account) {
            if let Ok(promise) = metadataPut(CACHED_ACCOUNT_KEY, &serialized) {
                let _ = JsFuture::from(promise).await;
            }
        }
    } else {
        clear_cached_account().await;
    }
}

pub async fn clear_cached_account() {
    if let Ok(promise) = metadataDelete(CACHED_ACCOUNT_KEY) {
        let _ = JsFuture::from(promise).await;
    }
}

pub async fn fetch_account_status() -> (MeResponse, bool) {
    match Request::get("/api/me").send().await {
        Ok(response) if response.ok() => match response.json::<MeResponse>().await {
            Ok(value) => {
                save_cached_account(&value).await;
                (value, false)
            }
            Err(_) => (load_cached_account().await.unwrap_or_default(), true),
        },
        _ => (load_cached_account().await.unwrap_or_default(), true),
    }
}

pub async fn logout() -> bool {
    match Request::post("/api/auth/logout").send().await {
        Ok(response) if response.ok() => {
            clear_cached_account().await;
            if let Ok(promise) = metadataDelete(SYNC_CURSOR_KEY) {
                let _ = JsFuture::from(promise).await;
            }
            true
        }
        _ => false,
    }
}

pub async fn fetch_trainers() -> Result<Vec<TrainerSummary>, String> {
    match Request::get("/api/trainers").send().await {
        Ok(response) if response.ok() => response
            .json::<Vec<TrainerSummary>>()
            .await
            .map_err(|error| format!("Unable to decode trainer list: {error}")),
        Ok(response) => Err(format!(
            "Unable to load trainer list (HTTP {}).",
            response.status()
        )),
        Err(error) => Err(format!("Unable to load trainer list: {error}")),
    }
}

pub async fn fetch_trainer_bundle(id: &str) -> Result<TrainerBundle, String> {
    match Request::get(&format!("/api/trainers/{id}")).send().await {
        Ok(response) if response.ok() => response
            .json::<TrainerBundle>()
            .await
            .map_err(|cause| format!("Unable to decode trainer: {cause}")),
        _ => Err("Unable to load this trainer.".to_owned()),
    }
}
