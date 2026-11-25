use crate::logic::theme::color_input_style;
use crate::services::storage::colorInputStyle;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SettingsSectionProps {
    pub title: AttrValue,
    #[prop_or_default]
    pub open: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or(AttrValue::from("trainer-settings"))]
    pub name: AttrValue,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(SettingsSection)]
pub fn settings_section(props: &SettingsSectionProps) -> Html {
    html! {
        <details
            class={classes!("settingsDetails", props.class.clone())}
            name={props.name.clone()}
            open={props.open}
        >
            <summary class="settingsSummary">{&props.title}</summary>
            { for props.children.iter() }
        </details>
    }
}

#[derive(Properties, PartialEq)]
pub struct ToggleSettingProps {
    pub label: AttrValue,
    pub checked: bool,
    pub on_change: Callback<bool>,
}

#[function_component(ToggleSetting)]
pub fn toggle_setting(props: &ToggleSettingProps) -> Html {
    let onchange = {
        let on_change = props.on_change.clone();
        Callback::from(move |event: Event| {
            on_change.emit(event.target_unchecked_into::<HtmlInputElement>().checked());
        })
    };

    html! {
        <div class="settingsEntry">
            <div>{&props.label}</div>
            <label class="switch">
                <input type="checkbox" checked={props.checked} {onchange} />
                <span class="slider round"></span>
            </label>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct StepperSettingProps {
    pub label: AttrValue,
    pub on_increment: Callback<()>,
    pub on_decrement: Callback<()>,
}

#[function_component(StepperSetting)]
pub fn stepper_setting(props: &StepperSettingProps) -> Html {
    let inc = {
        let cb = props.on_increment.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let dec = {
        let cb = props.on_decrement.clone();
        Callback::from(move |_| cb.emit(()))
    };

    html! {
        <div class="settingsEntry">
            <div>{&props.label}</div>
            <div class="plusMinus">
                <button onclick={inc} class="abutton">{"+"}</button>
                <button onclick={dec} class="abutton">{"-"}</button>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ColorInputRowProps {
    pub label: AttrValue,
    pub aria_label: AttrValue,
    pub value: AttrValue,
    pub background: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
    pub on_change: Callback<String>,
}

#[function_component(ColorInputRow)]
pub fn color_input_row(props: &ColorInputRowProps) -> Html {
    let style = color_input_style(&props.value, &props.background);
    let onchange = {
        let previous = props.value.clone();
        let background = props.background.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |event: Event| {
            let input = event.target_unchecked_into::<HtmlInputElement>();
            let new_value = input.value();
            if colorInputStyle(&new_value, &background).is_err() {
                if let Some(window) = web_sys::window() {
                    let _ = window.alert_with_message("Please enter a valid CSS color.");
                }
                input.set_value(&previous);
                return;
            }
            on_change.emit(new_value);
        })
    };

    html! {
        <label class="settingsEntry colorSetting">
            <span>{&props.label}</span>
            <input
                type="text"
                aria-label={props.aria_label.clone()}
                value={props.value.clone()}
                style={style}
                disabled={props.disabled}
                {onchange}
            />
        </label>
    }
}
