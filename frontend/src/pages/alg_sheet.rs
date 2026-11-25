use crate::components::case_image::CaseImage;
use crate::logic::{algs, cube_schemes, search};
use crate::models::{TrainerBundle, UserAlgState};
use crate::services::api::fetch_trainer_bundle;
use crate::services::storage::{load_custom_algs, load_settings};
use std::collections::BTreeMap;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component(AlgSheetPage)]
pub fn alg_sheet_page() -> Html {
    let bundle = use_state(|| None::<TrainerBundle>);
    let query = use_state(String::new);
    let custom_algs = use_state(BTreeMap::<String, Vec<UserAlgState>>::new);
    let settings = use_state(|| {
        let mut initial = serde_json::json!({});
        cube_schemes::migrate(&mut initial);
        initial
    });

    {
        let settings = settings.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                settings.set(load_settings(false).await);
            });
            || ()
        });
    }

    {
        let bundle = bundle.clone();
        use_effect_with((), move |_| {
            let id = web_sys::window()
                .and_then(|window| window.location().search().ok())
                .unwrap_or_default()
                .strip_prefix("?algset=")
                .unwrap_or("")
                .to_owned();
            spawn_local(async move {
                if let Ok(value) = fetch_trainer_bundle(&id).await {
                    bundle.set(Some(value));
                }
            });
            || ()
        });
    }

    {
        let custom_algs = custom_algs.clone();
        let loaded = bundle.clone();
        let dependency = bundle
            .as_ref()
            .map(|value| value.selection_key.clone())
            .unwrap_or_default();
        use_effect_with(dependency, move |_| {
            if let Some(bundle) = loaded.as_ref() {
                let selection_key = bundle.selection_key.clone();
                let cases = bundle.cases.clone();
                spawn_local(async move {
                    let loaded_algs = load_custom_algs(&selection_key, &cases).await;
                    if !loaded_algs.is_empty() {
                        custom_algs.set(loaded_algs);
                    }
                });
            }
            || ()
        });
    }

    let Some(bundle) = bundle.as_ref() else {
        return html! { <main class="colFlex pad"><h1>{"Loading algorithm sheet..."}</h1></main> };
    };

    let needle = query.trim().to_lowercase();
    let cube_colors = Rc::new(cube_schemes::active_colors(
        &settings,
        &bundle.trainer_id,
        &bundle.puzzle,
    ));
    let cases = bundle
        .cases
        .iter()
        .map(|case| (case.case_id.clone(), case))
        .collect::<BTreeMap<_, _>>();

    html! {
        <main class="colFlex" style="align-items:center;width:100%;padding:0.5em;">
            <div class="rowFlex center" style="justify-content:space-between;width:min(80vw,62em);">
                <h1>{format!("{} algorithms", bundle.title)}</h1>
                <a class="abutton" href={format!("/trainer/{}", bundle.trainer_id)}>{"Back"}</a>
            </div>
            <div class="borderedContainer pad rowFlex" style="width:min(80vw,62em);">
                <input
                    type="search"
                    placeholder="Search algorithms"
                    style="width:100%;"
                    value={(*query).clone()}
                    oninput={{
                        let query = query.clone();
                        Callback::from(move |event: InputEvent| {
                            query.set(event.target_unchecked_into::<HtmlInputElement>().value());
                        })
                    }}
                />
            </div>
            <div class="algsheetGrid">
                {for bundle.algsets.iter().map(|algset| {
                    let needle = needle.clone();
                    let cases = cases.clone();
                    let custom_algs = custom_algs.clone();
                    let cube_colors = cube_colors.clone();
                    html! {
                        <>
                            <div class="borderedContainer itemUnsel pad groupNameDiv fullSpan"><b>{&algset.name}</b></div>
                            {for algset.groups.iter().filter_map(|group_name| bundle.groups.iter().find(|group| &group.name == group_name)).map(|group| {
                                let visible_cases = group.cases.iter()
                                    .filter_map(|id| cases.get(id).copied())
                                    .filter(|case| search::case_matches_search(case, bundle, &needle, &BTreeMap::new(), false))
                                    .collect::<Vec<_>>();
                                if visible_cases.is_empty() {
                                    return Html::default();
                                }
                                html! {
                                    <>
                                        <div class="borderedContainer itemUnsel pad groupNameDiv fullSpan">{&group.name}</div>
                                        {for visible_cases.into_iter().map(|case| html! {
                                            <section class="algsheetCase colFlex">
                                                <div class="itemUnsel borderedContainer algsheetCaseImage">
                                                    <b class="caseName">{&case.name}</b>
                                                    <span class="caseSetup">{case.setup.as_deref().unwrap_or("")}</span>
                                                    if let Some(svg) = case.image_svg.as_ref() {
                                                        <CaseImage svg={svg.clone()} colors={cube_colors.clone()} class="caseImage" />
                                                    }
                                                </div>
                                                <div class="colFlex algsheetAlgs">
                                                    {for algs::display_algs(case, &custom_algs).into_iter().map(|alg| html! {
                                                        <div class="borderedContainer pad">{alg.alg}</div>
                                                    })}
                                                </div>
                                            </section>
                                        })}
                                    </>
                                }
                            })}
                        </>
                    }
                })}
            </div>
        </main>
    }
}
