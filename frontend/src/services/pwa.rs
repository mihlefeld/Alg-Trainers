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

export async function checkForPwaUpdate() {
  if (!("serviceWorker" in navigator)) {
    return "unsupported";
  }
  if (navigator.onLine === false) {
    return "offline";
  }
  try {
    let registration = swRegistration || (await navigator.serviceWorker.getRegistration());
    if (!registration) {
      registration = await navigator.serviceWorker.register("/sw.js", { updateViaCache: "none" });
    }
    swRegistration = registration;
    if (registration.waiting) {
      notifyUpdateAvailable();
      return "available";
    }
    await registration.update();
    if (registration.waiting) {
      notifyUpdateAvailable();
      return "available";
    }
    const installing = registration.installing;
    if (installing) {
      trackInstallingWorker(installing);
      const result = await new Promise((resolve) => {
        const onStateChange = () => {
          if (installing.state === "installed") {
            installing.removeEventListener("statechange", onStateChange);
            if (navigator.serviceWorker.controller) {
              notifyUpdateAvailable();
              resolve("available");
            } else {
              resolve("up-to-date");
            }
          } else if (installing.state === "redundant") {
            installing.removeEventListener("statechange", onStateChange);
            resolve("error");
          }
        };
        installing.addEventListener("statechange", onStateChange);
      });
      return result;
    }
    return "up-to-date";
  } catch (error) {
    console.warn("Update check failed:", error);
    return navigator.onLine === false ? "offline" : "error";
  }
}

export function isNavigatorOnline() {
  return typeof navigator === "undefined" ? true : navigator.onLine !== false;
}
"#)]
extern "C" {
    pub fn initPwa(on_update_available: &Closure<dyn FnMut()>);
    pub fn applyPwaUpdate() -> bool;
    #[wasm_bindgen(catch)]
    pub fn checkForPwaUpdate() -> Result<js_sys::Promise, JsValue>;
    pub fn isNavigatorOnline() -> bool;
}
