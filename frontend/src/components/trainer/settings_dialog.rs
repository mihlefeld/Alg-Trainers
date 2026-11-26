use crate::components::account_settings::AccountSettings;
use crate::components::cube_settings::CubeSettings;
use crate::components::modal::ModalDialog;
use crate::components::settings_controls::{
    ColorInputRow, SettingsSection, StepperSetting, ToggleSetting,
};
use crate::logic::theme::{
    adjust_size_setting, default_theme_colors, setting_color, update_scramble_override,
};
use crate::services::storage::{export_backup, import_backup_file, read_file_text};
use std::collections::BTreeMap;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SettingsDialogProps {
    pub settings: serde_json::Value,
    pub settings_ready: bool,
    pub selection_key: String,
    pub on_update_settings: Callback<serde_json::Value>,
    pub on_close: Callback<()>,
}

const BEHAVIOR_TOGGLES: [(&str, &str); 5] = [
    ("enableHoldDelay", "Hold down delay"),
    ("weightedChoice", "Weighted choice (# of Solves)"),
    ("weightedChoiceTime", "Weighted choice (Mean time)"),
    ("weightedChoiceStd", "Weighted choice (Standard deviation)"),
    ("showProbabilities", "Show case probabilities"),
];

const SCRAMBLE_OPTIONS: [(&str, &str); 4] = [
    ("preRotations", "Pre-rotations"),
    ("postRotations", "Post-rotations"),
    ("preMoves", "Pre-moves"),
    ("postMoves", "Post-moves"),
];

const SIZE_SETTINGS: [(&str, &str); 3] = [
    ("baseSize", "Base size"),
    ("timerSize", "Timer size"),
    ("scrambleSize", "Scramble size"),
];

const THEME_COLORS: [(&str, &str); 5] = [
    ("--background", "Background"),
    ("--text", "Text"),
    ("--primary", "Primary"),
    ("--secondary", "Secondary"),
    ("--accent", "Accent"),
];

const BLD_SCHEMES: [(&str, &str); 2] = [
    ("letterSchemeCorners", "Corner letter scheme"),
    ("letterSchemeEdges", "Edge letter scheme"),
];

const BLD_TOGGLES: [(&str, &str); 2] = [
    ("useCornersLetterPairs", "Corner letter-pair words"),
    ("useEdgesLetterPairs", "Edge letter-pair words"),
];

#[function_component(SettingsDialog)]
pub fn settings_dialog(props: &SettingsDialogProps) -> Html {
    let settings = &props.settings;
    let on_update = &props.on_update_settings;

    let on_export_backup = Callback::from(|_| {
        spawn_local(async {
            export_backup().await;
        });
    });

    let on_import_backup = Callback::from(|event: Event| {
        let input = event.target_unchecked_into::<HtmlInputElement>();
        let Some(file) = input.files().and_then(|files| files.get(0)) else {
            return;
        };
        spawn_local(async move {
            let result = import_backup_file(&file).await;
            if let Some(window) = web_sys::window() {
                let message = match result {
                    Ok(()) => "Backup imported. Reload the page to load the restored data.",
                    Err(message) => message,
                };
                let _ = window.alert_with_message(message);
            }
        });
    });

    let on_import_letter_pairs = {
        let settings = settings.clone();
        let on_update = on_update.clone();
        Callback::from(move |event: Event| {
            let input = event.target_unchecked_into::<HtmlInputElement>();
            let Some(file) = input.files().and_then(|files| files.get(0)) else {
                return;
            };
            let settings = settings.clone();
            let on_update = on_update.clone();
            spawn_local(async move {
                let result = async {
                    let text = read_file_text(&file, "Unable to read letter-pair file").await?;
                    let pairs = serde_json::from_str::<BTreeMap<String, String>>(&text)
                        .map_err(|_| "Invalid letter-pair JSON")?;
                    let mut next = settings.clone();
                    next["letterpairs"] = serde_json::to_value(pairs).unwrap();
                    on_update.emit(next);
                    Ok::<(), &str>(())
                }
                .await;
                if let Some(window) = web_sys::window() {
                    let message = match result {
                        Ok(()) => "Letter-pair words imported.",
                        Err(message) => message,
                    };
                    let _ = window.alert_with_message(message);
                }
            });
        })
    };

    let background = setting_color(settings, "--background");

    html! {
        <ModalDialog id="settings" class="colFlex borderedContainer" on_close={props.on_close.clone()}>
            <AccountSettings />

            <SettingsSection title="Behavior">
                <div class="colFlex">
                    {for BEHAVIOR_TOGGLES.into_iter().map(|(key, label)| {
                        let checked = settings
                            .get(key)
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(key == "enableHoldDelay" || key == "weightedChoice");
                        let next = settings.clone();
                        let on_update = on_update.clone();
                        html! {
                            <ToggleSetting
                                {label}
                                {checked}
                                on_change={Callback::from(move |value: bool| {
                                    let mut next = next.clone();
                                    next[key] = value.into();
                                    on_update.emit(next);
                                })}
                            />
                        }
                    })}
                </div>
            </SettingsSection>


            <SettingsSection title="Scramble options">
                <div class="colFlex">
                    <div class="trainerScrambleHelp">{"Enter one option per line. Leave blank to use the trainer defaults."}</div>
                    {for SCRAMBLE_OPTIONS.into_iter().map(|(key, label)| {
                        let value = settings
                            .get("trainerScrambleOverrides")
                            .and_then(|value| value.get(&props.selection_key))
                            .and_then(|value| value.get(key))
                            .and_then(serde_json::Value::as_array)
                            .map(|items| {
                                items
                                    .iter()
                                    .filter_map(serde_json::Value::as_str)
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            })
                            .unwrap_or_default();
                        let next = settings.clone();
                        let selection_key = props.selection_key.clone();
                        let on_update = on_update.clone();
                        html! {
                            <div class="settingsEntry trainerScrambleEntry">
                                <label>{label}</label>
                                <textarea
                                    rows="2"
                                    placeholder="Trainer default"
                                    value={value}
                                    onchange={Callback::from(move |event: Event| {
                                        let raw = event.target_unchecked_into::<HtmlTextAreaElement>().value();
                                        on_update.emit(update_scramble_override(&next, &selection_key, key, &raw));
                                    })}
                                />
                            </div>
                        }
                    })}
                </div>
            </SettingsSection>

            <SettingsSection title="Styling">
                <div class="colFlex">
                    {for SIZE_SETTINGS.into_iter().map(|(key, label)| {
                        let next_inc = settings.clone();
                        let next_dec = settings.clone();
                        let on_inc = on_update.clone();
                        let on_dec = on_update.clone();
                        html! {
                            <StepperSetting
                                {label}
                                on_increment={Callback::from(move |_| {
                                    on_inc.emit(adjust_size_setting(&next_inc, key, 0.1));
                                })}
                                on_decrement={Callback::from(move |_| {
                                    on_dec.emit(adjust_size_setting(&next_dec, key, -0.1));
                                })}
                            />
                        }
                    })}
                    <div class="settingsEntry">
                        <div>{"Theme"}</div>
                        <div class="plusMinus">
                            <button
                                class="material-symbols-outlined abutton"
                                title="Light theme"
                                onclick={{
                                    let next = settings.clone();
                                    let on_update = on_update.clone();
                                    Callback::from(move |_| {
                                        let mut next = next.clone();
                                        next["colors"] = serde_json::to_value(default_theme_colors("light")).unwrap();
                                        on_update.emit(next);
                                    })
                                }}
                            >
                                {"light_mode"}
                            </button>
                            <button
                                class="material-symbols-outlined abutton"
                                title="Dark theme"
                                onclick={{
                                    let next = settings.clone();
                                    let on_update = on_update.clone();
                                    Callback::from(move |_| {
                                        let mut next = next.clone();
                                        next["colors"] = serde_json::to_value(default_theme_colors("dark")).unwrap();
                                        on_update.emit(next);
                                    })
                                }}
                            >
                                {"dark_mode"}
                            </button>
                        </div>
                    </div>
                    {for THEME_COLORS.into_iter().map(|(key, label)| {
                        let value = setting_color(settings, key);
                        let next = settings.clone();
                        let on_update = on_update.clone();
                        html! {
                            <ColorInputRow
                                {label}
                                aria_label={format!("{label} color")}
                                {value}
                                background={background.clone()}
                                on_change={Callback::from(move |new_value: String| {
                                    let mut next = next.clone();
                                    if next.get("colors").and_then(serde_json::Value::as_object).is_none() {
                                        next["colors"] = serde_json::json!({});
                                    }
                                    next["colors"][key] = new_value.into();
                                    on_update.emit(next);
                                })}
                            />
                        }
                    })}
                </div>
            </SettingsSection>

            <SettingsSection title="Cube colors">
                if props.settings_ready {
                    <CubeSettings settings={settings.clone()} on_change={on_update.clone()} />
                } else {
                    <div>{"Loading settings..."}</div>
                }
            </SettingsSection>

            <SettingsSection title="Blind trainer behavior">
                <div class="colFlex">
                    {for BLD_SCHEMES.into_iter().map(|(key, label)| {
                        let value = settings
                            .get(key)
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("abcdefghijklmnopqrstuvwx")
                            .to_owned();
                        let next = settings.clone();
                        let on_update = on_update.clone();
                        html! {
                            <div class="settingsEntry">
                                <div>{label}</div>
                                <input
                                    type="text"
                                    value={value}
                                    onchange={Callback::from(move |event: Event| {
                                        let mut next = next.clone();
                                        next[key] = event.target_unchecked_into::<HtmlInputElement>().value().into();
                                        on_update.emit(next);
                                    })}
                                />
                            </div>
                        }
                    })}
                    {for BLD_TOGGLES.into_iter().map(|(key, label)| {
                        let checked = settings
                            .get(key)
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false);
                        let next = settings.clone();
                        let on_update = on_update.clone();
                        html! {
                            <ToggleSetting
                                {label}
                                {checked}
                                on_change={Callback::from(move |value: bool| {
                                    let mut next = next.clone();
                                    next[key] = value.into();
                                    on_update.emit(next);
                                })}
                            />
                        }
                    })}
                    <div class="settingsEntry">
                        <div>{"Letter-pair words"}</div>
                        <label
                            for="letterPairUpload"
                            class="material-symbols-outlined abutton compactButton"
                            title="Import letter-pair JSON"
                        >
                            {"upload"}
                        </label>
                        <input
                            id="letterPairUpload"
                            type="file"
                            accept="application/json,.json"
                            style="display:none;"
                            onchange={on_import_letter_pairs}
                        />
                    </div>
                </div>
            </SettingsSection>

            <SettingsSection title="Import / Export">
                <div class="colFlex">
                    <div class="settingsEntry">
                        <div>{"Backup settings and trainer data"}</div>
                        <div class="plusMinus">
                            <label
                                for="trainerBackupUpload"
                                class="material-symbols-outlined abutton compactButton"
                                title="Import backup"
                            >
                                {"upload"}
                            </label>
                            <input
                                id="trainerBackupUpload"
                                type="file"
                                accept="application/json,.json"
                                style="display:none;"
                                onchange={on_import_backup}
                            />
                            <button
                                class="material-symbols-outlined abutton compactButton"
                                title="Export backup"
                                onclick={on_export_backup}
                            >
                                {"download"}
                            </button>
                        </div>
                    </div>
                </div>
            </SettingsSection>
            <div class="settingsSidebar"> </div>
        </ModalDialog>
    }
}
