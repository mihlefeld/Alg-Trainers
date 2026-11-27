use crate::components::modal::take_modal_history_pop;
use crate::components::trainer::timer::{use_trainer_timer, TrainerTimer};
use crate::logic::{algs, cube_schemes, results, scramble, search, selection, theme};
use crate::models::{
    Dialog, DisplayAlg, Mode, SelectionPreset, StoredResult, TrainerBundle, TrainerCase,
    UserAlgState,
};
use crate::services::settings::use_settings;
use crate::services::storage::{
    get_stored, load_custom_algs, put_stored, put_stored_results, randomAlgId, save_custom_algs,
    save_presets, save_selected_cases, save_selection_and_algsets,
};
use gloo_timers::callback::Timeout;
use js_sys::Date;
use std::collections::BTreeMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::KeyboardEvent;
use yew::prelude::*;

fn confirm_destructive_action(message: &str) -> bool {
    web_sys::window().is_some_and(|window| window.confirm_with_message(message).unwrap_or(false))
}

fn last_solve_deletion_message(case_name: &str, result_time: &str) -> String {
    format!("Delete the last solve?\n\nCase: {case_name}\nResult time: {result_time}")
}

pub struct TrainerController {
    pub bundle: Rc<TrainerBundle>,
    pub cases_by_id: Rc<BTreeMap<String, TrainerCase>>,
    pub selected: Vec<String>,
    pub selected_algsets: BTreeMap<String, bool>,
    pub recap_remaining_count: usize,
    pub recap_active: bool,
    pub mode: Mode,
    pub results: Vec<StoredResult>,
    pub dialog: Dialog,
    pub search_open: bool,
    pub search_query: String,
    pub search_cells: BTreeMap<usize, String>,
    pub search_color: String,
    pub search_all_angles: bool,
    pub settings: serde_json::Value,
    pub settings_ready: bool,
    pub presets: BTreeMap<String, SelectionPreset>,
    pub current_scramble: String,
    pub last_case: Option<TrainerCase>,
    pub last_scramble: String,
    pub hint_open: bool,
    pub hint_case: Option<TrainerCase>,
    pub stats_case_id: String,
    pub custom_algs: BTreeMap<String, Vec<UserAlgState>>,
    pub probabilities: Rc<BTreeMap<String, f64>>,
    pub cube_colors: Rc<BTreeMap<String, String>>,
    pub timer: TrainerTimer,
    pub on_key_down: Callback<KeyboardEvent>,
    pub on_key_up: Callback<KeyboardEvent>,
    pub on_open_settings: Callback<()>,
    pub on_start_training: Callback<()>,
    pub on_switch_to_train: Callback<()>,
    pub on_back_to_selection: Callback<()>,
    pub on_open_dialog: Callback<Dialog>,
    pub on_close_dialog: Callback<()>,
    pub on_toggle_case: Callback<String>,
    pub on_toggle_case_group: Callback<Vec<String>>,
    pub on_toggle_algset: Callback<(String, Vec<String>)>,
    pub on_use_preset: Callback<SelectionPreset>,
    pub on_add_preset_to_selection: Callback<SelectionPreset>,
    pub on_remove_preset_from_selection: Callback<SelectionPreset>,
    pub on_save_preset: Callback<String>,
    pub on_delete_preset: Callback<String>,
    pub on_open_hint_for_case: Callback<TrainerCase>,
    pub on_open_active_hint: Callback<()>,
    pub on_open_last_hint: Callback<()>,
    pub on_close_hint: Callback<()>,
    pub on_step_hint: Callback<isize>,
    pub on_save_alg: Callback<(Option<String>, String)>,
    pub on_hide_alg: Callback<DisplayAlg>,
    pub on_delete_alg: Callback<String>,
    pub on_reorder_alg: Callback<(String, String)>,
    pub on_open_stats: Callback<String>,
    pub on_clear_results: Callback<()>,
    pub on_delete_result: Callback<String>,
    pub on_bookmark_last_case: Callback<()>,
    pub on_remove_last_case: Callback<()>,
    pub on_remove_last_time: Callback<()>,
    pub on_update_settings: Callback<serde_json::Value>,
    pub on_toggle_search_open: Callback<()>,
    pub on_search_query_change: Callback<String>,
    pub on_search_cells_change: Callback<BTreeMap<usize, String>>,
    pub on_search_color_change: Callback<String>,
    pub on_search_all_angles_change: Callback<bool>,
}

#[hook]
pub fn use_trainer_controller(bundle: Rc<TrainerBundle>) -> TrainerController {
    let cases_by_id: Rc<BTreeMap<String, TrainerCase>> = Rc::new(
        bundle
            .cases
            .iter()
            .cloned()
            .map(|case| (case.case_id.clone(), case))
            .collect(),
    );

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
    let stored_results = use_state(Vec::<StoredResult>::new);
    let dialog = use_state(|| Dialog::None);
    let search_open = use_state(|| false);
    let search_query = use_state(String::new);
    let search_cells = use_state(BTreeMap::<usize, String>::new);
    let search_color = use_state(|| search::search_color_names()[0].to_owned());
    let search_all_angles = use_state(|| true);
    let settings_ctx = use_settings();
    let settings = settings_ctx.settings.clone();
    let settings_ready = settings_ctx.ready;
    let settings_ref = use_mut_ref(|| settings.clone());
    *settings_ref.borrow_mut() = settings.clone();
    let presets = use_state(BTreeMap::<String, SelectionPreset>::new);
    let current_scramble = use_state(String::new);
    let last_case = use_state(|| None::<TrainerCase>);
    let last_scramble = use_state(String::new);
    let hint_open = use_state(|| false);
    let hint_case = use_state(|| None::<TrainerCase>);
    let stats_case_id = use_state(String::new);
    let custom_algs = use_state(BTreeMap::<String, Vec<UserAlgState>>::new);

    let trainer_id = bundle.trainer_id.clone();
    let selection_storage_key = bundle.selection_key.clone();
    let times_storage_key = bundle.times_key.clone();

    // Load persisted trainer state from IndexedDB when trainer changes.
    {
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let stored_results = stored_results.clone();
        let settings = settings.clone();
        let presets = presets.clone();
        let custom_algs = custom_algs.clone();
        let selection_key = selection_storage_key.clone();
        let times_key = times_storage_key.clone();
        let bundle = bundle.clone();
        let recap_active = recap_active.clone();
        let recap_remaining = recap_remaining.clone();
        let active_case = active_case.clone();
        let current_scramble = current_scramble.clone();
        use_effect_with(trainer_id, move |_| {
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
                    if let Some((parsed, changed)) =
                        results::parse_stored_results(&times_key, &value)
                    {
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
                        stored_results.set(loaded_results.clone());
                    }
                }
                if let Some(value) = get_stored(&format!("{selection_key}Presets")).await {
                    if let Ok(parsed) =
                        serde_json::from_str::<BTreeMap<String, SelectionPreset>>(&value)
                    {
                        presets.set(parsed);
                    }
                }
                let loaded_algs = load_custom_algs(&selection_key, &bundle.cases).await;
                if !loaded_algs.is_empty() {
                    custom_algs.set(loaded_algs);
                }
                if let Some(window) = web_sys::window() {
                    let query = window.location().search().unwrap_or_default();
                    let (case_id, remaining, active_recap) = scramble::pick_initial_case_for_query(
                        &query,
                        &loaded_selected,
                        &loaded_results,
                        &settings,
                    );
                    recap_remaining.set(remaining);
                    recap_active.set(active_recap);
                    if let Some(case) = case_id
                        .and_then(|id| bundle.cases.iter().find(|case| case.case_id == id).cloned())
                    {
                        current_scramble.set(scramble::make_scramble(
                            &case,
                            &bundle,
                            &settings,
                            &bundle.selection_key,
                        ));
                        active_case.set(Some(case));
                    }
                }
            });
            || ()
        });
    }

    // Reload trainer state from browser storage whenever remote sync completes.
    {
        let trainer_id = bundle.trainer_id.clone();
        let selection_key = selection_storage_key.clone();
        let times_key = times_storage_key.clone();
        let cases = bundle.cases.clone();
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let stored_results = stored_results.clone();
        let presets = presets.clone();
        let custom_algs = custom_algs.clone();
        use_effect_with(trainer_id, move |_| {
            let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                let selection_key = selection_key.clone();
                let times_key = times_key.clone();
                let cases = cases.clone();
                let selected = selected.clone();
                let selected_algsets = selected_algsets.clone();
                let stored_results = stored_results.clone();
                let presets = presets.clone();
                let custom_algs = custom_algs.clone();
                spawn_local(async move {
                    if let Some(value) = get_stored(&selection_key).await {
                        if let Ok(parsed) = serde_json::from_str::<Vec<String>>(&value) {
                            selected.set(parsed);
                        } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&value)
                        {
                            if let Some(cases) =
                                parsed.get("cases").and_then(serde_json::Value::as_array)
                            {
                                selected.set(
                                    cases
                                        .iter()
                                        .filter_map(serde_json::Value::as_str)
                                        .map(str::to_owned)
                                        .collect(),
                                );
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
                    }
                    if let Some(value) = get_stored(&format!("{selection_key}Algsets")).await {
                        if let Ok(parsed) = serde_json::from_str::<BTreeMap<String, bool>>(&value) {
                            selected_algsets.set(parsed);
                        }
                    }
                    if let Some(value) = get_stored(&times_key).await {
                        if let Some((parsed, _)) = results::parse_stored_results(&times_key, &value)
                        {
                            stored_results.set(
                                parsed
                                    .into_iter()
                                    .filter(|result| !result.deleted)
                                    .collect(),
                            );
                        }
                    }
                    if let Some(value) = get_stored(&format!("{selection_key}Presets")).await {
                        if let Ok(parsed) =
                            serde_json::from_str::<BTreeMap<String, SelectionPreset>>(&value)
                        {
                            presets.set(parsed);
                        }
                    }
                    let loaded_algs = load_custom_algs(&selection_key, &cases).await;
                    custom_algs.set(loaded_algs);
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

    // Sync trainer mode with browser query string (?train / ?recap) and history navigation.
    {
        let mode = mode.clone();
        let recap_active = recap_active.clone();
        let recap_remaining = recap_remaining.clone();
        let selected = selected.clone();
        let stored_results = stored_results.clone();
        let settings_ref = settings_ref.clone();
        let active_case = active_case.clone();
        let current_scramble = current_scramble.clone();
        let bundle = bundle.clone();
        use_effect_with((), move |_| {
            let set_from_location = Rc::new(move || {
                if let Some(window) = web_sys::window() {
                    let state = window.location().search().unwrap_or_default();
                    let current_settings = settings_ref.borrow().clone();
                    if state == "?train" || state == "?recap" {
                        let (case_id, remaining, is_recap) = scramble::pick_initial_case_for_query(
                            &state,
                            &selected,
                            &stored_results,
                            &current_settings,
                        );
                        mode.set(Mode::Practice);
                        theme::set_body_background("var(--backgroundDarker)");
                        recap_remaining.set(remaining);
                        recap_active.set(is_recap);
                        if let Some(case) = case_id.and_then(|id| {
                            bundle.cases.iter().find(|case| case.case_id == id).cloned()
                        }) {
                            current_scramble.set(scramble::make_scramble(
                                &case,
                                &bundle,
                                &current_settings,
                                &bundle.selection_key,
                            ));
                            active_case.set(Some(case));
                        }
                    } else {
                        mode.set(Mode::Select);
                        recap_active.set(false);
                        theme::set_body_background("var(--background)");
                    }
                }
            });
            set_from_location();
            let on_pop = set_from_location.clone();
            let listener = Closure::wrap(Box::new(move || {
                if take_modal_history_pop()
                    || web_sys::window()
                        .and_then(|window| window.document())
                        .and_then(|document| document.query_selector("dialog[open]").ok())
                        .flatten()
                        .is_some()
                {
                    return;
                }
                on_pop();
            }) as Box<dyn Fn()>);
            let window = web_sys::window();
            if let Some(window) = window.as_ref() {
                let _ = window.add_event_listener_with_callback(
                    "popstate",
                    listener.as_ref().unchecked_ref(),
                );
            }
            move || {
                theme::set_body_background("var(--backgroundDarker)");
                if let Some(window) = window {
                    let _ = window.remove_event_listener_with_callback(
                        "popstate",
                        listener.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    let choose_case = {
        let cases_by_id = cases_by_id.clone();
        let selected = selected.clone();
        let recap_remaining = recap_remaining.clone();
        let recap_active = recap_active.clone();
        let active_case = active_case.clone();
        let current_scramble = current_scramble.clone();
        let stored_results = stored_results.clone();
        let settings = settings.clone();
        let bundle = bundle.clone();
        Callback::from(move |_| {
            if selected.is_empty() {
                return;
            }
            let mut recap = (*recap_remaining).clone();
            let id = if *recap_active && !recap.is_empty() {
                Some(recap.remove(scramble::random_index(recap.len())))
            } else {
                recap_active.set(false);
                scramble::weighted_case(&selected, &stored_results, &settings)
            };
            recap_remaining.set(recap);
            let case = id.and_then(|id| cases_by_id.get(&id)).cloned();
            if let Some(case) = case.as_ref() {
                current_scramble.set(scramble::make_scramble(
                    case,
                    &bundle,
                    &settings,
                    &bundle.selection_key,
                ));
            }
            active_case.set(case);
        })
    };

    let on_timer_finish = {
        let active_case = active_case.clone();
        let stored_results = stored_results.clone();
        let times_key = times_storage_key.clone();
        let choose_case = choose_case.clone();
        let last_case = last_case.clone();
        let last_scramble = last_scramble.clone();
        let current_scramble = current_scramble.clone();
        Callback::from(move |ms: i64| {
            if let Some(case) = active_case.as_ref() {
                last_case.set(Some(case.clone()));
                last_scramble.set((*current_scramble).clone());
                let mut next = (*stored_results).clone();
                let now = Date::new_0()
                    .to_iso_string()
                    .as_string()
                    .unwrap_or_default();
                next.push(StoredResult {
                    id: results::make_uuid(),
                    datetime: now.clone(),
                    ms,
                    case: case.case_id.clone(),
                    time: results::format_time(ms),
                    deleted: false,
                    updated_at: Some(now),
                });
                stored_results.set(next.clone());
                let key = times_key.clone();
                spawn_local(async move {
                    put_stored_results(key, next, Vec::new()).await;
                });
            }
            choose_case.emit(());
        })
    };

    let timer = use_trainer_timer(settings.clone(), on_timer_finish);

    let on_start_training = {
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
            let (case_id, recap) = scramble::pick_recap_case(&selected);
            recap_remaining.set(recap);
            recap_active.set(true);
            if let Some(case) = case_id.and_then(|id| cases_by_id.get(&id).cloned()) {
                current_scramble.set(scramble::make_scramble(
                    &case,
                    &bundle,
                    &settings,
                    &bundle.selection_key,
                ));
                active_case.set(Some(case));
            }
            theme::set_body_background("var(--backgroundDarker)");
            mode.set(Mode::Practice);
            if let Some(window) = web_sys::window() {
                if let Ok(history) = window.history() {
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

    let on_switch_to_train = {
        let recap_remaining = recap_remaining.clone();
        let recap_active = recap_active.clone();
        let selected = selected.clone();
        let stored_results = stored_results.clone();
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
            if let Some(case_id) = scramble::weighted_case(&selected, &stored_results, &settings) {
                if let Some(case) = cases_by_id.get(&case_id).cloned() {
                    current_scramble.set(scramble::make_scramble(
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

    let on_back_to_selection = {
        let mode = mode.clone();
        Callback::from(move |_| {
            theme::set_body_background("var(--background)");
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
        })
    };

    let on_toggle_case = {
        let selected = selected.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |case_id: String| {
            let next = selection::toggle_single_case(&selected, &case_id);
            selected.set(next.clone());
            save_selected_cases(&selection_key, next);
        })
    };

    let on_toggle_case_group = {
        let selected = selected.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |case_ids: Vec<String>| {
            let next = selection::toggle_case_ids(&selected, &case_ids);
            selected.set(next.clone());
            save_selected_cases(&selection_key, next);
        })
    };

    let on_toggle_algset = {
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |(name, group_case_ids): (String, Vec<String>)| {
            let (cases, algsets) =
                selection::apply_algset_toggle(&selected, &selected_algsets, name, &group_case_ids);
            selected.set(cases.clone());
            selected_algsets.set(algsets.clone());
            save_selection_and_algsets(&selection_key, cases, algsets);
        })
    };

    let on_use_preset = {
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |preset: SelectionPreset| {
            selected.set(preset.sel_cases.clone());
            selected_algsets.set(preset.selected_alg_sets.clone());
            save_selection_and_algsets(&selection_key, preset.sel_cases, preset.selected_alg_sets);
        })
    };

    let on_add_preset_to_selection = {
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |preset: SelectionPreset| {
            let (cases, algsets) =
                selection::add_preset_cases(&selected, &selected_algsets, preset);
            selected.set(cases.clone());
            selected_algsets.set(algsets.clone());
            save_selection_and_algsets(&selection_key, cases, algsets);
        })
    };

    let on_remove_preset_from_selection = {
        let selected = selected.clone();
        let selected_algsets = selected_algsets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |preset: SelectionPreset| {
            let (cases, algsets) =
                selection::remove_preset_cases(&selected, &selected_algsets, preset);
            selected.set(cases.clone());
            selected_algsets.set(algsets.clone());
            save_selection_and_algsets(&selection_key, cases, algsets);
        })
    };

    let on_save_preset = {
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
            save_presets(&selection_key, next);
        })
    };

    let on_delete_preset = {
        let presets = presets.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |name: String| {
            let mut next = (*presets).clone();
            next.remove(&name);
            presets.set(next.clone());
            save_presets(&selection_key, next);
        })
    };

    let on_remove_last_time = {
        let stored_results = stored_results.clone();
        let times_key = times_storage_key.clone();
        let cases_by_id = cases_by_id.clone();
        Callback::from(move |_| {
            let Some(last_result) = (*stored_results).last() else {
                return;
            };
            let case_name = cases_by_id
                .get(&last_result.case)
                .map(|case| case.name.as_str())
                .unwrap_or(last_result.case.as_str());
            let result_time = if last_result.time.is_empty() {
                results::format_time(last_result.ms)
            } else {
                last_result.time.clone()
            };
            let message = last_solve_deletion_message(case_name, &result_time);
            if !confirm_destructive_action(&message) {
                return;
            }
            let mut next = (*stored_results).clone();
            let deleted_ids = next.pop().map(|result| vec![result.id]).unwrap_or_default();
            stored_results.set(next.clone());
            let key = times_key.clone();
            spawn_local(async move {
                put_stored_results(key, next, deleted_ids).await;
            });
        })
    };

    let on_clear_results = {
        let stored_results = stored_results.clone();
        let times_key = times_storage_key.clone();
        Callback::from(move |_| {
            if !confirm_destructive_action("Delete all times?") {
                return;
            }
            let deleted_ids = (*stored_results)
                .iter()
                .map(|result| result.id.clone())
                .collect();
            stored_results.set(Vec::new());
            let key = times_key.clone();
            spawn_local(async move {
                put_stored_results(key, Vec::new(), deleted_ids).await;
            });
        })
    };

    let on_delete_result = {
        let stored_results = stored_results.clone();
        let times_key = times_storage_key.clone();
        Callback::from(move |result_id: String| {
            let mut next = (*stored_results).clone();
            next.retain(|result| result.id != result_id);
            stored_results.set(next.clone());
            let key = times_key.clone();
            spawn_local(async move {
                put_stored_results(key, next, vec![result_id]).await;
            });
        })
    };

    let on_remove_last_case = {
        let last_case = last_case.clone();
        let on_toggle_case = on_toggle_case.clone();
        Callback::from(move |_| {
            if let Some(case) = last_case.as_ref() {
                let message = format!("Unselect the last case ({})?", case.name);
                if !confirm_destructive_action(&message) {
                    return;
                }
                on_toggle_case.emit(case.case_id.clone());
            }
        })
    };

    let on_bookmark_last_case = {
        let presets = presets.clone();
        let last_case = last_case.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |_| {
            if let Some(case) = last_case.as_ref() {
                let is_bookmarked = presets
                    .get("Bookmarks")
                    .is_some_and(|preset| preset.sel_cases.contains(&case.case_id));
                if is_bookmarked {
                    let message = format!("Remove the bookmark for ({})?", case.name);
                    if !confirm_destructive_action(&message) {
                        return;
                    }
                }
                let next = selection::toggle_bookmark(&presets, &case.case_id);
                presets.set(next.clone());
                save_presets(&selection_key, next);
            }
        })
    };

    let on_open_hint_for_case = {
        let hint_open = hint_open.clone();
        let hint_case = hint_case.clone();
        Callback::from(move |case: TrainerCase| {
            hint_case.set(Some(case));
            hint_open.set(true);
        })
    };

    let on_open_active_hint = {
        let active_case = active_case.clone();
        let on_open_hint_for_case = on_open_hint_for_case.clone();
        Callback::from(move |_| {
            if let Some(case) = active_case.as_ref() {
                on_open_hint_for_case.emit(case.clone());
            }
        })
    };

    let on_open_last_hint = {
        let last_case = last_case.clone();
        let on_open_hint_for_case = on_open_hint_for_case.clone();
        Callback::from(move |_| {
            if let Some(case) = last_case.as_ref() {
                on_open_hint_for_case.emit(case.clone());
            }
        })
    };

    let on_close_hint = {
        let hint_open = hint_open.clone();
        Callback::from(move |_| hint_open.set(false))
    };

    let on_step_hint = {
        let hint_case = hint_case.clone();
        let hint_open = hint_open.clone();
        let bundle = bundle.clone();
        let cases_by_id = cases_by_id.clone();
        let selected_algsets = selected_algsets.clone();
        let search_query = search_query.clone();
        let search_cells = search_cells.clone();
        let search_all_angles = search_all_angles.clone();
        Callback::from(move |delta: isize| {
            if let Some(current) = hint_case.as_ref() {
                let search_needle = search_query.trim().to_lowercase();
                let visible_case_ids = selection::ordered_visible_case_ids(
                    &bundle,
                    &selected_algsets,
                    &search_needle,
                    &search_cells,
                    *search_all_angles,
                );
                if let Some(target_id) =
                    selection::step_visible_case_id(&visible_case_ids, &current.case_id, delta)
                {
                    if let Some(next_case) = cases_by_id.get(&target_id) {
                        hint_case.set(Some(next_case.clone()));
                        hint_open.set(true);
                    }
                }
            }
        })
    };

    let on_save_alg = {
        let custom_algs = custom_algs.clone();
        let hint_case = hint_case.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |(alg_id, value): (Option<String>, String)| {
            let Some(case) = hint_case.as_ref() else {
                return;
            };
            if let Some(next) =
                algs::upsert_user_alg(&custom_algs, case, alg_id.as_deref(), randomAlgId, &value)
            {
                custom_algs.set(next.clone());
                save_custom_algs(&selection_key, &next);
            }
        })
    };

    let on_hide_alg = {
        let custom_algs = custom_algs.clone();
        let hint_case = hint_case.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |item: DisplayAlg| {
            let Some(case) = hint_case.as_ref() else {
                return;
            };
            let next = algs::hide_alg(&custom_algs, &case.case_id, item);
            custom_algs.set(next.clone());
            save_custom_algs(&selection_key, &next);
        })
    };

    let on_delete_alg = {
        let custom_algs = custom_algs.clone();
        let hint_case = hint_case.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |alg_id: String| {
            let Some(case) = hint_case.as_ref() else {
                return;
            };
            let next = algs::delete_alg(&custom_algs, &case.case_id, &alg_id);
            custom_algs.set(next.clone());
            save_custom_algs(&selection_key, &next);
        })
    };

    let on_reorder_alg = {
        let custom_algs = custom_algs.clone();
        let hint_case = hint_case.clone();
        let selection_key = selection_storage_key.clone();
        Callback::from(move |(from, to): (String, String)| {
            let Some(case) = hint_case.as_ref() else {
                return;
            };
            if let Some(next) = algs::reorder_alg(&custom_algs, case, &from, &to) {
                custom_algs.set(next.clone());
                save_custom_algs(&selection_key, &next);
            }
        })
    };

    let on_update_settings = settings_ctx.update.clone();

    let on_open_dialog = {
        let dialog = dialog.clone();
        Callback::from(move |next: Dialog| dialog.set(next))
    };
    let on_close_dialog = {
        let dialog = dialog.clone();
        Callback::from(move |_| dialog.set(Dialog::None))
    };
    let on_open_settings = {
        let on_open_dialog = on_open_dialog.clone();
        Callback::from(move |_| on_open_dialog.emit(Dialog::Settings))
    };
    let on_open_stats = {
        let dialog = dialog.clone();
        let stats_case_id = stats_case_id.clone();
        Callback::from(move |case_id: String| {
            stats_case_id.set(case_id);
            dialog.set(Dialog::Stats);
        })
    };

    let on_key_down = {
        let timer = timer.clone();
        let dialog = dialog.clone();
        let hint_open = hint_open.clone();
        let mode = mode.clone();
        let recap_active = recap_active.clone();
        let on_step_hint = on_step_hint.clone();
        let on_start_training = on_start_training.clone();
        let on_switch_to_train = on_switch_to_train.clone();
        let on_open_active_hint = on_open_active_hint.clone();
        let on_open_last_hint = on_open_last_hint.clone();
        let on_bookmark_last_case = on_bookmark_last_case.clone();
        let on_remove_last_case = on_remove_last_case.clone();
        let on_remove_last_time = on_remove_last_time.clone();
        let on_clear_results = on_clear_results.clone();
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
                } else if timer.running {
                    timer.abort.emit(());
                }
                return;
            }
            if timer.running && !is_typing && *dialog == Dialog::None && !*hint_open {
                event.prevent_default();
                timer.finish.emit(());
                return;
            }
            if *hint_open && !is_typing {
                match event.code().as_str() {
                    "ArrowLeft" => {
                        event.prevent_default();
                        on_step_hint.emit(-1);
                    }
                    "ArrowRight" => {
                        event.prevent_default();
                        on_step_hint.emit(1);
                    }
                    _ => {}
                }
            }
            if is_typing || *dialog != Dialog::None || *hint_open {
                return;
            }
            if *mode == Mode::Practice && !timer.running {
                match event.code().as_str() {
                    "KeyX" => {
                        if *recap_active {
                            on_switch_to_train.emit(());
                        } else {
                            on_start_training.emit(());
                        }
                        return;
                    }
                    "KeyH" => {
                        if event.shift_key() {
                            on_open_last_hint.emit(());
                        } else {
                            on_open_active_hint.emit(());
                        }
                        return;
                    }
                    "KeyB" => {
                        on_bookmark_last_case.emit(());
                        return;
                    }
                    "KeyR" => {
                        on_remove_last_case.emit(());
                        return;
                    }
                    "Delete" => {
                        if event.shift_key() {
                            on_clear_results.emit(());
                        } else {
                            on_remove_last_time.emit(());
                        }
                        return;
                    }
                    _ => {}
                }
            }
            if event.code() != "Space" {
                return;
            }
            event.prevent_default();
            if timer.running {
                timer.finish.emit(());
            } else {
                timer.start_hold.emit(false);
            }
        })
    };

    let on_key_up = {
        let timer = timer.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.code() == "Space" {
                timer.release.emit(false);
            }
        })
    };

    let probabilities = Rc::new(scramble::case_probabilities(
        &selected,
        &stored_results,
        &settings,
    ));
    let cube_colors = Rc::new(cube_schemes::active_colors(
        &settings,
        &bundle.trainer_id,
        &bundle.puzzle,
    ));

    let on_toggle_search_open = {
        let search_open = search_open.clone();
        Callback::from(move |_| search_open.set(!*search_open))
    };
    let on_search_query_change = {
        let search_query = search_query.clone();
        Callback::from(move |value| search_query.set(value))
    };
    let on_search_cells_change = {
        let search_cells = search_cells.clone();
        Callback::from(move |value| search_cells.set(value))
    };
    let on_search_color_change = {
        let search_color = search_color.clone();
        Callback::from(move |value| search_color.set(value))
    };
    let on_search_all_angles_change = {
        let search_all_angles = search_all_angles.clone();
        Callback::from(move |value| search_all_angles.set(value))
    };

    TrainerController {
        bundle,
        cases_by_id,
        selected: (*selected).clone(),
        selected_algsets: (*selected_algsets).clone(),
        recap_remaining_count: recap_remaining.len(),
        recap_active: *recap_active,
        mode: *mode,
        results: (*stored_results).clone(),
        dialog: *dialog,
        search_open: *search_open,
        search_query: (*search_query).clone(),
        search_cells: (*search_cells).clone(),
        search_color: (*search_color).clone(),
        search_all_angles: *search_all_angles,
        settings: (*settings).clone(),
        settings_ready,
        presets: (*presets).clone(),
        current_scramble: (*current_scramble).clone(),
        last_case: (*last_case).clone(),
        last_scramble: (*last_scramble).clone(),
        hint_open: *hint_open,
        hint_case: (*hint_case).clone(),
        stats_case_id: (*stats_case_id).clone(),
        custom_algs: (*custom_algs).clone(),
        probabilities,
        cube_colors,
        timer,
        on_key_down,
        on_key_up,
        on_open_settings,
        on_start_training,
        on_switch_to_train,
        on_back_to_selection,
        on_open_dialog,
        on_close_dialog,
        on_toggle_case,
        on_toggle_case_group,
        on_toggle_algset,
        on_use_preset,
        on_add_preset_to_selection,
        on_remove_preset_from_selection,
        on_save_preset,
        on_delete_preset,
        on_open_hint_for_case,
        on_open_active_hint,
        on_open_last_hint,
        on_close_hint,
        on_step_hint,
        on_save_alg,
        on_hide_alg,
        on_delete_alg,
        on_reorder_alg,
        on_open_stats,
        on_clear_results,
        on_delete_result,
        on_bookmark_last_case,
        on_remove_last_case,
        on_remove_last_time,
        on_update_settings,
        on_toggle_search_open,
        on_search_query_change,
        on_search_cells_change,
        on_search_color_change,
        on_search_all_angles_change,
    }
}

#[cfg(test)]
mod tests {
    use super::last_solve_deletion_message;

    #[test]
    fn last_solve_deletion_message_identifies_the_solve() {
        assert_eq!(
            last_solve_deletion_message("T Perm", "12.34"),
            "Delete the last solve?\n\nCase: T Perm\nResult time: 12.34"
        );
    }
}
