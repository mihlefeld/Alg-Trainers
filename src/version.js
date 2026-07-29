const APP_VERSION = "1.7.4";

// Export for service worker (importScripts) and expose globally for pages
if (typeof self !== "undefined") {
    self.APP_VERSION = APP_VERSION;
}