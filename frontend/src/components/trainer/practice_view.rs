use crate::components::trainer::timer::TrainerTimer;
use crate::logic::results::format_time;
use crate::models::{SelectionPreset, StoredResult, TrainerCase};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use web_sys::PointerEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PracticeViewProps {
    pub selected_count: usize,
    pub recap_active: bool,
    pub recap_remaining_count: usize,
    pub current_scramble: String,
    pub last_scramble: String,
    pub last_case: Option<TrainerCase>,
    pub presets: BTreeMap<String, SelectionPreset>,
    pub results: Vec<StoredResult>,
    pub cases_by_id: Rc<BTreeMap<String, TrainerCase>>,
    pub settings: serde_json::Value,
    pub timer: TrainerTimer,
    pub on_back_to_selection: Callback<()>,
    pub on_start_recap: Callback<()>,
    pub on_switch_to_train: Callback<()>,
    pub on_open_settings: Callback<()>,
    pub on_open_active_hint: Callback<()>,
    pub on_open_last_hint: Callback<()>,
    pub on_open_case_hint: Callback<TrainerCase>,
    pub on_open_stats: Callback<String>,
    pub on_clear_results: Callback<()>,
    pub on_bookmark_last_case: Callback<()>,
    pub on_remove_last_case: Callback<()>,
    pub on_remove_last_time: Callback<()>,
}

#[function_component(PracticeView)]
pub fn practice_view(props: &PracticeViewProps) -> Html {
    let scramble_size = props
        .settings
        .get("scrambleSize")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(2.0);
    let timer_size = props
        .settings
        .get("timerSize")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(5.0);
    let solved_case_count = props
        .results
        .iter()
        .map(|result| &result.case)
        .collect::<BTreeSet<_>>()
        .len();
    let is_last_bookmarked = props
        .last_case
        .as_ref()
        .and_then(|case| {
            props
                .presets
                .get("Bookmarks")
                .map(|preset| preset.sel_cases.contains(&case.case_id))
        })
        .unwrap_or(false);

    let on_back = {
        let cb = props.on_back_to_selection.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_start_recap = {
        let cb = props.on_start_recap.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_switch_to_train = {
        let cb = props.on_switch_to_train.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_open_settings = {
        let cb = props.on_open_settings.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_open_active_hint = {
        let cb = props.on_open_active_hint.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_open_last_hint = {
        let cb = props.on_open_last_hint.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_clear_results = {
        let cb = props.on_clear_results.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_bookmark = {
        let cb = props.on_bookmark_last_case.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_remove_case = {
        let cb = props.on_remove_last_case.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_remove_time = {
        let cb = props.on_remove_last_time.clone();
        Callback::from(move |_| cb.emit(()))
    };
    html! {
        <div id="practiceLayout" class="colFlex" style="display:flex;">
            <div id="casesBox" class="borderedContainer pad rowFlex center">
                <button class="material-symbols-outlined abutton mbutton" onclick={on_back}>
                    {"arrow_back"}
                </button>
                <span id="selInfo" style="font-size:2em;">
                    <b>
                        if !props.recap_active {
                            <a onclick={on_start_recap}>{"Train"}</a>
                            {format!(" {} Cases", props.selected_count)}
                        } else {
                            <a onclick={on_switch_to_train}>{"Recap"}</a>
                            {format!(" {} Cases", props.recap_remaining_count + 1)}
                        }
                    </b>
                </span>
                <button
                    class="material-symbols-outlined abutton"
                    title="Settings"
                    aria-label="Settings"
                    onclick={on_open_settings}
                >
                    {"settings"}
                </button>
            </div>

            <div class="borderedContainer pad center rowFlex practiceScramble" style="justify-content:space-between;">
                <div
                    id="scramble"
                    onclick={on_open_active_hint}
                    style={format!("font-size:{scramble_size}em;")}
                >
                    {if props.current_scramble.is_empty() {
                        "Select at least one case".to_owned()
                    } else {
                        props.current_scramble.clone()
                    }}
                    <span class="inlineButton" style="font-size:1em !important;">{"?"}</span>
                </div>
            </div>

            <div id="timer_and_stats" class="rowFlex grow">
                <div
                    id="timerDiv"
                    class="borderedContainer"
                    onpointerdown={props.timer.on_pointer_down.clone()}
                    onpointerup={props.timer.on_pointer_up.clone()}
                    onpointermove={Callback::from(|event: PointerEvent| event.prevent_default())}
                    onpointercancel={props.timer.on_pointer_cancel.clone()}
                >
                    <span
                        id="timer"
                        style={format!(
                            "text-align:center; color:{}; font-size:{timer_size}em;",
                            props.timer.color()
                        )}
                    >
                        {props.timer.display_text()}
                    </span>
                </div>

                <div id="stats" class="borderedContainer colFlex pad">
                    <div class="timeEntry rowFlex" style="justify-content:space-between;">
                        <div class="colFlex">
                            <span>{format!("Times {}", props.results.len())}</span>
                            <span>{format!("Cases {solved_case_count}")}</span>
                        </div>
                        <button class="material-symbols-outlined inlineButton" onclick={on_clear_results}>
                            {"delete"}
                        </button>
                    </div>
                    <div id="times" class="colFlex">
                        { for props.cases_by_id.values().filter_map(|case| {
                            let case_results = props.results.iter()
                                .filter(|result| result.case == case.case_id)
                                .collect::<Vec<_>>();
                            if case_results.is_empty() {
                                return None;
                            }
                            let mean = case_results.iter().map(|result| result.ms as f64).sum::<f64>()
                                / case_results.len() as f64;
                            let on_open_case_hint = props.on_open_case_hint.clone();
                            let hint_case = case.clone();
                            let on_open_stats = props.on_open_stats.clone();
                            let case_id = case.case_id.clone();
                            Some(html! {
                                <div class="timeEntry">
                                    <button
                                        class="caseNameStats"
                                        onclick={Callback::from(move |_| on_open_case_hint.emit(hint_case.clone()))}
                                    >
                                        {&case.name}
                                    </button>
                                    <button
                                        class="caseNameStats"
                                        title={format!("View {} statistics", &case.name)}
                                        onclick={Callback::from(move |_| on_open_stats.emit(case_id.clone()))}
                                    >
                                        {format!("(#{}; avg {})", case_results.len(), format_time(mean.round() as i64))}
                                    </button>
                                </div>
                            })
                        }) }
                    </div>
                </div>
            </div>

            <div class="borderedContainer pad rowFlex center practiceFooter" style="justify-content:space-between;flex-direction:row-reverse;">
                <div id="last_scramble" class="rowFlex" style="width:100%;">
                    <span class="lastScrambleText">
                        {"Last scramble: "}
                        if props.last_scramble.is_empty() { {"None yet"} } else { {&props.last_scramble} }
                    </span>
                    if let Some(case) = props.last_case.as_ref() {
                        <button class="caseNameStats" onclick={on_open_last_hint}>
                            {format!("({})", case.name)}
                        </button>
                    }
                    <span id="last-scramble-buttons">
                        <button
                            class="material-symbols-outlined inlineButton"
                            title="Bookmark last case"
                            aria-label="Bookmark last case"
                            onclick={on_bookmark}
                        >
                            {if is_last_bookmarked { "bookmark_remove" } else { "bookmark_add" }}
                        </button>
                        <button
                            class="material-symbols-outlined inlineButton"
                            title="Remove last case from selection"
                            aria-label="Remove last case from selection"
                            onclick={on_remove_case}
                        >
                            {"close"}
                        </button>
                        <button
                            class="material-symbols-outlined inlineButton"
                            title="Delete last time"
                            aria-label="Delete last time"
                            onclick={on_remove_time}
                        >
                            {"undo"}
                        </button>
                    </span>
                </div>
            </div>
        </div>
    }
}
