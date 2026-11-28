use crate::components::case_image::CaseImage;
use crate::components::trainer::presets_panel::PresetsPanel;
use crate::components::trainer::search_panel::SearchPanel;
use crate::logic::search::case_matches_search;
use crate::logic::selection::compute_selection_metrics;
use crate::models::{Dialog, SavedSearchQuery, SelectionPreset, TrainerBundle, TrainerCase};
use crate::Route;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use web_sys::MouseEvent;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
struct CaseCardProps {
    case: TrainerCase,
    selected: bool,
    include_name: bool,
    cube_colors: Rc<BTreeMap<String, String>>,
    probability: Option<f64>,
    on_toggle: Callback<String>,
    on_open_hint: Callback<TrainerCase>,
}

#[function_component(CaseCard)]
fn case_card(props: &CaseCardProps) -> Html {
    let case = &props.case;

    let onclick = {
        let on_toggle = props.on_toggle.clone();
        let case_id = case.case_id.clone();
        Callback::from(move |_| on_toggle.emit(case_id.clone()))
    };

    let open_hint_mouse = {
        let on_open_hint = props.on_open_hint.clone();
        let case = case.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            on_open_hint.emit(case.clone());
        })
    };

    html! {
        <div
            id={format!("itemTd{}", case.case_id)}
            class={classes!("borderedContainer", "caseItem", if props.selected { "itemSel" } else { "itemUnsel" })}
            {onclick}
            oncontextmenu={open_hint_mouse}
        >
            if props.include_name {
                <span class="caseSpan">{&case.name}</span>
            }
            if let Some(svg) = case.image_svg.as_ref() {
                <CaseImage svg={svg.clone()} colors={props.cube_colors.clone()} class="caseImage" />
            } else {
                <img class="caseImage" src="" />
            }
            if let Some(prob) = props.probability {
                <span class="caseProbability">{format!("{:.1}%", prob * 100.0)}</span>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SelectionViewProps {
    pub bundle: Rc<TrainerBundle>,
    pub cases_by_id: Rc<BTreeMap<String, TrainerCase>>,
    pub selected: Vec<String>,
    pub selected_algsets: BTreeMap<String, bool>,
    pub presets: BTreeMap<String, SelectionPreset>,
    pub settings: serde_json::Value,
    pub probabilities: Rc<BTreeMap<String, f64>>,
    pub cube_colors: Rc<BTreeMap<String, String>>,
    pub search_open: bool,
    pub search_query: String,
    pub search_cells: BTreeMap<usize, String>,
    pub search_color: String,
    pub search_all_angles: bool,
    pub saved_search_queries: BTreeMap<String, SavedSearchQuery>,
    pub active_search_query: String,
    pub default_search_query: Option<String>,
    pub on_open_settings: Callback<()>,
    pub on_start_training: Callback<()>,
    pub on_open_dialog: Callback<Dialog>,
    pub on_toggle_case: Callback<String>,
    pub on_toggle_case_group: Callback<Vec<String>>,
    pub on_toggle_algset: Callback<(String, Vec<String>)>,
    pub on_use_preset: Callback<SelectionPreset>,
    pub on_add_preset_to_selection: Callback<SelectionPreset>,
    pub on_remove_preset_from_selection: Callback<SelectionPreset>,
    pub on_create_preset: Callback<String>,
    pub on_update_preset: Callback<String>,
    pub on_delete_preset: Callback<String>,
    pub on_open_hint: Callback<TrainerCase>,
    pub on_toggle_search_open: Callback<()>,
    pub on_search_query_change: Callback<String>,
    pub on_search_cells_change: Callback<BTreeMap<usize, String>>,
    pub on_search_color_change: Callback<String>,
    pub on_search_all_angles_change: Callback<bool>,
    pub on_save_search_query: Callback<(String, bool)>,
    pub on_use_search_query: Callback<String>,
    pub on_set_default_search_query: Callback<(String, bool)>,
    pub on_delete_search_query: Callback<String>,
}

#[function_component(SelectionView)]
pub fn selection_view(props: &SelectionViewProps) -> Html {
    let bundle = &props.bundle;
    let search_needle = props.search_query.trim().to_lowercase();
    let metrics = compute_selection_metrics(
        bundle,
        &props.selected,
        &props.selected_algsets,
        &search_needle,
        &props.search_cells,
        props.search_all_angles,
    );
    let show_probabilities = props
        .settings
        .get("showProbabilities")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let include_name = bundle.title.contains("PLL")
        || bundle.title.contains("BLD")
        || bundle.title.contains("Square-1");
    let compact_group_layout = props
        .settings
        .get("compactCaseGroupLayout")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    let on_open_settings = {
        let cb = props.on_open_settings.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_start_training = {
        let cb = props.on_start_training.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_toggle_search = {
        let cb = props.on_toggle_search_open.clone();
        Callback::from(move |_| cb.emit(()))
    };

    html! {
        <div
            id="selectionLayout"
            class="colFlex scroller"
            style={format!(
                "--selection-columns:{}; margin: 0.5em; width: fit-content; display:flex; min-width: 50vw;",
                metrics.selection_columns
            )}
        >
            <div class="colFlex">
                <div class="rowFlex center" style="justify-content: space-between;">
                    <h1 id="trainerTitle">{&bundle.title}<span>{" | "}</span><Link<Route> to={Route::Home}>{"Back"}</Link<Route>></h1>
                    <div class="colFlex">
                        <div id="settingsButton">
                            <button
                                onclick={on_open_settings}
                                title="Settings"
                                aria-label="Settings"
                                class="material-symbols-outlined abutton largeButton"
                            >
                                {"settings"}
                            </button>
                        </div>
                        <div id="modeButtons" class="colFlex">
                            <button
                                onclick={on_start_training}
                                class="material-symbols-outlined abutton largeButton startPracticeButton"
                                disabled={props.selected.is_empty()}
                                title="Start recap"
                                aria-label="Start recap"
                            >
                                {"send"}
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <PresetsPanel
                trainer_id={bundle.trainer_id.clone()}
                presets={props.presets.clone()}
                selected_algsets={props.selected_algsets.clone()}
                on_use_preset={props.on_use_preset.clone()}
                on_add_to_selection={props.on_add_preset_to_selection.clone()}
                on_remove_from_selection={props.on_remove_preset_from_selection.clone()}
                on_create_preset={props.on_create_preset.clone()}
                on_update_preset={props.on_update_preset.clone()}
                on_delete_preset={props.on_delete_preset.clone()}
                on_open_dialog={props.on_open_dialog.clone()}
            />

            <div id="cases_selection" class="colFlex" style="width: 100%;">
                if bundle.algsets.len() > 1 {
                    <div class="rowFlex algsetSelectors" style="flex-wrap:wrap;">
                        { for bundle.algsets.iter().map(|algset| {
                            let on_toggle_algset = props.on_toggle_algset.clone();
                            let on_toggle_case_group = props.on_toggle_case_group.clone();
                            let search_active = metrics.search_active;
                            let case_ids = bundle.groups.iter()
                                .filter(|group| algset.groups.contains(&group.name))
                                .flat_map(|group| group.cases.clone())
                                .filter(|id| props.cases_by_id.contains_key(id))
                                .filter(|id| !metrics.search_active || metrics.visible_case_ids.contains(id))
                                .collect::<BTreeSet<_>>()
                                .into_iter()
                                .collect::<Vec<_>>();
                            let enabled = props.selected_algsets
                                .get(&algset.name)
                                .copied()
                                .unwrap_or(algset.selected);
                            let selected_count = case_ids.iter().filter(|id| props.selected.contains(id)).count();
                            let count = if enabled { selected_count.to_string() } else { "-".to_owned() };
                            let fully_selected = enabled
                                && !case_ids.is_empty()
                                && selected_count == case_ids.len();
                            let label = format!("{} ({}/{})", algset.name, count, case_ids.len());
                            let name = algset.name.clone();
                            html! {
                                <button
                                    class={classes!(
                                        "borderedContainer",
                                        "pad",
                                        if fully_selected { "itemSel" } else { "itemUnsel" },
                                        (!enabled).then_some("algsetDisabled")
                                    )}
                                    disabled={search_active && case_ids.is_empty()}
                                    onclick={Callback::from(move |_| {
                                        if search_active {
                                            on_toggle_case_group.emit(case_ids.clone());
                                        } else {
                                            on_toggle_algset.emit((name.clone(), case_ids.clone()));
                                        }
                                    })}
                                >
                                    {label}
                                </button>
                            }
                        }) }
                    </div>
                }

                <div class="rowFlex" style="align-items:stretch;">
                    <div
                        id="allSelector"
                        class={classes!(
                            "borderedContainer",
                            "pad",
                            if !metrics.visible_case_ids.is_empty()
                                && metrics.visible_case_ids.iter().all(|id| props.selected.contains(id))
                            {
                                "itemSel"
                            } else {
                                "itemUnsel"
                            }
                        )}
                        style="flex-grow:1; text-align:center;"
                        onclick={{
                            let on_toggle_case_group = props.on_toggle_case_group.clone();
                            let visible_case_ids = metrics.visible_case_ids.clone();
                            Callback::from(move |_| on_toggle_case_group.emit(visible_case_ids.clone()))
                        }}
                    >
                        <b>{format!(
                            "{} ({}/{})",
                            if metrics.search_active { "Search Results" } else { "All Cases" },
                            metrics.visible_case_ids.iter().filter(|id| props.selected.contains(id)).count(),
                            metrics.visible_case_ids.len()
                        )}</b>
                        if metrics.search_active || metrics.enabled_case_ids.len() < bundle.cases.len() {
                            <span class="hiddenCaseCount">
                                {format!("Hidden: {} ({} selected)", metrics.hidden_cases, metrics.hidden_selected)}
                            </span>
                        }
                    </div>
                    <button
                        class={classes!(
                            "borderedContainer",
                            "pad",
                            "searchToggle",
                            props.search_open.then_some("searchToggleOpen"),
                            metrics.search_active.then_some("searchToggleActive")
                        )}
                        style="display:flex; flex-direction:column; justify-content:center; padding-left:1em; padding-right:1em;"
                        title="Search"
                        aria-label="Search cases"
                        onclick={on_toggle_search}
                    >
                        <span class="material-symbols-outlined">{"search"}</span>
                    </button>
                </div>

                if props.search_open {
                    <SearchPanel
                        bundle={bundle.clone()}
                        cube_colors={props.cube_colors.clone()}
                        query={props.search_query.clone()}
                        cells={props.search_cells.clone()}
                        active_color={props.search_color.clone()}
                        all_angles={props.search_all_angles}
                        saved_queries={props.saved_search_queries.clone()}
                        active_query={props.active_search_query.clone()}
                        default_query={props.default_search_query.clone()}
                        on_query_change={props.on_search_query_change.clone()}
                        on_cells_change={props.on_search_cells_change.clone()}
                        on_color_change={props.on_search_color_change.clone()}
                        on_all_angles_change={props.on_search_all_angles_change.clone()}
                        on_save_query={props.on_save_search_query.clone()}
                        on_use_query={props.on_use_search_query.clone()}
                        on_set_default_query={props.on_set_default_search_query.clone()}
                        on_delete_query={props.on_delete_search_query.clone()}
                    />
                }

                <>
                            { for bundle.algsets.iter()
                                .filter(|algset| {
                                    props.selected_algsets
                                        .get(&algset.name)
                                        .copied()
                                        .unwrap_or(algset.selected)
                                })
                                .map(|algset| {
                                    let groups = bundle.groups.iter()
                                        .filter(|group| algset.groups.contains(&group.name))
                                        .filter_map(|group| {
                                            let visible_cases = group.cases.iter()
                                                .filter_map(|id| props.cases_by_id.get(id))
                                                .filter(|case| case_matches_search(
                                                    case,
                                                    bundle,
                                                    &search_needle,
                                                    &props.search_cells,
                                                    props.search_all_angles,
                                                ))
                                                .map(|case| case.case_id.clone())
                                                .collect::<Vec<_>>();
                                            if metrics.search_active && visible_cases.is_empty() {
                                                None
                                            } else {
                                                Some((group, visible_cases))
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    if metrics.search_active && groups.is_empty() {
                                        return Html::default();
                                    }
                                    html! {
                                        <section class="algsetSection">
                                            <div class="borderedContainer pad algsetNameHeader">
                                                <b>{&algset.name}</b>
                                            </div>
                                            <div class={classes!(
                                                "algsetGroups",
                                                compact_group_layout.then_some("algsetGroupsCompact")
                                            )}>
                                                { for groups.into_iter().map(|(group, visible_cases)| {
                                                    let group_selected = group.cases.iter().all(|id| props.selected.contains(id));
                                                    let on_toggle_group = {
                                                        let cb = props.on_toggle_case_group.clone();
                                                        let targets = if metrics.search_active {
                                                            visible_cases.clone()
                                                        } else {
                                                            group.cases.clone()
                                                        };
                                                        Callback::from(move |_| cb.emit(targets.clone()))
                                                    };
                                                    html! {
                                                        <section class={classes!(
                                                            "caseGroup",
                                                            (visible_cases.len() == metrics.widest_group_cases).then_some("widestCaseGroup")
                                                        )}>
                                                            <div
                                                                class={classes!(
                                                                    "borderedContainer",
                                                                    "pad",
                                                                    "groupNameDiv",
                                                                    if group_selected { "itemSel" } else { "itemUnsel" }
                                                                )}
                                                                style="text-align:center;"
                                                                onclick={on_toggle_group}
                                                            >
                                                                <b>{&group.name}</b>
                                                            </div>
                                                            <div class="rowFlex caseGroupCases">
                                                                { for visible_cases.iter().filter_map(|id| props.cases_by_id.get(id)).map(|case| {
                                                                    let probability = show_probabilities
                                                                        .then(|| props.probabilities.get(&case.case_id).copied())
                                                                        .flatten();
                                                                    html! {
                                                                        <CaseCard
                                                                            case={case.clone()}
                                                                            selected={props.selected.contains(&case.case_id)}
                                                                            {include_name}
                                                                            cube_colors={props.cube_colors.clone()}
                                                                            {probability}
                                                                            on_toggle={props.on_toggle_case.clone()}
                                                                            on_open_hint={props.on_open_hint.clone()}
                                                                        />
                                                                    }
                                                                }) }
                                                            </div>
                                                        </section>
                                                    }
                                                }) }
                                            </div>
                                        </section>
                                    }
                                })
                            }
                </>
            </div>
        </div>
    }
}
