use crate::components::settings_controls::SettingsSection;
use crate::models::MeResponse;
use crate::services::api::{fetch_account_status, load_cached_account, logout};
use crate::services::pwa::{applyPwaUpdate, checkForPwaUpdate, isNavigatorOnline};
use crate::services::sync::{last_sync_label, sync_now, SyncOutcome};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use yew::prelude::*;

#[function_component(AccountSettings)]
pub fn account_settings() -> Html {
    let account = use_state(MeResponse::default);
    let offline = use_state(|| !isNavigatorOnline());
    let syncing = use_state(|| false);
    let sync_time = use_state(last_sync_label);
    let error = use_state(|| None::<String>);
    let checking_updates = use_state(|| false);
    let update_ready = use_state(|| false);
    let update_status = use_state(|| None::<String>);

    {
        let account = account.clone();
        let offline = offline.clone();
        let sync_time = sync_time.clone();
        use_effect_with((), move |_| {
            let account_request = account.clone();
            let offline_request = offline.clone();
            spawn_local(async move {
                let (value, is_offline) = fetch_account_status().await;
                account_request.set(value);
                offline_request.set(is_offline);
            });
            let sync_time_listener = sync_time.clone();
            let account_online = account.clone();
            let offline_online = offline.clone();
            let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                sync_time_listener.set(last_sync_label());
                offline_online.set(false);
                let account_online = account_online.clone();
                spawn_local(async move {
                    if let Some(cached) = load_cached_account().await {
                        account_online.set(cached);
                    }
                });
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
        let account = account.clone();
        let syncing = syncing.clone();
        let sync_time = sync_time.clone();
        let offline = offline.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let account = account.clone();
            let syncing = syncing.clone();
            let sync_time = sync_time.clone();
            let offline = offline.clone();
            let error = error.clone();
            syncing.set(true);
            error.set(None);
            spawn_local(async move {
                let outcome = sync_now(true).await;
                match outcome {
                    SyncOutcome::Synced | SyncOutcome::Throttled => {
                        offline.set(false);
                        error.set(None);
                    }
                    SyncOutcome::Offline => {
                        offline.set(true);
                        error.set(outcome.user_message());
                    }
                    SyncOutcome::Unauthenticated => {
                        offline.set(false);
                        account.set(MeResponse::default());
                        error.set(outcome.user_message());
                    }
                    other => {
                        offline.set(false);
                        error.set(other.user_message());
                    }
                }
                sync_time.set(last_sync_label());
                syncing.set(false);
            });
        })
    };

    let on_check_updates = {
        let checking_updates = checking_updates.clone();
        let update_ready = update_ready.clone();
        let update_status = update_status.clone();
        Callback::from(move |_| {
            if *update_ready {
                if applyPwaUpdate() {
                    update_status.set(Some("Installing update...".to_owned()));
                } else {
                    update_ready.set(false);
                    update_status.set(Some("Unable to install update. Please reload.".to_owned()));
                }
                return;
            }
            let checking_updates = checking_updates.clone();
            let update_ready = update_ready.clone();
            let update_status = update_status.clone();
            checking_updates.set(true);
            update_status.set(None);
            spawn_local(async move {
                let status = match checkForPwaUpdate() {
                    Ok(promise) => JsFuture::from(promise)
                        .await
                        .ok()
                        .and_then(|value| value.as_string())
                        .unwrap_or_else(|| "error".to_owned()),
                    Err(_) => "error".to_owned(),
                };
                match status.as_str() {
                    "available" => {
                        update_ready.set(true);
                        update_status.set(Some("Update available and ready to install.".to_owned()));
                    }
                    "up-to-date" => {
                        update_ready.set(false);
                        update_status.set(Some("You are on the latest version.".to_owned()));
                    }
                    "offline" => {
                        update_status.set(Some("Offline — cannot check for updates.".to_owned()));
                    }
                    "unsupported" => {
                        update_status.set(Some(
                            "App updates are not supported in this browser.".to_owned(),
                        ));
                    }
                    _ => {
                        update_status.set(Some(
                            "Unable to check for updates. Please try again.".to_owned(),
                        ));
                    }
                }
                checking_updates.set(false);
            });
        })
    };

    let on_logout = {
        let account = account.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let account = account.clone();
            let error = error.clone();
            spawn_local(async move {
                if logout().await {
                    account.set(MeResponse::default());
                } else {
                    error.set(Some("Unable to sign out.".to_owned()));
                }
            });
        })
    };

    html! {
        <SettingsSection title="Account" class="accountDetails" open=true>
            <div class="colFlex accountSection">
                if account.authenticated {
                    <div class="settingsEntry accountIdentity">
                        <b>{account.wca_id.as_deref().unwrap_or("WCA user")}</b>
                        <span>{account.email.as_deref().unwrap_or("")}</span>
                    </div>
                    if *offline {
                        <div class="settingsEntry">
                            <span>{"Offline — progress is saved locally and will sync when back online."}</span>
                        </div>
                    }
                    <div class="settingsEntry accountSync">
                        <div class="colFlex">
                            <b>{"Last sync"}</b>
                            <span>{(*sync_time).clone()}</span>
                        </div>
                        <button class="abutton" disabled={*syncing} onclick={force_sync}>
                            {if *syncing { "Syncing..." } else { "Sync now" }}
                        </button>
                    </div>
                    <div class="settingsEntry accountUpdates">
                        <div class="colFlex">
                            <b>{"App updates"}</b>
                            if let Some(status) = update_status.as_ref() {
                                <span>{status}</span>
                            }
                        </div>
                        <button class="abutton" disabled={*checking_updates} onclick={on_check_updates.clone()}>
                            {if *checking_updates {
                                "Checking..."
                            } else if *update_ready {
                                "Update now"
                            } else {
                                "Look for updates"
                            }}
                        </button>
                    </div>
                    <div class="settingsEntry">
                        <div></div>
                        <button class="abutton" disabled={*offline} onclick={on_logout}>{"Sign out"}</button>
                    </div>
                } else {
                    <div class="settingsEntry accountLogin">
                        <div class="colFlex">
                            <b>{if *offline { "Offline" } else { "Not signed in" }}</b>
                            <span>{if *offline {
                                "Progress is saved locally on this device."
                            } else {
                                "Sign in to sync settings and results."
                            }}</span>
                        </div>
                        if !*offline {
                            <a class="abutton" href="/api/auth/wca">{"Sign in with WCA"}</a>
                        }
                    </div>
                    <div class="settingsEntry accountUpdates">
                        <div class="colFlex">
                            <b>{"App updates"}</b>
                            if let Some(status) = update_status.as_ref() {
                                <span>{status}</span>
                            }
                        </div>
                        <button class="abutton" disabled={*checking_updates} onclick={on_check_updates}>
                            {if *checking_updates {
                                "Checking..."
                            } else if *update_ready {
                                "Update now"
                            } else {
                                "Look for updates"
                            }}
                        </button>
                    </div>
                }
                if let Some(message) = error.as_ref() {
                    <div class="accountError" role="alert">{message}</div>
                }
            </div>
        </SettingsSection>
    }
}
