use wasm_bindgen::JsCast;
use web_sys::{Element, MouseEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ModalDialogProps {
    pub id: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    pub on_close: Callback<()>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(ModalDialog)]
pub fn modal_dialog(props: &ModalDialogProps) -> Html {
    let on_backdrop_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |event: MouseEvent| {
            if event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
                .is_some_and(|target| target.tag_name() == "DIALOG")
            {
                on_close.emit(());
            }
        })
    };

    let on_close_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| on_close.emit(()))
    };

    html! {
        <dialog id={props.id.clone()} class={props.class.clone()} open=true onclick={on_backdrop_click}>
            <div class="dialogDiv colFlex borderedContainer">
                <button
                    class="material-symbols-outlined abutton closeButton"
                    onclick={on_close_click}
                    aria-label="Close"
                >
                    {"close"}
                </button>
                { for props.children.iter() }
            </div>
        </dialog>
    }
}
