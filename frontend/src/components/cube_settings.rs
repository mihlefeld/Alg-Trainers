use crate::components::settings_controls::ColorInputRow;
use crate::logic::cube_schemes;
use crate::logic::theme::{default_cube_colors, setting_color};
use crate::models::TrainerSummary;
use crate::services::api::fetch_trainers;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub settings: Value,
    pub on_change: Callback<Value>,
}

fn options(settings: &Value, fallback: Option<&str>, selected: &str) -> Vec<Html> {
    let mut result = Vec::new();
    if let Some(label) = fallback {
        result.push(
            html! { <option value="" selected={selected.is_empty()}>{label.to_owned()}</option> },
        );
    }
    for (id, name) in cube_schemes::PRESETS {
        result.push(html! { <option value={id} selected={id == selected}>{name}</option> });
    }
    if let Some(schemes) = settings.get("cubeColorSchemes").and_then(Value::as_array) {
        for scheme in schemes {
            if let (Some(id), Some(name)) = (
                scheme.get("id").and_then(Value::as_str),
                scheme.get("name").and_then(Value::as_str),
            ) {
                result.push(html! { <option value={id.to_owned()} selected={id == selected}>{name.to_owned()}</option> });
            }
        }
    }
    result
}

fn notify(message: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.alert_with_message(message);
    }
}

#[function_component(CubeSettings)]
pub fn cube_settings(props: &Props) -> Html {
    let editing = use_state(|| "default".to_owned());
    let catalog = use_state(|| None::<Result<Vec<TrainerSummary>, String>>);
    let retry = use_state(|| 0_u32);
    {
        let catalog = catalog.clone();
        use_effect_with(*retry, move |_| {
            catalog.set(None);
            spawn_local(async move {
                catalog.set(Some(fetch_trainers().await));
            });
            || ()
        });
    }
    {
        let on_change = props.on_change.clone();
        use_effect_with(
            ((*catalog).clone(), props.settings.clone()),
            move |(catalog, settings)| {
                if let Some(Ok(trainers)) = catalog {
                    let mut next = settings.clone();
                    let mut changed = false;
                    if let Some(overrides) = next
                        .get_mut("puzzleCubeSchemes")
                        .and_then(Value::as_object_mut)
                    {
                        for (puzzle, entry) in overrides {
                            let members = trainers
                                .iter()
                                .filter(|trainer| &trainer.puzzle == puzzle)
                                .map(|trainer| trainer.trainer_id.clone())
                                .collect::<Vec<_>>();
                            if !members.is_empty() && entry.get("trainers") != Some(&json!(members))
                            {
                                entry["trainers"] = json!(members);
                                changed = true;
                            }
                        }
                    }
                    if changed {
                        on_change.emit(next);
                    }
                }
                || ()
            },
        );
    }
    let settings = &props.settings;
    let selected_id = (*editing).clone();
    let custom = settings
        .get("cubeColorSchemes")
        .and_then(Value::as_array)
        .and_then(|schemes| {
            schemes
                .iter()
                .find(|scheme| scheme.get("id").and_then(Value::as_str) == Some(&selected_id))
        });
    let name = custom
        .and_then(|scheme| scheme.get("name"))
        .and_then(Value::as_str)
        .or_else(|| {
            cube_schemes::PRESETS
                .iter()
                .find(|(id, _)| *id == selected_id)
                .map(|(_, name)| *name)
        })
        .unwrap_or("Unknown scheme")
        .to_owned();
    let is_custom = custom.is_some();
    let colors = cube_schemes::scheme_colors(settings, &selected_id);
    let background = setting_color(settings, "--background");
    let mut groups = BTreeMap::<String, Vec<TrainerSummary>>::new();
    if let Some(Ok(trainers)) = catalog.as_ref() {
        for trainer in trainers {
            groups
                .entry(trainer.puzzle.clone())
                .or_default()
                .push(trainer.clone());
        }
    }
    let global = settings
        .get("defaultCubeScheme")
        .and_then(Value::as_str)
        .unwrap_or("default")
        .to_owned();

    html! {
        <div class="colFlex">
            <div class="trainerScrambleHelp">{"Choose a global scheme and optional puzzle or trainer overrides. Changes apply immediately. Schemes are included in backups."}</div>
            <label class="settingsEntry cubeSchemeEntry">
                <span>{"Global scheme"}</span>
                <select aria-label="Global cube scheme" value={global.clone()} onchange={{
                    let next = settings.clone();
                    let on_change = props.on_change.clone();
                    Callback::from(move |event: Event| {
                        let mut next = next.clone();
                        next["defaultCubeScheme"] = json!(event.target_unchecked_into::<HtmlSelectElement>().value());
                        on_change.emit(next);
                    })
                }}>{for options(settings, None, &global)}</select>
            </label>
            <details class="cubeTrainerGroup" name="puzzle-scheme-sections">
                <summary class="cubePuzzleSummary">
                    <b>{"Manage scheme"}</b>
                    <span>{name.clone()}</span>
                </summary>
                <label class="settingsEntry cubeSchemeEntry">
                    <span>{"Manage scheme"}</span>
                    <select aria-label="Manage cube scheme" value={selected_id.clone()} onchange={{
                        let editing = editing.clone();
                        Callback::from(move |event: Event| editing.set(event.target_unchecked_into::<HtmlSelectElement>().value()))
                    }}>{for options(settings, None, &selected_id)}</select>
                </label>
                <label class="settingsEntry cubeSchemeEntry">
                    <span>{"Name"}</span>
                    <input type="text" aria-label="Scheme name" value={name.clone()} disabled={!is_custom} onchange={{
                        let next = settings.clone();
                        let id = selected_id.clone();
                        let name = name.clone();
                        let on_change = props.on_change.clone();
                        Callback::from(move |event: Event| {
                            let mut next = next.clone();
                            let input = event.target_unchecked_into::<HtmlInputElement>();
                            let value = input.value();
                            if value.trim().is_empty() {
                                notify("Please enter a scheme name.");
                                input.set_value(&name);
                                return;
                            }
                            if let Some(scheme) = next["cubeColorSchemes"].as_array_mut()
                                .and_then(|schemes| schemes.iter_mut().find(|s| s["id"].as_str() == Some(&id))) {
                                scheme["name"] = json!(value.trim());
                                on_change.emit(next);
                            }
                        })
                    }} />
                </label>
                <div class="cubeSchemeActions">
                    <button class="abutton" onclick={{
                        let next = settings.clone();
                        let colors = colors.clone();
                        let editing = editing.clone();
                        let on_change = props.on_change.clone();
                        Callback::from(move |_| {
                            let mut next = next.clone();
                            let Some(schemes) = next["cubeColorSchemes"].as_array_mut() else { return };
                            let id = loop {
                                let id = format!("custom-{}-{}", js_sys::Date::now() as u64, (js_sys::Math::random() * 1_000_000_000.0) as u64);
                                if !schemes.iter().any(|scheme| scheme["id"].as_str() == Some(&id)) { break id; }
                            };
                            schemes.push(json!({"id": id, "name": format!("Custom scheme {}", schemes.len() + 1), "colors": colors}));
                            editing.set(id);
                            on_change.emit(next);
                        })
                    }}>{"Add copy"}</button>
                    <button class="abutton" disabled={!is_custom} onclick={{
                        let next = settings.clone();
                        let id = selected_id.clone();
                        let name = name.clone();
                        let editing = editing.clone();
                        let on_change = props.on_change.clone();
                        Callback::from(move |_| {
                            let mut next = next.clone();
                            let confirmed = web_sys::window().and_then(|w| w.confirm_with_message(&format!("Delete \"{name}\"? Overrides using it will fall back to their inherited scheme.")).ok()).unwrap_or(false);
                            if confirmed {
                                cube_schemes::delete_scheme(&mut next, &id);
                                editing.set("default".to_owned());
                                on_change.emit(next);
                            }
                        })
                    }}>{"Delete"}</button>
                </div>
                {for default_cube_colors().keys().map(|key| {
                    let key = *key;
                    let value = colors.get(key).cloned().unwrap_or_default();
                    let next = settings.clone();
                    let id = selected_id.clone();
                    let on_change = props.on_change.clone();
                    html! {
                        <ColorInputRow
                            label={key}
                            aria_label={format!("{key} cube color")}
                            value={value}
                            background={background.clone()}
                            disabled={!is_custom}
                            on_change={Callback::from(move |new_value: String| {
                                let mut next = next.clone();
                                if let Some(scheme) = next["cubeColorSchemes"].as_array_mut()
                                    .and_then(|schemes| schemes.iter_mut().find(|s| s["id"].as_str() == Some(&id))) {
                                    scheme["colors"][key] = json!(new_value);
                                    on_change.emit(next);
                                }
                            })}
                        />
                    }
                })}
            </details>
            <div class="cubeTrainerList">
                if let Some(Err(error)) = catalog.as_ref() {
                    <div role="alert">{error}</div>
                    <button class="abutton" onclick={{
                        let retry = retry.clone();
                        Callback::from(move |_| retry.set(*retry + 1))
                    }}>{"Retry"}</button>
                } else if catalog.is_none() {
                    <div>{"Loading trainers..."}</div>
                }
                {for groups.into_iter().map(|(puzzle, trainers)| {
                    let puzzle_value = settings.get("puzzleCubeSchemes").and_then(|v| v.get(&puzzle))
                        .and_then(|v| v.get("scheme")).and_then(Value::as_str).unwrap_or("").to_owned();
                    let next = settings.clone();
                    let puzzle_key = puzzle.clone();
                    let puzzle_scheme_label = cube_schemes::scheme_label(settings, &puzzle_value);
                    let trainer_ids = trainers.iter().map(|t| t.trainer_id.clone()).collect::<Vec<_>>();
                    let on_change = props.on_change.clone();
                    html! {
                        <details class="cubeTrainerGroup" name="puzzle-scheme-sections">
                            <summary class="cubePuzzleSummary">
                                <b>{puzzle.clone()}</b>
                                <span>{puzzle_scheme_label}</span>
                            </summary>
                            <label class="settingsEntry cubeTrainerEntry cubePuzzleEntry">
                                <select aria-label={format!("{puzzle} puzzle scheme")} value={puzzle_value.clone()} onchange={Callback::from(move |event: Event| {
                                    let mut next = next.clone();
                                    let value = event.target_unchecked_into::<HtmlSelectElement>().value();
                                    if value.is_empty() {
                                        if let Some(overrides) = next["puzzleCubeSchemes"].as_object_mut() { overrides.remove(&puzzle_key); }
                                    } else {
                                        next["puzzleCubeSchemes"][&puzzle_key] = json!({"scheme": value, "trainers": trainer_ids});
                                    }
                                    on_change.emit(next);
                                })}>{for options(settings, Some("Use global scheme"), &puzzle_value)}</select>
                            </label>
                            {for trainers.into_iter().map(|trainer| {
                                let value = settings.get("trainerCubeSchemes").and_then(|v| v.get(&trainer.trainer_id))
                                    .and_then(Value::as_str).unwrap_or("").to_owned();
                                let next = settings.clone();
                                let id = trainer.trainer_id.clone();
                                let on_change = props.on_change.clone();
                                html! {
                                    <label class="settingsEntry cubeTrainerEntry">
                                        <span>{trainer.name.clone()}</span>
                                        <select aria-label={format!("{} {} scheme", puzzle, trainer.name)} value={value.clone()} onchange={Callback::from(move |event: Event| {
                                            let mut next = next.clone();
                                            let value = event.target_unchecked_into::<HtmlSelectElement>().value();
                                            if value.is_empty() {
                                                if let Some(overrides) = next["trainerCubeSchemes"].as_object_mut() { overrides.remove(&id); }
                                            } else {
                                                next["trainerCubeSchemes"][&id] = json!(value);
                                            }
                                            on_change.emit(next);
                                        })}>{for options(settings, Some("Use puzzle scheme"), &value)}</select>
                                    </label>
                                }
                            })}
                        </details>
                    }
                })}
            </div>
        </div>
    }
}
