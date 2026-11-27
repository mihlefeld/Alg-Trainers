use crate::services::pwa::{applyPwaUpdate, initPwa};
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

#[function_component(UpdatePrompt)]
pub fn update_prompt() -> Html {
    let update_available = use_state(|| false);
    let dismissed = use_state(|| false);
    let updating = use_state(|| false);

    {
        let update_available = update_available.clone();
        let dismissed = dismissed.clone();
        use_effect_with((), move |_| {
            let callback = Closure::<dyn FnMut()>::new(move || {
                dismissed.set(false);
                update_available.set(true);
            });
            initPwa(&callback);
            callback.forget();
            || ()
        });
    }

    if !*update_available || *dismissed {
        return Html::default();
    }

    let on_update = {
        let updating = updating.clone();
        Callback::from(move |_| {
            updating.set(true);
            if !applyPwaUpdate() {
                updating.set(false);
            }
        })
    };

    let on_dismiss = {
        let dismissed = dismissed.clone();
        Callback::from(move |_| {
            dismissed.set(true);
        })
    };

    html! {
        <div
            id="pwa-update-banner"
            class="pwaUpdateBanner borderedContainer"
            role="alertdialog"
            aria-live="polite"
            aria-label="App update available"
        >
            <div class="pwaUpdateText">
                <b>{"Update available"}</b>
                <span>{"A new version of ALLG Trainer is ready to install."}</span>
            </div>
            <div class="rowFlex pwaUpdateActions">
                <button
                    id="pwa-update-now"
                    class="abutton pwaUpdatePrimary"
                    disabled={*updating}
                    onclick={on_update}
                >
                    {if *updating { "Updating..." } else { "Update now" }}
                </button>
                <button
                    id="pwa-update-later"
                    class="abutton"
                    disabled={*updating}
                    onclick={on_dismiss}
                >
                    {"Later"}
                </button>
            </div>
        </div>
    }
}
