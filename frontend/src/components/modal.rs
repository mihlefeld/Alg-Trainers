use std::cell::Cell;

use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Element, MouseEvent};
use yew::prelude::*;

thread_local! {
    static IGNORE_MODAL_HISTORY_POP: Cell<bool> = const { Cell::new(false) };
}

pub(crate) fn take_modal_history_pop() -> bool {
    IGNORE_MODAL_HISTORY_POP.with(|ignore| ignore.replace(false))
}

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
    {
        let id = props.id.clone();
        let on_close = props.on_close.clone();
        use_effect_with(id.clone(), move |_| {
            let Some(window) = web_sys::window() else {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            };
            let Ok(history) = window.history() else {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            };
            let marker = format!("modal:{}", id);
            let modal_url = window.location().href().ok().map(|href| {
                let base_url = href.split('#').next().unwrap_or(&href);
                format!("{base_url}#modal={id}")
            });
            if history
                .push_state_with_url(&JsValue::from_str(&marker), "", modal_url.as_deref())
                .is_err()
            {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            }

            let listener = Closure::wrap(Box::new(move || on_close.emit(())) as Box<dyn FnMut()>);
            let _ = window
                .add_event_listener_with_callback("popstate", listener.as_ref().unchecked_ref());

            let cleanup_window = window;
            let cleanup_history = history;
            Box::new(move || {
                let _ = cleanup_window.remove_event_listener_with_callback(
                    "popstate",
                    listener.as_ref().unchecked_ref(),
                );
                if cleanup_history
                    .state()
                    .ok()
                    .and_then(|state| state.as_string())
                    .as_deref()
                    == Some(&marker)
                {
                    IGNORE_MODAL_HISTORY_POP.with(|ignore| ignore.set(true));
                    let _ = cleanup_history.back();
                }
            }) as Box<dyn FnOnce()>
        });
    }

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
