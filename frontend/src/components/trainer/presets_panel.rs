use crate::models::{Dialog, SelectionPreset};
use crate::{AlgSheetQuery, Route};
use std::collections::BTreeMap;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
struct PresetRowProps {
    name: AttrValue,
    readonly: bool,
    on_add: Callback<()>,
    on_remove: Callback<()>,
    on_delete: Callback<()>,
    on_update: Callback<()>,
    on_use: Callback<()>,
}

#[function_component(PresetRow)]
fn preset_row(props: &PresetRowProps) -> Html {
    let name = &props.name;
    let on_add = {
        let cb = props.on_add.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_remove = {
        let cb = props.on_remove.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_delete = {
        let cb = props.on_delete.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_update = {
        let cb = props.on_update.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_use = {
        let cb = props.on_use.clone();
        Callback::from(move |_| cb.emit(()))
    };

    html! {
        <div class="settingsEntry presetRow">
            <span class="presetName">{name}</span>
            <div class="plusMinus presetActions">
                <button
                    class="abutton"
                    aria-label={(!props.readonly).then(|| format!("Add {name} to selection"))}
                    onclick={on_add}
                >
                    {"+"}
                </button>
                <button
                    class="abutton"
                    disabled={props.readonly}
                    aria-label={(!props.readonly).then(|| format!("Remove {name} from selection"))}
                    onclick={on_remove}
                >
                    {"-"}
                </button>
                <button class="abutton" disabled={props.readonly} onclick={on_delete}>{"Del"}</button>
                <button class="abutton" disabled={props.readonly} onclick={on_update}>{"Set"}</button>
                <button class="abutton" onclick={on_use}>{"Use"}</button>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct PresetsPanelProps {
    pub trainer_id: String,
    pub presets: BTreeMap<String, SelectionPreset>,
    pub selected_algsets: BTreeMap<String, bool>,
    pub on_use_preset: Callback<SelectionPreset>,
    pub on_add_to_selection: Callback<SelectionPreset>,
    pub on_remove_from_selection: Callback<SelectionPreset>,
    pub on_create_preset: Callback<String>,
    pub on_update_preset: Callback<String>,
    pub on_delete_preset: Callback<String>,
    pub on_open_dialog: Callback<Dialog>,
}

#[function_component(PresetsPanel)]
pub fn presets_panel(props: &PresetsPanelProps) -> Html {
    let preset_name = use_state(String::new);

    let on_add_new = {
        let preset_name = preset_name.clone();
        let on_create = props.on_create_preset.clone();
        Callback::from(move |_| {
            let trimmed = preset_name.trim().to_owned();
            if !trimmed.is_empty() {
                on_create.emit(trimmed);
                preset_name.set(String::new());
            }
        })
    };

    let default_preset = SelectionPreset {
        sel_cases: vec!["1".to_owned(), "2".to_owned()],
        selected_alg_sets: props.selected_algsets.clone(),
    };

    html! {
        <div class="borderedContainer colFlex pad center" style="background-color: var(--backgroundDarker);">
            <div id="presetsSettings" class="colFlex" style="width:fit-content; max-width:100%;">
                if !props.presets.contains_key("Default") {
                    <PresetRow
                        name="Default"
                        readonly=true
                        on_add={{
                            let cb = props.on_add_to_selection.clone();
                            let preset = default_preset.clone();
                            Callback::from(move |_| cb.emit(preset.clone()))
                        }}
                        on_remove={Callback::noop()}
                        on_delete={Callback::noop()}
                        on_update={Callback::noop()}
                        on_use={{
                            let cb = props.on_use_preset.clone();
                            Callback::from(move |_| cb.emit(default_preset.clone()))
                        }}
                    />
                }
                { for props.presets.iter().map(|(name, preset)| {
                    let on_use = props.on_use_preset.clone();
                    let on_add = props.on_add_to_selection.clone();
                    let on_remove = props.on_remove_from_selection.clone();
                    let on_update = props.on_update_preset.clone();
                    let on_delete = props.on_delete_preset.clone();
                    let preset_add = preset.clone();
                    let preset_remove = preset.clone();
                    let preset_use = preset.clone();
                    let name_del = name.clone();
                    let name_set = name.clone();
                    html! {
                        <PresetRow
                            name={name.clone()}
                            readonly=false
                            on_add={Callback::from(move |_| on_add.emit(preset_add.clone()))}
                            on_remove={Callback::from(move |_| on_remove.emit(preset_remove.clone()))}
                            on_delete={Callback::from(move |_| on_delete.emit(name_del.clone()))}
                            on_update={Callback::from(move |_| on_update.emit(name_set.clone()))}
                            on_use={Callback::from(move |_| on_use.emit(preset_use.clone()))}
                        />
                    }
                }) }
                <div class="settingsEntry">
                    <input
                        id="newPresetInput"
                        type="text"
                        placeholder="New Preset"
                        value={(*preset_name).clone()}
                        oninput={{
                            let preset_name = preset_name.clone();
                            Callback::from(move |event: InputEvent| {
                                preset_name.set(event.target_unchecked_into::<HtmlInputElement>().value());
                            })
                        }}
                    />
                    <button class="abutton" onclick={on_add_new}>{"Add"}</button>
                </div>
            </div>
            <div style="width:100%; display:flex; justify-content:space-between; align-items:end;">
                <button class="linkButton" onclick={{
                    let open_dialog = props.on_open_dialog.clone();
                    Callback::from(move |_| open_dialog.emit(Dialog::Hotkeys))
                }}>{"Hotkeys"}</button>
                <Link<Route, AlgSheetQuery> to={Route::AlgSheet} query={Some(AlgSheetQuery { algset: props.trainer_id.clone() })}>{"Algs lookup"}</Link<Route, AlgSheetQuery>>
                <button class="linkButton" onclick={{
                    let open_dialog = props.on_open_dialog.clone();
                    Callback::from(move |_| open_dialog.emit(Dialog::Credits))
                }}>{"Credits"}</button>
            </div>
            <span class="version">{crate::APP_VERSION}</span>
        </div>
    }
}
