
/// \value stringified json object or standard type
/// \returns true if succeed
function saveLocal(name, value) {
    // If the platform supports localStorage, then save the value
    try {
        localStorage.setItem(name, value);
        return true;
    }
    catch(e) {
        // Most likely cause of errors is a very old browser that doesn't support localStorage (fail silently)
        console.warn("saving error");
        return false;
    }
}

/// \returns loaded value or specified defaultValue in case of error
function loadLocal(name, defaultValue) {
    // If the platform supports localStorage, then load the selection
    try {
        return localStorage.getItem(name);
    }
    catch(e) {
        // Either no selection in localStorage or browser does not support localStorage (fail silently)
        console.warn("can\'t load from localstorage");
        return defaultValue;
    }
}

function saveSelection() {
    return saveLocal(selectionArrayKey, JSON.stringify(selCases));
}

function loadSelection() {
    selCases = JSON.parse(loadLocal(selectionArrayKey, '[1,2]'));
    if (selCases == null)
        selCases = [1,2];
    saveSelection();
}
