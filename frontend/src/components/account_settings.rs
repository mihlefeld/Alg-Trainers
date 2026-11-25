use crate::components::settings_controls::SettingsSection;
use crate::models::MeResponse;
use crate::services::api::{fetch_account_status, load_cached_account, logout};
use crate::services::pwa::isNavigatorOnline;
use crate::services::sync::{last_sync_label, sync_now};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(AccountSettings)]
pub fn account_settings() -> Html {
    let account = use_state(MeResponse::default);
    let offline = use_state(|| !isNavigatorOnline());
    let syncing = use_state(|| false);
    let sync_time = use_state(last_sync_label);
    let error = use_state(|| None::<String>);

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
        let syncing = syncing.clone();
        let sync_time = sync_time.clone();
        let offline = offline.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let syncing = syncing.clone();
            let sync_time = sync_time.clone();
            let offline = offline.clone();
            let error = error.clone();
            syncing.set(true);
            error.set(None);
            spawn_local(async move {
                let before = last_sync_label();
                sync_now(true).await;
                let after = last_sync_label();
                if before == after {
                    offline.set(!isNavigatorOnline());
                    error.set(Some(
                        "Sync did not complete. Please try again when online.".to_owned(),
                    ));
                } else {
                    offline.set(false);
                }
                sync_time.set(after);
                syncing.set(false);
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
        <SettingsSection title="Account" class="accountDetails">
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
                }
                if let Some(message) = error.as_ref() {
                    <div class="accountError" role="alert">{message}</div>
                }
            </div>
        </SettingsSection>
    }
}
