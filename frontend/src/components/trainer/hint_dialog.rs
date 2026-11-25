use crate::components::case_image::CaseImage;
use crate::components::modal::ModalDialog;
use crate::logic::algs::display_algs;
use crate::models::{DisplayAlg, TrainerCase, UserAlgState};
use std::collections::BTreeMap;
use std::rc::Rc;
use web_sys::{DragEvent, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
struct AlgEditorRowProps {
    value: String,
    #[prop_or_default]
    placeholder: Option<AttrValue>,
    save_title: AttrValue,
    cancel_title: AttrValue,
    on_input: Callback<String>,
    on_save: Callback<()>,
    on_cancel: Callback<()>,
}

#[function_component(AlgEditorRow)]
fn alg_editor_row(props: &AlgEditorRowProps) -> Html {
    let oninput = {
        let on_input = props.on_input.clone();
        Callback::from(move |event: InputEvent| {
            on_input.emit(event.target_unchecked_into::<HtmlTextAreaElement>().value());
        })
    };
    let on_save = {
        let cb = props.on_save.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_cancel = {
        let cb = props.on_cancel.clone();
        Callback::from(move |_| cb.emit(()))
    };

    html! {
        <>
            <textarea
                class="algEditorInput"
                rows="2"
                placeholder={props.placeholder.clone()}
                value={props.value.clone()}
                {oninput}
            />
            <button
                class="material-symbols-outlined algAction"
                title={props.save_title.clone()}
                onclick={on_save}
            >
                {"check"}
            </button>
            <button
                class="material-symbols-outlined algAction"
                title={props.cancel_title.clone()}
                onclick={on_cancel}
            >
                {"close"}
            </button>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct HintDialogProps {
    pub case: Option<TrainerCase>,
    pub custom_algs: BTreeMap<String, Vec<UserAlgState>>,
    pub cube_colors: Rc<BTreeMap<String, String>>,
    pub on_close: Callback<()>,
    pub on_prev: Callback<()>,
    pub on_next: Callback<()>,
    pub on_save_alg: Callback<(Option<String>, String)>,
    pub on_hide_alg: Callback<DisplayAlg>,
    pub on_delete_alg: Callback<String>,
    pub on_reorder_alg: Callback<(String, String)>,
}

#[function_component(HintDialog)]
pub fn hint_dialog(props: &HintDialogProps) -> Html {
    let alg_editor = use_state(|| None::<(Option<String>, String)>);
    let dragged_alg = use_state(String::new);

    {
        let alg_editor = alg_editor.clone();
        let case_id = props
            .case
            .as_ref()
            .map(|c| c.case_id.clone())
            .unwrap_or_default();
        use_effect_with(case_id, move |_| {
            alg_editor.set(None);
            || ()
        });
    }

    let save_editor = {
        let alg_editor = alg_editor.clone();
        let on_save_alg = props.on_save_alg.clone();
        Callback::from(move |_| {
            if let Some((alg_id, value)) = (*alg_editor).clone() {
                if !value.trim().is_empty() {
                    on_save_alg.emit((alg_id, value));
                    alg_editor.set(None);
                }
            }
        })
    };

    let cancel_editor = {
        let alg_editor = alg_editor.clone();
        Callback::from(move |_| alg_editor.set(None))
    };

    let on_prev = {
        let cb = props.on_prev.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let on_next = {
        let cb = props.on_next.clone();
        Callback::from(move |_| cb.emit(()))
    };

    html! {
        <ModalDialog id="hintWindow" class="colFlex" on_close={props.on_close.clone()}>
            if let Some(case) = props.case.as_ref() {
                <div class="borderedContainer rowFlex pad center hintCaseHeader" style="justify-content:space-between;">
                    <button class="material-symbols-outlined abutton prevNext" onclick={on_prev} title="Previous case">{"arrow_back"}</button>
                    <b>{&case.name}</b>
                    <button class="material-symbols-outlined abutton prevNext" onclick={on_next} title="Next case">{"arrow_forward"}</button>
                </div>
                <div class="borderedContainer colFlex center" style="background-color:var(--secondary);">
                    if let Some(svg) = case.image_svg.as_ref() {
                        <CaseImage svg={svg.clone()} colors={props.cube_colors.clone()} style="width:16em;" />
                    }
                </div>
                <div class="borderedContainer grow rowFlex center pad">{case.setup.as_deref().unwrap_or("")}</div>
                <div class="borderedContainer grow colFlex algList">
                    {for display_algs(case, &props.custom_algs).into_iter().map(|item| {
                        let editing = alg_editor.as_ref().is_some_and(|(id, _)| id.as_deref() == Some(item.alg_id.as_str()));
                        let dragged_alg_start = dragged_alg.clone();
                        let dragged_alg_drop = dragged_alg.clone();
                        let on_reorder_alg = props.on_reorder_alg.clone();
                        let on_hide_alg = props.on_hide_alg.clone();
                        let on_delete_alg = props.on_delete_alg.clone();
                        let alg_editor = alg_editor.clone();
                        let drag_id = item.alg_id.clone();
                        let drop_id = item.alg_id.clone();
                        html! {
                            <div
                                class="algItem borderedContainer"
                                ondragover={Callback::from(|event: DragEvent| event.prevent_default())}
                                ondrop={Callback::from(move |event: DragEvent| {
                                    event.prevent_default();
                                    if !dragged_alg_drop.is_empty() && dragged_alg_drop.as_str() != drop_id.as_str() {
                                        on_reorder_alg.emit((dragged_alg_drop.to_string(), drop_id.clone()));
                                    }
                                })}
                            >
                                <button
                                    class="material-symbols-outlined algHandle"
                                    draggable="true"
                                    title="Drag to reorder"
                                    aria-label="Drag to reorder"
                                    ondragstart={Callback::from(move |_| dragged_alg_start.set(drag_id.clone()))}
                                >
                                    {"drag_indicator"}
                                </button>
                                if editing {
                                    <AlgEditorRow
                                        value={alg_editor.as_ref().map(|(_, value)| value.clone()).unwrap_or_default()}
                                        save_title="Save algorithm"
                                        cancel_title="Cancel edit"
                                        on_input={{
                                            let alg_editor = alg_editor.clone();
                                            Callback::from(move |value: String| {
                                                if let Some((id, _)) = alg_editor.as_ref() {
                                                    alg_editor.set(Some((id.clone(), value)));
                                                }
                                            })
                                        }}
                                        on_save={save_editor.clone()}
                                        on_cancel={cancel_editor.clone()}
                                    />
                                } else {
                                    <span class="algText">{&item.alg}</span>
                                    if item.user {
                                        <button
                                            class="material-symbols-outlined algAction"
                                            title="Edit algorithm"
                                            onclick={{
                                                let alg_editor = alg_editor.clone();
                                                let id = item.alg_id.clone();
                                                let value = item.alg.clone();
                                                Callback::from(move |_| alg_editor.set(Some((Some(id.clone()), value.clone()))))
                                            }}
                                        >
                                            {"edit"}
                                        </button>
                                        <button
                                            class="material-symbols-outlined algAction"
                                            title="Delete algorithm"
                                            onclick={{
                                                let id = item.alg_id.clone();
                                                Callback::from(move |_| on_delete_alg.emit(id.clone()))
                                            }}
                                        >
                                            {"close"}
                                        </button>
                                    } else {
                                        <button
                                            class="material-symbols-outlined algAction"
                                            title="Hide algorithm"
                                            onclick={Callback::from(move |_| on_hide_alg.emit(item.clone()))}
                                        >
                                            {"visibility_off"}
                                        </button>
                                    }
                                }
                            </div>
                        }
                    })}
                    if let Some((None, value)) = alg_editor.as_ref() {
                        <div class="algItem algNewItem borderedContainer">
                            <AlgEditorRow
                                value={value.clone()}
                                placeholder="Enter a new algorithm"
                                save_title="Add algorithm"
                                cancel_title="Cancel"
                                on_input={{
                                    let alg_editor = alg_editor.clone();
                                    Callback::from(move |value: String| alg_editor.set(Some((None, value))))
                                }}
                                on_save={save_editor}
                                on_cancel={cancel_editor}
                            />
                        </div>
                    } else {
                        <button
                            class="material-symbols-outlined abutton algAddButton"
                            title="Add algorithm"
                            onclick={{
                                let alg_editor = alg_editor.clone();
                                Callback::from(move |_| alg_editor.set(Some((None, String::new()))))
                            }}
                        >
                            {"add"}
                        </button>
                    }
                </div>
            }
        </ModalDialog>
    }
}
