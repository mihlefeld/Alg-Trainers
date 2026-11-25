use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen(inline_js = r#"
let swRegistration = null;
let userApprovedUpdate = false;
let refreshing = false;
let updateListeners = [];

function notifyUpdateAvailable() {
  for (const listener of updateListeners) {
    try {
      listener();
    } catch (_) {}
  }
}

function trackInstallingWorker(worker) {
  if (!worker) return;
  worker.addEventListener("statechange", () => {
    if (worker.state === "installed" && navigator.serviceWorker.controller) {
      notifyUpdateAvailable();
    }
  });
}

export function initPwa(onUpdateAvailable) {
  updateListeners.push(onUpdateAvailable);

  if (!("serviceWorker" in navigator)) {
    return;
  }

  if (swRegistration) {
    if (swRegistration.waiting) {
      onUpdateAvailable();
    }
    return;
  }

  navigator.serviceWorker.addEventListener("controllerchange", () => {
    if (!userApprovedUpdate || refreshing) return;
    refreshing = true;
    window.location.reload();
  });

  navigator.serviceWorker
    .register("/sw.js", { updateViaCache: "none" })
    .then((registration) => {
      swRegistration = registration;
      if (registration.waiting) {
        notifyUpdateAvailable();
      }
      if (registration.installing) {
        trackInstallingWorker(registration.installing);
      }
      registration.addEventListener("updatefound", () => {
        trackInstallingWorker(registration.installing);
      });

      window.addEventListener("online", () => {
        registration.update().catch(() => {});
      });
      document.addEventListener("visibilitychange", () => {
        if (!document.hidden && navigator.onLine) {
          registration.update().catch(() => {});
        }
      });
    })
    .catch((error) => {
      console.warn("Service worker registration failed:", error);
    });
}

export function applyPwaUpdate() {
  if (!swRegistration || !swRegistration.waiting) {
    return false;
  }
  userApprovedUpdate = true;
  swRegistration.waiting.postMessage({ type: "SKIP_WAITING" });
  return true;
}

export function isNavigatorOnline() {
  return typeof navigator === "undefined" ? true : navigator.onLine !== false;
}
"#)]
extern "C" {
    fn initPwa(on_update_available: &Closure<dyn FnMut()>);
    fn applyPwaUpdate() -> bool;
    pub fn isNavigatorOnline() -> bool;
}

#[function_component(UpdatePrompt)]
pub fn update_prompt() -> Html {
    let update_available = use_state(|| false);
    let dismissed = use_state(|| false);
    let updating = use_state(|| false);

    {
        let update_available = update_available.clone();
        use_effect_with((), move |_| {
            let callback = Closure::<dyn FnMut()>::new(move || {
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
