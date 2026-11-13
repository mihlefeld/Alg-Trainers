const APP_VERSION = "1.8.0";

// Export for service worker (importScripts) and expose globally for pages
if (typeof self !== "undefined") {
    self.APP_VERSION = APP_VERSION;
}
