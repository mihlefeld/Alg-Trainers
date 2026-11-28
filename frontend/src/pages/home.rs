use crate::components::trainer::SettingsDialog;
use crate::logic::{home_layout, theme};
use crate::models::{MeResponse, TrainerSummary};
use crate::services::api::{fetch_account_status, fetch_trainers, logout};
use crate::services::pwa::isNavigatorOnline;
use crate::services::settings::use_settings;
use crate::Route;
use wasm_bindgen_futures::spawn_local;
use web_sys::DragEvent;
use yew::prelude::*;
use yew_router::prelude::*;

const SETTINGS_BUTTON: &str = "settings";
const EDIT_BUTTON: &str = "edit";

#[function_component(Home)]
pub fn home() -> Html {
    let trainers = use_state(Vec::<TrainerSummary>::new);
    let me = use_state(MeResponse::default);
    let offline = use_state(|| !isNavigatorOnline());
    let error = use_state(|| None::<String>);
    let editing = use_state(|| false);
    let settings_open = use_state(|| false);
    let dragged_item = use_state(|| None::<String>);
    let drop_target = use_state(|| None::<String>);
    let settings = use_settings();

    {
        let trainers = trainers.clone();
        let me = me.clone();
        let offline = offline.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            theme::set_body_background("var(--backgroundDarker)");
            spawn_local(async move {
                match fetch_trainers().await {
                    Ok(value) => trainers.set(value),
                    Err(message) => error.set(Some(message)),
                }
            });
            spawn_local(async move {
                let (account, is_offline) = fetch_account_status().await;
                me.set(account);
                offline.set(is_offline);
            });
            || ()
        });
    }

    let layout = home_layout::load(&settings.settings);
    let groups = home_layout::groups(&trainers, &layout, *editing);
    let has_new_trainers = home_layout::has_new_trainers(&layout, &trainers);

    let persist_layout = {
        let current_settings = (*settings.settings).clone();
        let on_update = settings.update.clone();
        Callback::from(move |layout: home_layout::HomeLayout| {
            let mut next = current_settings.clone();
            home_layout::store(&mut next, &layout);
            on_update.emit(next);
        })
    };

    let on_toggle_edit = {
        let editing = editing.clone();
        let trainers = (*trainers).clone();
        let layout = layout.clone();
        let persist_layout = persist_layout.clone();
        Callback::from(move |_| {
            if *editing {
                editing.set(false);
            } else {
                let mut next = layout.clone();
                home_layout::capture_catalog(&mut next, &trainers);
                persist_layout.emit(next);
                editing.set(true);
            }
        })
    };

    let on_reset = {
        let layout = layout.clone();
        let trainers = (*trainers).clone();
        let persist_layout = persist_layout.clone();
        Callback::from(move |_| {
            let mut next = layout.clone();
            home_layout::reset_to_defaults(&mut next, &trainers);
            persist_layout.emit(next);
        })
    };

    let on_logout = Callback::from(|_| {
        spawn_local(async move {
            let _ = logout().await;
            if let Some(window) = web_sys::window() {
                let _ = window.location().reload();
            }
        });
    });

    let on_open_settings = {
        let settings_open = settings_open.clone();
        Callback::from(move |_| settings_open.set(true))
    };
    let on_close_settings = {
        let settings_open = settings_open.clone();
        Callback::from(move |_| settings_open.set(false))
    };

    html! {
        <main class="colFlex home-page">
            <div class="colFlex pad">
                <h3>{"Welcome to ALLG-Trainer"}</h3>
            </div>
            <div class="home-toolbar rowFlex pad">
                <div class="home-account rowFlex center">
                    if me.authenticated {
                        <span class="borderedContainer pad">
                            {if *offline {
                                format!("Signed in as {} (offline)", me.email.as_deref().unwrap_or("WCA user"))
                            } else {
                                format!("Signed in as {}", me.email.as_deref().unwrap_or("WCA user"))
                            }}
                        </span>
                        if !*offline {
                            <button class="abutton" onclick={on_logout}>{"Sign out"}</button>
                        }
                    } else if *offline {
                        <span class="borderedContainer pad">{"Offline mode"}</span>
                    } else {
                        <a class="abutton" href="/api/auth/wca">{"Sign in with the WCA"}</a>
                    }
                </div>
                <div class="home-actions rowFlex">
                    <button
                        class="abutton compactButton material-symbols-outlined home-edit-button"
                        title={if *editing { "Finish editing home page" } else { "Edit home page" }}
                        aria-label={if *editing { "Finish editing home page" } else { "Edit home page" }}
                        onclick={on_toggle_edit}
                    >
                        {if *editing { "done" } else { EDIT_BUTTON }}
                        if has_new_trainers && !*editing {
                            <span class="home-unread-dot" aria-label="New trainers available"></span>
                        }
                    </button>
                    <button
                        class="abutton compactButton material-symbols-outlined"
                        title="Settings"
                        aria-label="Settings"
                        onclick={on_open_settings}
                    >
                        {SETTINGS_BUTTON}
                    </button>
                </div>
            </div>

            if *editing {
                <div class="home-edit-help borderedContainer pad rowFlex center">
                    <span>{"Drag trainers or puzzle sections to rearrange them. Use the eye buttons to hide or show them."}</span>
                    <button
                        class="abutton compactButton home-reset-button"
                        title="Reset homepage to defaults"
                        aria-label="Reset homepage to defaults"
                        onclick={on_reset}
                    >
                        <svg viewBox="0 0 24 24" aria-hidden="true">
                            <path d="M4 5v5h5M5 10a7 7 0 1 1 2 7" />
                        </svg>
                    </button>
                </div>
            }

            <div id="algtrainer-selections" class={classes!("rowFlex", "pad", editing.then_some("home-editing"))}>
                { for groups.into_iter().map(|(puzzle, entries)| {
                    let hidden = layout.hidden_puzzles.contains(&puzzle);
                    let puzzle_drop_key = format!("puzzle:{puzzle}");
                    let puzzle_is_drop_target =
                        drop_target.as_deref() == Some(puzzle_drop_key.as_str());
                    let icon = entries.first()
                        .or_else(|| trainers.iter().find(|trainer| trainer.puzzle == puzzle))
                        .map(|trainer| format!("/catalog/{}/icon.png", trainer.trainer_id))
                        .unwrap_or_default();
                    let on_drag_start = {
                        let puzzle = puzzle.clone();
                        let dragged_item = dragged_item.clone();
                        Callback::from(move |event: DragEvent| {
                            let value = format!("puzzle:{puzzle}");
                            start_drag(&event, &value);
                            dragged_item.set(Some(value));
                        })
                    };
                    let on_drag_over = {
                        let dragged_item = dragged_item.clone();
                        let drop_target = drop_target.clone();
                        let puzzle_drop_key = puzzle_drop_key.clone();
                        Callback::from(move |event: DragEvent| {
                            if dragged_item.as_deref().is_some_and(|item| item.starts_with("puzzle:")) {
                                event.prevent_default();
                                drop_target.set(Some(puzzle_drop_key.clone()));
                            }
                        })
                    };
                    let on_drag_end = {
                        let dragged_item = dragged_item.clone();
                        let drop_target = drop_target.clone();
                        Callback::from(move |_| {
                            dragged_item.set(None);
                            drop_target.set(None);
                        })
                    };
                    let on_drop = {
                        let puzzle = puzzle.clone();
                        let layout = layout.clone();
                        let persist_layout = persist_layout.clone();
                        let dragged_item = dragged_item.clone();
                        let drop_target = drop_target.clone();
                        Callback::from(move |event: DragEvent| {
                            event.stop_propagation();
                            event.prevent_default();
                            if let Some(dragged) = drag_value(&event) {
                                if let Some(source) = dragged.strip_prefix("puzzle:") {
                                    let mut next = layout.clone();
                                    home_layout::move_puzzle(&mut next, source, &puzzle);
                                    persist_layout.emit(next);
                                }
                            }
                            dragged_item.set(None);
                            drop_target.set(None);
                        })
                    };
                    let toggle_visibility = {
                        let puzzle = puzzle.clone();
                        let layout = layout.clone();
                        let persist_layout = persist_layout.clone();
                        Callback::from(move |_| {
                            let mut next = layout.clone();
                            if !next.hidden_puzzles.remove(&puzzle) {
                                next.hidden_puzzles.insert(puzzle.clone());
                            }
                            persist_layout.emit(next);
                        })
                    };
                    html! {
                        <div
                            class={classes!(
                                "borderedContainer",
                                "pad",
                                "colFlex",
                                "home-puzzle",
                                hidden.then_some("home-hidden"),
                                puzzle_is_drop_target.then_some("home-drop-before")
                            )}
                            draggable={if *editing { "true" } else { "false" }}
                            ondragstart={(*editing).then_some(on_drag_start)}
                            ondragover={(*editing).then_some(on_drag_over)}
                            ondrop={(*editing).then_some(on_drop)}
                            ondragend={(*editing).then_some(on_drag_end)}
                        >
                            <div class="puzzle-header borderedContainer pad">
                                <span class="home-drag-label">
                                    if *editing { <span class="material-symbols-outlined home-drag-handle">{"drag_indicator"}</span> }
                                    <span>{puzzle.clone()}</span>
                                </span>
                                if *editing {
                                    <button
                                        class="home-visibility-button material-symbols-outlined"
                                        title={if hidden { "Show puzzle section" } else { "Hide puzzle section" }}
                                        aria-label={if hidden { "Show puzzle section" } else { "Hide puzzle section" }}
                                        onclick={toggle_visibility}
                                    >
                                        {if hidden { "visibility" } else { "visibility_off" }}
                                    </button>
                                } else if !hidden {
                                    <img src={icon} />
                                }
                            </div>
                            <div class="rowFlex home-trainer-list">
                                { for entries.into_iter().map(|entry| {
                                    let trainer_hidden = layout.hidden_trainers.contains(&entry.trainer_id);
                                    let trainer_drop_key =
                                        format!("trainer:{puzzle}:{}", entry.trainer_id);
                                    let trainer_is_drop_target =
                                        drop_target.as_deref() == Some(trainer_drop_key.as_str());
                                    let on_drag_start = {
                                        let id = entry.trainer_id.clone();
                                        let puzzle = puzzle.clone();
                                        let dragged_item = dragged_item.clone();
                                        Callback::from(move |event: DragEvent| {
                                            let value = format!("trainer:{puzzle}:{id}");
                                            start_drag(&event, &value);
                                            dragged_item.set(Some(value));
                                        })
                                    };
                                    let on_drag_over = {
                                        let dragged_item = dragged_item.clone();
                                        let drop_target = drop_target.clone();
                                        let trainer_puzzle = puzzle.clone();
                                        let trainer_drop_key = trainer_drop_key.clone();
                                        Callback::from(move |event: DragEvent| {
                                            let Some(item) = dragged_item.as_deref() else { return };
                                            if item.strip_prefix(&format!("trainer:{trainer_puzzle}:")).is_some()
                                                || item.starts_with("puzzle:")
                                            {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                drop_target.set(Some(if item.starts_with("puzzle:") {
                                                    format!("puzzle:{trainer_puzzle}")
                                                } else {
                                                    trainer_drop_key.clone()
                                                }));
                                            }
                                        })
                                    };
                                    let on_drag_end = {
                                        let dragged_item = dragged_item.clone();
                                        let drop_target = drop_target.clone();
                                        Callback::from(move |_| {
                                            dragged_item.set(None);
                                            drop_target.set(None);
                                        })
                                    };
                                    let on_drop = {
                                        let id = entry.trainer_id.clone();
                                        let trainer_puzzle = puzzle.clone();
                                        let layout = layout.clone();
                                        let persist_layout = persist_layout.clone();
                                        let dragged_item = dragged_item.clone();
                                        let drop_target = drop_target.clone();
                                        Callback::from(move |event: DragEvent| {
                                            event.stop_propagation();
                                            event.prevent_default();
                                            let Some(value) = drag_value(&event) else { return };
                                            let mut next = layout.clone();
                                            if let Some(source) = value.strip_prefix(&format!("trainer:{trainer_puzzle}:")) {
                                                home_layout::move_trainer(&mut next, &trainer_puzzle, source, &id);
                                                persist_layout.emit(next);
                                            } else if let Some(source) = value.strip_prefix("puzzle:") {
                                                home_layout::move_puzzle(&mut next, source, &trainer_puzzle);
                                                persist_layout.emit(next);
                                            }
                                            dragged_item.set(None);
                                            drop_target.set(None);
                                        })
                                    };
                                    let toggle_visibility = {
                                        let id = entry.trainer_id.clone();
                                        let layout = layout.clone();
                                        let persist_layout = persist_layout.clone();
                                        Callback::from(move |_| {
                                            let mut next = layout.clone();
                                            if !next.hidden_trainers.remove(&id) {
                                                next.hidden_trainers.insert(id.clone());
                                            }
                                            persist_layout.emit(next);
                                        })
                                    };
                                    if *editing {
                                        html! {
                                            <div
                                                class={classes!(
                                                    "abutton",
                                                    "grow",
                                                    "home-trainer",
                                                    trainer_hidden.then_some("home-hidden"),
                                                    trainer_is_drop_target.then_some("home-drop-before")
                                                )}
                                                draggable="true"
                                                ondragstart={on_drag_start}
                                                ondragover={on_drag_over}
                                                ondrop={on_drop}
                                                ondragend={on_drag_end}
                                            >
                                                <span class="material-symbols-outlined home-drag-handle">{"drag_indicator"}</span>
                                                <b>{entry.name}</b>
                                                <button
                                                    class="home-visibility-button material-symbols-outlined"
                                                    title={if trainer_hidden { "Show trainer" } else { "Hide trainer" }}
                                                    aria-label={if trainer_hidden { "Show trainer" } else { "Hide trainer" }}
                                                    onclick={toggle_visibility}
                                                >
                                                    {if trainer_hidden { "visibility" } else { "visibility_off" }}
                                                </button>
                                            </div>
                                        }
                                    } else {
                                        html! {
                                            <Link<Route> classes="abutton grow" to={Route::Trainer { id: entry.trainer_id }}>
                                                <b>{entry.name}</b>
                                            </Link<Route>>
                                        }
                                    }
                                }) }
                            </div>
                        </div>
                    }
                }) }
                if let Some(message) = error.as_ref() {
                    <div class="borderedContainer pad">{message}</div>
                }
            </div>

            <span class="version" style="text-align: center;">{crate::APP_VERSION}</span>

            if *settings_open {
                <SettingsDialog
                    settings={(*settings.settings).clone()}
                    settings_ready={settings.ready}
                    selection_key=""
                    home_page=true
                    on_update_settings={settings.update.clone()}
                    on_close={on_close_settings}
                />
            }
        </main>
    }
}

fn start_drag(event: &DragEvent, value: &str) {
    event.stop_propagation();
    if let Some(transfer) = event.data_transfer() {
        transfer.set_effect_allowed("move");
        let _ = transfer.set_data("text/plain", value);
    }
}

fn drag_value(event: &DragEvent) -> Option<String> {
    event
        .data_transfer()
        .and_then(|transfer| transfer.get_data("text/plain").ok())
        .filter(|value| !value.is_empty())
}
