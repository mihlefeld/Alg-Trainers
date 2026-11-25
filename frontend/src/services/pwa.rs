use wasm_bindgen::prelude::*;

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
    pub fn initPwa(on_update_available: &Closure<dyn FnMut()>);
    pub fn applyPwaUpdate() -> bool;
    pub fn isNavigatorOnline() -> bool;
}
