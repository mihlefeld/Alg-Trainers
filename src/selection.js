
var selCases = [1, 2];
var selectionPresets = {
    "Default": {
        "selCases": [],
        "selectedAlgSets": {}
    }
}

function getAlgsetIds(algset) {
    if (algsets[algset]) {
        var algsetIds = []
        for (const group of algsets[algset]) {
            algsetIds = algsetIds.concat(getGroupIds(group));
        }
        return algsetIds;
    }
    return [];
}

function getAllValidGroups() {
    var valid = [];
    for (const [algset, groups] of Object.entries(algsets)) {
        if (selectedAlgSets[algset]) {
            valid = valid.concat(groups);
        }
    }
    return valid;
}

function getAllValid(ignoreSearch) {
    var valid = [];
    var validGroups = getAllValidGroups();
    for (const group of validGroups) {
        valid = valid.concat(ignoreSearch ? algsGroups[group] : getGroupIds(group));
    }
    return valid;
}

function countAlgsetSelected(algset) {
    var algsetIds = getAlgsetIds(algset);
    var selectedCount = 0;
    for (const idx of algsetIds) {
        selectedCount += selCases.includes(idx);
    }
    return selectedCount;
}

function isAlgsetAllSelected(algset) {
    var algsetIds = getAlgsetIds(algset);
    var selectedCount = 0;
    for (const idx of algsetIds) {
        selectedCount += selCases.includes(idx);
    }
    var allSelected = selectedCount == algsetIds.length;
    return allSelected;
}


function updateTitle() {
    var validIds = getAllValid();
    var algs = validIds.length;
    var allSelector = document.getElementById('allSelector');
    if (isSearchActive()) {
        document.getElementById('allSelectorCount').innerText = validIds.filter(i => selCases.includes(i)).length;
    }
    if (validIds.every(i => selCases.includes(i)) && (algs > 0 || selCases.length == 0)) {
        allSelector.className = 'borderedContainer itemSel pad';
    } else {
        allSelector.className = 'borderedContainer itemUnsel pad';
    }
    for (const [algset, isShown] of Object.entries(selectedAlgSets)) {
        if (isShown && Object.keys(selectedAlgSets).length > 1) {
            document.getElementById(`${algset}Selector`).className = `borderedContainer ${isAlgsetAllSelected(algset) ? "itemSel" : "itemUnsel"} pad`
            document.getElementById(`${algset}csi`).innerText = countAlgsetSelected(algset);
        }
    }
    var csi = document.getElementById("csi");
    if (csi) csi.innerHTML = selCases.length;
    updateProbabilities();
}

/// shows the current chance of each selected case being chosen next
function updateProbabilities() {
    const show = currentSettings['showProbabilities'];
    const probabilities = show ? computeCaseProbabilities(selCases) : [];
    for (const el of document.querySelectorAll('.caseProbability')) {
        el.textContent = '';
    }
    selCases.forEach((c, idx) => {
        const el = document.getElementById('prob' + c);
        if (el && show) el.textContent = (probabilities[idx] * 100).toFixed(1) + '%';
    });
}

function setSelectionStatus(i, index, active) {
    if (!active && index < 0) {
        return;
    } 
    if (active && index >= 0) {
        return;
    }
    if (!active) {
        selCases.splice(index, 1);
    } else {
        selCases.push(i);
    }
    var element = document.getElementById("itemTd" + i);
    element.className = (active ? "itemSel" : "itemUnsel") + " borderedContainer caseItem";
    var groupElement = element.parentElement.previousElementSibling;
    var groupWasSelected = groupElement.classList[1] == 'itemSel';
    if (groupWasSelected && !active) {
        groupElement.className = 'borderedContainer itemUnsel pad groupNameDiv';
    }
    if (!groupWasSelected && active) {
        var groupElements = element.parentElement.childNodes;
        var selectedCount = 0;
        for (var i = 0; i < groupElements.length; i++) {
            selectedCount += groupElements[i].classList[0] == 'itemSel';
        }
        if (selectedCount == groupElements.length) {
            groupElement.className = 'borderedContainer itemSel pad groupNameDiv';
        }
    }
    return active;
}

function getBldInverseCase(caseId) {
    var algset = algsInfo[caseId]['algset'];
    var alg_name = algsInfo[caseId]['name'];
    var inverse_group = `${algset} ${alg_name[1]}`;
    var group_candidates = algsGroups[inverse_group];
    for (inverseCaseId of group_candidates) {
        if (algsInfo[inverseCaseId]['name'][1] == alg_name[0]) {
            var selCaseIndex = selCases.indexOf(inverseCaseId);
            return [inverseCaseId, selCaseIndex];
        }
    }
    return [null, null];
}

function matchInverseSelectionIfNecessary(caseId, matchGoal) {
    if (trainerTitle.includes("BLD")) {
        var inverse = getBldInverseCase(caseId);
        setSelectionStatus(inverse[0], inverse[1], matchGoal);
    }
}

function itemClicked(i) {
    if (window.scramblesMap[i] == null) {
        console.error("is null");
        return;
    }
    var index = selCases.indexOf(i);
    var wasSelected = (index >= 0); // if it was selected the index will be >= 0
    setSelectionStatus(i, index, !wasSelected); // if it was seleted, we want to deselect, otherwise select
    matchInverseSelectionIfNecessary(i, !wasSelected);

    saveSelection();
    updateTitle();
    return !wasSelected;
}

function selectAllNone() {
    var validAlgs = getAllValid();
    var allSelected = validAlgs.length > 0 && validAlgs.every(i => selCases.includes(i));
    if (isSearchActive()) {
        if (allSelected) {
            selCases = selCases.filter(i => !validAlgs.includes(i));
        } else {
            selCases = [...new Set([...selCases, ...validAlgs])];
        }
    } else if (!allSelected) {
        selCases = [...validAlgs];
    } else {
        selCases = [];
    }
    renderSelection();
    saveSelection();
    resize();
}

/// \returns true if at least one case selected in group groupName
function areAllSelected(groupName) {
    var indeces = getGroupIds(groupName);
    for (var i in indeces) {
        if (selCases.indexOf(indeces[i]) == -1)
            return false;
    }
    return true;
}

// select or deselect all cases in the group
function selectCaseGroup(name) {
    var allSelected = areAllSelected(name);
    var indeces = getGroupIds(name);
    var firstChild = document.getElementById(`itemTd${indeces[0]}`);
    var groupNameDiv = firstChild.parentElement.previousSibling;
    for (var i = 0; i < indeces.length; i++) {
        var j = selCases.indexOf(indeces[i]);
        setSelectionStatus(indeces[i], j, !allSelected);
        matchInverseSelectionIfNecessary(indeces[i], !allSelected);
    }
    if (allSelected) {
        groupNameDiv.className = 'borderedContainer itemUnsel pad groupNameDiv';
    } else {
        groupNameDiv.className = 'borderedContainer itemSel pad groupNameDiv'
    }
    saveSelection();
    updateTitle();
}

var touchholdtimer;
var touchduration = 1000; //length of time we want the user to touch before we do something

function touchstart(e, func) {
    // e.preventDefault();
    if (!touchholdtimer) {
        touchholdtimer = setTimeout(func, touchduration);
    }
}

function touchend(i) {
    //stops short touches from firing the event
    if (touchholdtimer) {
        clearTimeout(touchholdtimer);
        // itemClicked(i);
        touchholdtimer = null;
    }
}

function makeDivNormal(groupname) {
    const namedTrainers = [
        "3×3 PLL Trainer",
        "OH PLL Trainer",
        "FTO TCP Trainer",
        "FTO L3T Trainer",
        "FTO LL Trainer",
        "FTO L6X Trainer",
        "FTO LL Trainer",
        "FTO FTLT Trainer",
        "Square-1 CPEP Trainer",
        "Square-1 OBL Trainer",
        "Square-1 PBL Trainer",
    ];
    const includeNames = namedTrainers.includes(trainerTitle);
    var s = "";
    var indeces = getGroupIds(groupname);
    if (indeces.length == 0) {
        return "";
    }
    var displayGroupname = translateAlgGroup(groupname) 

    s += ` onclick='selectCaseGroup("${groupname}")'><b>${displayGroupname}</b></div><div class='rowFlex' style='flex-wrap: wrap'>`
    var allSelected = true;
    for (var j = 0; j < indeces.length; j++) {
        var i = indeces[j]; // case number
        var sel = (selCases.indexOf(i) != -1);
        var dblclick =` oncontextmenu='showHint(this, ${i}); return false;' `;
        if (!isMobile()) {
            dblclick += ` ondblclick='showHint(this, ${i})'`;
        }
        allSelected &= sel;

        var alg_name = getAlgName(i);
        if (trainerTitle == "Square-1 PBL Trainer" || trainerTitle.includes("BLD") ) {
            var content = `<span class='caseSpan'>${alg_name}</span>`;
        } 
        else if (includeNames) {
            var caseSpan = `<span class='caseSpan'>${alg_name}</span>`
            if (trainerTitle.includes('OBL')) {
                var nameParts = alg_name.split("|");
                caseSpan =  `<span class='caseSpan'>${nameParts[0]}</span><span class='caseSpan'>${nameParts[1]}</span>`
            }
            var content = `${caseSpan}<img oncontextmenu='return false;' class='caseImage' id='sel${i}' src='${blobUrls[i]}'>`;
        } 
        else {
            var content = `<img oncontextmenu='return false;' class='caseImage' id='sel${i}' src='${blobUrls[i]}'>`;
        }
        content += `<span class='caseProbability' id='prob${i}'></span>`;
        s += `<div id='itemTd${i}' ${dblclick} onclick='itemClicked(${i})' class='${(sel ? "itemSel" : "itemUnsel")} borderedContainer caseItem' title='${alg_name}' name='${alg_name}'>${content}</div>`
    }
    s = "<div class='colFlex' style='width: fit-content'> <div class='borderedContainer " + (allSelected ? "itemSel" : "itemUnsel") + " pad groupNameDiv'" + s;
    s += "</div></div>";
    return s;
}

function ensureSelectionMatchesShown() {
    var algs = getAllValid(true);
    var newSelected = selCases.filter((value) => { return algs.includes(value); })
    selCases = newSelected;
}



function selectAlgset(algset) {
    var algsetIds = getAlgsetIds(algset);

    var selectedCount = 0;
    for (const idx of algsetIds) {
        selectedCount += selCases.includes(idx);
    }
    var allSelected = selectedCount == algsetIds.length;
    for (const i of algsetIds) {
        var j = selCases.indexOf(i);
        if (allSelected && j != -1) { // need to delete
            selCases.splice(j, 1);
        }
        if (!allSelected && j == -1) { // need to add
            selCases.push(i);
        }
    }
    selectedAlgSets[algset] = selectedCount != 0 | !selectedAlgSets[algset];
    renderSelection();
    saveSelection();
    resize();
}

function makeAlgsetTitle(algset, enabled) {
    // var width = Math.max((95 / Object.keys(algsets).length), 20) + "%";
    return `<div id='${algset}Selector' class='borderedContainer\
     ${(enabled ? "itemSel" : "itemUnsel")} pad' 
     style='width: 7em; opacity: ${enabled ? 1.0 : 0.5}' 
     onclick='selectAlgset("${algset}")'><b>${algset} 
     (<span id='${algset}csi'>${enabled ? "0" : "-"}</span>/${getAlgsetIds(algset).length})</b></div>`;
}

function addPreset(name_) {
    var name = name_;
    if (name_ === null) {
        name = document.getElementById('newPresetInput').value;
    }
    if (name === '') {
        alert('Your preset must have a name.')
        return;
    }
    if (selectionPresets[name] != null) {
        addPreset(name + '1');
        return;
    }
    selectionPresets[name] = {
        'selCases': [],
        'selectedAlgSets': {}
    };
    Object.assign(selectionPresets[name]['selCases'], selCases);
    Object.assign(selectionPresets[name]['selectedAlgSets'], selectedAlgSets);
    saveLocal(selectionArrayKey + 'Presets', JSON.stringify(selectionPresets));
    renderPresets();
}

function updatePreset(name) {
    if (window.confirm(`Are you sure you want to set preset ${name} to the current selection?`)) {
        selectionPresets[name] = {
            'selCases': [],
            'selectedAlgSets': {}
        };
        Object.assign(selectionPresets[name]['selCases'], selCases);
        Object.assign(selectionPresets[name]['selectedAlgSets'], selectedAlgSets);
        saveLocal(selectionArrayKey + 'Presets', JSON.stringify(selectionPresets));
    }
}

function deletePreset(name) {
    if (window.confirm(`Are you sure you want to delete preset ${name}?`)) {
        delete selectionPresets[name];
        saveLocal(selectionArrayKey + 'Presets', JSON.stringify(selectionPresets));
        renderPresets();
    }
}

function ensureSelCasesIncludeInverseWhenBlind() {
    if (trainerTitle.includes("BLD")) {
        var withInverse = []
        for (const caseId of selCases) {
            if (!withInverse.includes(caseId)) {
                withInverse.push(caseId);
            }
            inverse = getBldInverseCase(caseId)[0];
            if (!withInverse.includes(inverse)) {
                withInverse.push(inverse)
            }
        }
        selCases = [...withInverse];
    }
}

function ensureAlgsetsAreSelected() {
    for (const i of selCases) {
        selectAlgsets[algsInfo[i].algset] = true;
    }
}

function presetsPost(name) {
    ensureSelCasesIncludeInverseWhenBlind();
    Object.assign(selectedAlgSets, selectionPresets[name]['selectedAlgSets']);
    renderSelection();
    saveSelection();
}

function usePreset(name) {
    selCases = [...selectionPresets[name]['selCases']];
    presetsPost(name);
}

function addPresetToSelection(name) {
    selCases.push(...selectionPresets[name]['selCases']);
    selCases = [...new Set(selCases)];
    presetsPost(name);
}

function removePresetFromSelection(name) {
    var selCaseSet = new Set(selCases);
    var presetSet = new Set(selectionPresets[name]['selCases'])
    selCases = [...selCaseSet.difference(presetSet)];
    presetsPost(name);
}


function createPresetActionButton(label, handler, name, limit_width) {
    var button = document.createElement('span');
    button.className = 'abutton';
    button.textContent = label;
    if (limit_width) {
        button.style.width = '1em';
    }
    button.addEventListener('click', function () {
        handler(name);
    });
    return button;
}

function renderPresets() {
    var previousText = '';
    var presetInput = null;
    try {
        presetInput = document.getElementById('newPresetInput');
        previousText = presetInput.value;
    } catch (e) {}

    var presetsContainer = document.getElementById('presetsSettings');
    if (!presetsContainer) {
        return;
    }

    presetsContainer.replaceChildren();

    for (const [name] of Object.entries(selectionPresets)) {
        var row = document.createElement('div');
        row.className = 'settingsEntry';

        var label = document.createElement('span');
        label.textContent = name;
        row.appendChild(label);

        var buttonGroup = document.createElement('div');
        buttonGroup.className = 'plusMinus';
        buttonGroup.appendChild(createPresetActionButton('+', addPresetToSelection, name, true));
        buttonGroup.appendChild(createPresetActionButton('-', removePresetFromSelection, name, true));
        buttonGroup.appendChild(createPresetActionButton('Del', deletePreset, name));
        buttonGroup.appendChild(createPresetActionButton('Set', updatePreset, name));
        buttonGroup.appendChild(createPresetActionButton('Use', usePreset, name));
        row.appendChild(buttonGroup);

        presetsContainer.appendChild(row);
    }

    var inputRow = document.createElement('div');
    inputRow.className = 'settingsEntry';

    var input = document.createElement('input');
    input.type = 'text';
    input.id = 'newPresetInput';
    input.value = previousText;
    input.style.width = '100%';
    input.placeholder = 'New Preset';
    inputRow.appendChild(input);

    var addButton = document.createElement('span');
    addButton.className = 'abutton';
    addButton.textContent = 'Add';
    addButton.addEventListener('click', function () {
        addPreset(null);
    });
    inputRow.appendChild(addButton);

    presetsContainer.appendChild(inputRow);
}


/// iterates the scramblesMap and highlights HTML elements according to the selection
function renderSelection() {
    var groups = getAllValidGroups();
    var algs = getAllValid().length;
    var s = "";
    var searching = isSearchActive();
    s += `<div class='rowFlex' style='align-items: stretch; gap: 0.3em;'><div id='allSelector' class='borderedContainer  ${(selCases.length == algs ? "itemSel" : "itemUnsel")} pad' style='flex-grow: 1;' onclick='selectAllNone()'><b>${searching ? "Search Results" : "All Cases"} (<span id='${searching ? "allSelectorCount" : "csi"}'></span>/${algs})</b></div>`;
    var tabStyle = searchState.open
        ? "background-color: var(--backgroundDarker); color: var(--accent); border-color: transparent; border-bottom-left-radius: 0; border-bottom-right-radius: 0; box-shadow: 0 0.2em 0 var(--backgroundDarker); position: relative;"
        : "";
    s += `<div class='borderedContainer pad ${searching ? "itemSel" : "itemUnsel"}' style='display: flex; flex-direction: column; justify-content: center; padding-left: 1em; padding-right: 1em; ${tabStyle}' title='Search' onclick='toggleSearchPanel()'><span class='material-symbols-outlined'>search</span></div></div>`;
    s += "<div id='searchSlot'></div>";
    if (Object.keys(selectedAlgSets).length > 1) {
        s += "<div class='rowFlex' style='flex-wrap: wrap;'>"
        for (const [algset, isShown] of Object.entries(selectedAlgSets)) {
            s += makeAlgsetTitle(algset, isShown);
        }
        s += "</div>"
    }

    for (const [algset, isShown] of Object.entries(selectedAlgSets)) {
        if (isShown) {
            if (Object.keys(selectedAlgSets).length > 1) {
                s += `<div class="borderedContainer pad" style='background-color: var(--backgroundDarker); color: var(--text);'><b>${algset}</b></div>`
            }
            for (const key of algsets[algset]) {
                s += makeDivNormal(key)
            }
        }
    }

    document.getElementById("cases_selection").innerHTML = s;
    mountSearchPanel();
    ensureSelectionMatchesShown();
    updateTitle();
    renderPresets();
}


function saveSelection() {
    saveLocal(selectionArrayKey, JSON.stringify(selCases));
    saveLocal(selectionArrayKey + "AlgSets", JSON.stringify(selectedAlgSets));
}

function loadSelection() {
    var cases = loadLocal(selectionArrayKey);
    if (cases != null)
        selCases = JSON.parse(cases);
    var loadedAlgSets = loadLocal(selectionArrayKey + "AlgSets");
    if (loadedAlgSets != null)  {
        tempAlgset = JSON.parse(loadedAlgSets);
        for (const [algset, isShown] of Object.entries(selectedAlgSets)) {
            if (tempAlgset[algset]) {
                selectedAlgSets[algset] = tempAlgset[algset];
            }
        }
    }
    var preset = loadLocal(selectionArrayKey + 'Presets')
    if (preset != null) {
        selectionPresets = JSON.parse(preset);
    }
}


// ---- Search ----
var searchState = {text: '', cells: {}, open: false, color: null, allAngles: true};
var searchMatches = null; // Set of case ids matching the current search, or null when inactive
var searchPanel = null;
var caseFillsCache = {};
var caseAnglesCache = {};
// Cycles of sticker positions for a U move, loaded from u_move.json (null if the trainer has none).
// A sticker at a position moves to the next position in its cycle.
var uMoveCycles = null;

function isSearchActive() {
    return searchMatches !== null;
}

function getGroupIds(group) {
    var ids = algsGroups[group] || [];
    return searchMatches ? ids.filter(i => searchMatches.has(i)) : ids;
}

function searchColorNames() {
    return Object.keys(defaultSettings.cubecolors).filter(c => c != 'black');
}

function getCaseFills(caseId) {
    if (!caseFillsCache[caseId]) {
        var svg = combined[caseId] || '';
        caseFillsCache[caseId] = [...svg.matchAll(/<polygon[^>]*?fill=['"]([^'"]+)['"]/g)].map(m => m[1].toLowerCase());
    }
    return caseFillsCache[caseId];
}

function hasUMove() {
    return Array.isArray(uMoveCycles) && uMoveCycles.length > 0;
}

function applyUMove(fills) {
    var moved = fills.slice();
    for (const cycle of uMoveCycles) {
        for (var k = 0; k < cycle.length; k++) {
            moved[cycle[(k + 1) % cycle.length]] = fills[cycle[k]];
        }
    }
    return moved;
}

function uMoveOrder() {
    var gcd = (a, b) => b ? gcd(b, a % b) : a;
    return uMoveCycles.reduce((order, cycle) => order * cycle.length / gcd(order, cycle.length), 1);
}

/// the fills of a case after 0, 1, ... U moves, until the original position is reached again
function getCaseAngles(caseId) {
    if (!caseAnglesCache[caseId]) {
        var angles = [getCaseFills(caseId)];
        for (var k = 1; k < uMoveOrder(); k++) {
            angles.push(applyUMove(angles[k - 1]));
        }
        caseAnglesCache[caseId] = angles;
    }
    return caseAnglesCache[caseId];
}

function searchTemplateKey() {
    var best = null, bestCount = -1;
    for (const key of Object.keys(combined)) {
        var count = getCaseFills(key).length;
        if (count > bestCount) { best = key; bestCount = count; }
    }
    return best;
}

function fuzzyMatches(query, text) {
    var q = query.toLowerCase().replace(/\s+/g, '');
    var t = text.toLowerCase().replace(/\s+/g, '');
    var pos = 0;
    for (const ch of q) {
        pos = t.indexOf(ch, pos);
        if (pos < 0) return false;
        pos++;
    }
    return true;
}

function caseMatchesSearch(caseId) {
    var info = algsInfo[caseId];
    if (searchState.text.trim() && !fuzzyMatches(searchState.text, `${info.algset}${info.group}${info.name}`)) {
        return false;
    }
    var angles = searchState.allAngles && hasUMove() ? getCaseAngles(caseId) : [getCaseFills(caseId)];
    return angles.some(fillsMatchPattern);
}

// Search colours are abstract labels: positions sharing a label must share a case colour,
// and different labels must map to different case colours.
function fillsMatchPattern(fills) {
    var labelToColor = {};
    var usedColors = new Set();
    for (const [idx, label] of Object.entries(searchState.cells)) {
        var fill = fills[idx];
        if (fill === undefined) return false;
        if (labelToColor[label] === undefined) {
            if (usedColors.has(fill)) return false;
            labelToColor[label] = fill;
            usedColors.add(fill);
        } else if (labelToColor[label] !== fill) {
            return false;
        }
    }
    return true;
}

function updateSearchMatches() {
    if (!searchState.text.trim() && Object.keys(searchState.cells).length == 0) {
        searchMatches = null;
    } else {
        searchMatches = new Set(Object.keys(algsInfo).map(Number).filter(caseMatchesSearch));
    }
}

function applySearch() {
    updateSearchMatches();
    var active = document.activeElement;
    var refocus = active && active.id == 'searchTextInput';
    var cursor = refocus ? active.selectionStart : 0;
    renderSelection();
    if (refocus) {
        var input = document.getElementById('searchTextInput');
        input.focus();
        input.setSelectionRange(cursor, cursor);
    }
    resize();
}

function toggleSearchPanel() {
    searchState.open = !searchState.open;
    renderSelection();
    resize();
}

function clearSearch() {
    searchState.text = '';
    searchState.cells = {};
    applySearch();
}

function buildSearchPanel() {
    var panel = document.createElement('div');
    panel.className = 'borderedContainer pad colFlex';
    panel.style.backgroundColor = 'var(--backgroundDarker)';
    panel.style.borderTopRightRadius = '0';
    panel.style.gap = '0.5em';
    panel.style.alignItems = 'center';

    var input = document.createElement('input');
    input.type = 'text';
    input.id = 'searchTextInput';
    input.placeholder = 'Search algset, group and name';
    input.style.flexGrow = '1';
    input.style.minWidth = '0';
    input.style.padding = '0.5em';
    input.value = searchState.text;
    input.addEventListener('input', () => {
        searchState.text = input.value;
        applySearch();
    });

    var key = searchTemplateKey();
    var imageContainer = document.createElement('div');
    imageContainer.className = 'borderedContainer itemUnsel pad';
    imageContainer.style.position = 'relative';
    imageContainer.style.cursor = 'default';
    imageContainer.style.maxWidth = '100%';
    imageContainer.style.boxSizing = 'border-box';
    var box = document.createElement('div');
    box.style.width = '16em';
    box.style.maxWidth = '100%';
    var doc = new DOMParser().parseFromString(combined[key], 'image/svg+xml');
    var svg = document.importNode(doc.documentElement, true);
    var colors = activeCubeColors();
    svg.querySelectorAll('polygon').forEach((poly, idx) => {
        var color = searchState.cells[idx];
        poly.setAttribute('fill', color ? colors[color] : '#222');
        poly.style.cursor = 'pointer';
        poly.addEventListener('click', () => {
            if (searchState.cells[idx] == searchState.color) {
                delete searchState.cells[idx];
            } else {
                searchState.cells[idx] = searchState.color;
            }
            applySearch();
        });
    });
    box.appendChild(svg);
    imageContainer.appendChild(box);
    var clearImage = document.createElement('span');
    clearImage.className = 'material-symbols-outlined';
    clearImage.textContent = 'close';
    clearImage.title = 'Clear image';
    clearImage.style.cssText = 'position: absolute; right: 0.2em; bottom: 0.2em; font-size: 1.2em; cursor: pointer;';
    clearImage.addEventListener('click', () => {
        searchState.cells = {};
        applySearch();
    });
    imageContainer.appendChild(clearImage);
    panel.appendChild(imageContainer);

    if (hasUMove()) {
        var angleRow = document.createElement('div');
        angleRow.className = 'rowFlex';
        angleRow.style.cssText = 'align-items: center; gap: 0.5em;';
        var angleLabel = document.createElement('span');
        angleLabel.textContent = 'Search all angles';
        var angleSwitch = document.createElement('label');
        angleSwitch.className = 'switch';
        var angleInput = document.createElement('input');
        angleInput.type = 'checkbox';
        angleInput.checked = searchState.allAngles;
        angleInput.addEventListener('change', () => {
            searchState.allAngles = angleInput.checked;
            applySearch();
        });
        var angleSlider = document.createElement('span');
        angleSlider.className = 'slider round';
        angleSwitch.append(angleInput, angleSlider);
        angleRow.append(angleLabel, angleSwitch);
        panel.appendChild(angleRow);
    }

    var palette = document.createElement('div');
    palette.className = 'rowFlex';
    palette.style.flexWrap = 'wrap';
    palette.style.justifyContent = 'center';
    palette.style.gap = '0.3em';
    for (const name of searchColorNames()) {
        var swatch = document.createElement('div');
        swatch.title = name;
        swatch.style.cssText = `width: 1.8em; height: 1.8em; border-radius: 0.3em; cursor: pointer; background: ${colors[name]}; border: 3px solid ${name == searchState.color ? 'var(--accent)' : 'var(--text)'}; box-sizing: border-box;`;
        swatch.addEventListener('click', () => {
            searchState.color = name;
            renderSelection();
        });
        palette.appendChild(swatch);
    }
    panel.appendChild(palette);

    var bottomRow = document.createElement('div');
    bottomRow.className = 'rowFlex';
    bottomRow.style.cssText = 'width: 100%; align-items: center; justify-content: space-between; gap: 0.5em;';
    bottomRow.appendChild(input);

    var clear = document.createElement('span');
    clear.className = 'material-symbols-outlined';
    clear.textContent = 'close';
    clear.title = 'Clear search';
    clear.style.cssText = 'color: var(--accent); cursor: pointer; font-size: 1.8em;';
    clear.addEventListener('click', clearSearch);
    bottomRow.appendChild(clear);
    panel.appendChild(bottomRow);
    return panel;
}

function mountSearchPanel() {
    var slot = document.getElementById('searchSlot');
    if (!slot || !searchState.open) return;
    if (!searchState.color) searchState.color = searchColorNames()[0];
    slot.replaceChildren(buildSearchPanel());
}
