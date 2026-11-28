use crate::logic::search::{cube_color, search_color_names, search_template_svg};
use crate::models::{SavedSearchQuery, TrainerBundle};
use std::collections::BTreeMap;
use std::rc::Rc;
use web_sys::{Element, HtmlInputElement, HtmlSelectElement, MouseEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SearchPanelProps {
    pub bundle: Rc<TrainerBundle>,
    pub cube_colors: Rc<BTreeMap<String, String>>,
    pub query: String,
    pub cells: BTreeMap<usize, String>,
    pub active_color: String,
    pub all_angles: bool,
    pub saved_queries: BTreeMap<String, SavedSearchQuery>,
    pub active_query: String,
    pub default_query: Option<String>,
    pub on_query_change: Callback<String>,
    pub on_cells_change: Callback<BTreeMap<usize, String>>,
    pub on_color_change: Callback<String>,
    pub on_all_angles_change: Callback<bool>,
    pub on_save_query: Callback<(String, bool)>,
    pub on_use_query: Callback<String>,
    pub on_set_default_query: Callback<(String, bool)>,
    pub on_delete_query: Callback<String>,
}

#[function_component(SearchPanel)]
pub fn search_panel(props: &SearchPanelProps) -> Html {
    html! {
        <div class="searchPanel borderedContainer pad colFlex">
            if let Some(template) = search_template_svg(&props.bundle, &props.cells, &props.cube_colors) {
                <div
                    class="searchDiagram borderedContainer itemUnsel pad"
                    onclick={{
                        let cells = props.cells.clone();
                        let color = props.active_color.clone();
                        let on_cells_change = props.on_cells_change.clone();
                        Callback::from(move |event: MouseEvent| {
                            let Some(element) = event.target_dyn_into::<Element>() else {
                                return;
                            };
                            let Some(index) = element
                                .get_attribute("data-search-index")
                                .and_then(|value| value.parse::<usize>().ok())
                            else {
                                return;
                            };
                            let mut next = cells.clone();
                            if next.get(&index) == Some(&color) {
                                next.remove(&index);
                            } else {
                                next.insert(index, color.clone());
                            }
                            on_cells_change.emit(next);
                        })
                    }}
                >
                    {Html::from_html_unchecked(template.into())}
                    <button
                        class="material-symbols-outlined searchDiagramClear compactIconButton"
                        title="Clear image search"
                        aria-label="Clear image search"
                        onclick={{
                            let on_cells_change = props.on_cells_change.clone();
                            Callback::from(move |_| on_cells_change.emit(BTreeMap::new()))
                        }}
                    >
                        {"close"}
                    </button>
                </div>
            }
            <div class="rowFlex searchOptions">
                if !props.bundle.u_move_cycles.is_empty() {
                    <label class="rowFlex searchAngleToggle">
                        <span>{"Search all angles"}</span>
                        <input
                            type="checkbox"
                            checked={props.all_angles}
                            onchange={{
                                let on_all_angles_change = props.on_all_angles_change.clone();
                                Callback::from(move |event: Event| {
                                    on_all_angles_change.emit(
                                        event.target_unchecked_into::<HtmlInputElement>().checked(),
                                    );
                                })
                            }}
                        />
                    </label>
                }
                <label class="rowFlex savedSearchDefault">
                    <span>{"Default"}</span>
                    <input
                        type="checkbox"
                        checked={if props.active_query.is_empty() {
                            props.default_query.is_none()
                        } else {
                            props.default_query.as_deref() == Some(props.active_query.as_str())
                        }}
                        onchange={{
                            let on_set_default_query = props.on_set_default_query.clone();
                            let active_query = props.active_query.clone();
                            Callback::from(move |event: Event| {
                                on_set_default_query.emit((
                                    active_query.clone(),
                                    event
                                        .target_unchecked_into::<HtmlInputElement>()
                                        .checked(),
                                ));
                            })
                        }}
                    />
                </label>
            </div>
            <div class="rowFlex searchPalette" aria-label="Search colors">
                {for search_color_names().into_iter().map(|name| {
                    let on_color_change = props.on_color_change.clone();
                    let active = props.active_color == name;
                    let color = cube_color(&props.cube_colors, name);
                    html! {
                        <button
                            class={classes!("searchSwatch", active.then_some("active"))}
                            title={name}
                            aria-label={format!("Use {name} search color")}
                            aria-pressed={active.to_string()}
                            style={format!("background:{color}")}
                            onclick={Callback::from(move |_| on_color_change.emit(name.to_owned()))}
                        >
                        </button>
                    }
                })}
            </div>
            <div class="rowFlex searchTextRow">
                <span class="material-symbols-outlined">{"search"}</span>
                <input
                    id="searchTextInput"
                    type="search"
                    placeholder="Search algset, group, name, setup or algorithm"
                    value={props.query.clone()}
                    oninput={{
                        let on_query_change = props.on_query_change.clone();
                        Callback::from(move |event: InputEvent| {
                            on_query_change.emit(
                                event.target_unchecked_into::<HtmlInputElement>().value(),
                            );
                        })
                    }}
                />
                <button
                    class="material-symbols-outlined abutton compactButton"
                    title="Save search query"
                    aria-label="Save search query"
                    onclick={{
                        let on_save_query = props.on_save_query.clone();
                        Callback::from(move |_| {
                            let Some(window) = web_sys::window() else {
                                return;
                            };
                            let Ok(Some(name)) = window.prompt_with_message("Name this search query")
                            else {
                                return;
                            };
                            let name = name.trim().to_owned();
                            if name.is_empty() || name.eq_ignore_ascii_case("No filter") {
                                if !name.is_empty() {
                                    let _ = window.alert_with_message(
                                        "\"No filter\" is reserved and cannot be edited.",
                                    );
                                }
                                return;
                            }
                            let make_default = window
                                .confirm_with_message(&format!(
                                    "Set \"{name}\" as the default search query?"
                                ))
                                .unwrap_or(false);
                            on_save_query.emit((name, make_default));
                        })
                    }}
                >
                    {"save"}
                </button>
                if !props.active_query.is_empty() {
                    <button
                        class="material-symbols-outlined abutton compactButton savedSearchDelete"
                        title="Delete saved search query"
                        aria-label="Delete saved search query"
                        onclick={{
                            let on_delete_query = props.on_delete_query.clone();
                            let active_query = props.active_query.clone();
                            Callback::from(move |_| on_delete_query.emit(active_query.clone()))
                        }}
                    >
                        {"delete"}
                    </button>
                }
                <button
                    class="material-symbols-outlined abutton compactButton"
                    title="Clear search"
                    aria-label="Clear search"
                    onclick={{
                        let on_use_query = props.on_use_query.clone();
                        Callback::from(move |_| on_use_query.emit(String::new()))
                    }}
                >
                    {"close"}
                </button>
            </div>
            <div class="rowFlex savedSearchRow">
                <select
                    aria-label="Saved search queries"
                    value={props.active_query.clone()}
                    onchange={{
                        let on_use_query = props.on_use_query.clone();
                        Callback::from(move |event: Event| on_use_query.emit(
                            event
                                .target_unchecked_into::<HtmlSelectElement>()
                                .value(),
                        ))
                    }}
                >
                    <option value="" selected={props.active_query.is_empty()}>{"No filter"}</option>
                    {for props.saved_queries.keys().map(|name| html! {
                        <option value={name.clone()} selected={props.active_query == *name}>
                            {name}
                        </option>
                    })}
                </select>
            </div>
        </div>
    }
}
