var defaultColors = {
    'light': {
        '--text': "oklch(25% 0 0)",
        '--background': "oklch(96% 0.01 97)",
        '--secondary': "oklch(75% 0.09 99)",
        '--primary': "oklch(45% 0.08 136)",
        '--accent': "oklch(85% 0.16 98)"
    },
    "dark": {
        '--text': "oklch(99% 0 91)",
        '--background': "oklch(33% 0.0 0)",
        '--secondary': "oklch(69% 0.09 98)",
        '--primary': "oklch(82% 0.07 135)", 
        '--accent': "oklch(91% 0.16 98)"
    }  
}
var defaultSettings = {
    'baseSize':  isMobile() ? 1.0 : 1.0,
    'timerSize': 5.0,
    'scrambleSize': 2,
    'weightedChoice': true,
    'weightedChoiceTime' : false,
    'enableHoldDelay': true,
    'colors': {},
    'cubecolors': {
        "white": "#fafafa",
        "ignore": "#777",
        "black": "#222",
        "lightblue": "#A0CBE8",
        "blue": "#4E79A7",
        "red": "#E15759",
        "pink": "#FF9D9A",
        "beige": "#f4f3aa",
        "yellow": "#F1CE63",
        "green": "#59A14F",
        "lightgreen": "#8CD17D",
        "purple": "#B07AA1",
        "orange": "#F28E2B"
    },
    'selectedAlgSets': {},
    'trainerScrambleOverrides': {},
    'letterSchemeCorners': "abcdefghijklmnopqrstuvwx",
    'letterSchemeEdges': "abcdefghijklmnopqrstuvwx",
    'letterpairs': {},
    'useEdgesLetterPairs': false,
    'useCornersLetterPairs': false,
};

var defaultCubeColors = {
    'default': {...defaultSettings['cubecolors']},
    'redgreen': {
        "white": "#fafafa",
        "ignore": "#777",
        "black": "#222",
        "lightblue": "#A0CBE8",
        "blue": "#4E79A7",
        "red": "#E157BD",
        "pink": "#FF9AD5",
        "beige": "#F4F3AA",
        "yellow": "#F1CE63",
        "green": "#59A14F",
        "lightgreen": "#8CD17D",
        "purple": "#900067",
        "orange": "#9E6405"
    },
    'blueyellow': {
        "white": "#fafafa",
        "ignore": "#777",
        "black": "#222",
        "lightblue": "#A0E8E5",
        "blue": "#4E75A7",
        "red": "#E15759",
        "pink": "#FF9D9A",
        "beige": "#f4f3aa",
        "yellow": "#F1CE63",
        "green": "#59A14F",
        "lightgreen": "#8CD17D",
        "purple": "#B07AA1",
        "orange": "#F28E2B"
    }
}

var defaultCubeSchemeNames = {
    default: 'Default',
    redgreen: 'Red-Green color blind',
    blueyellow: 'Blue-Yellow color blind'
};
var editingCubeScheme = 'default';
var cubeTrainerList;

var currentSettings = {};
Object.assign(currentSettings, defaultSettings);
Object.assign(currentSettings['colors'], defaultColors[getColorScheme()]);
Object.assign(currentSettings['selectedAlgSets'], window.selectedAlgSets);

var defaultTrainerScrambleOptions = {
    preRotations: [...preRotations],
    postRotations: [...postRotations],
    preMoves: [...preMoves],
    postMoves: [...postMoves]
};
var trainerScrambleOptionNames = ['preRotations', 'postRotations', 'preMoves', 'postMoves'];


function loadSettings() {
    var loaded = loadLocal('ALLGSettings');
    if (loaded != null) {    
        currentSettings = JSON.parse(loaded);
        for (const [key, value] of Object.entries(defaultSettings)) {
            if (!(key in currentSettings)) {
                currentSettings[key] = value;
            }
        }
    };
    if (!currentSettings.cubeColorSchemes) {
        var oldColors = {...defaultCubeColors.default, ...currentSettings.cubecolors};
        var matchingPreset = Object.keys(defaultCubeColors).find(preset =>
            Object.keys(defaultCubeColors[preset]).every(key =>
                String(oldColors[key]).toLowerCase() === defaultCubeColors[preset][key].toLowerCase()));
        currentSettings.cubeColorSchemes = [];
        currentSettings.defaultCubeScheme = matchingPreset || 'legacy';
        currentSettings.trainerCubeSchemes = {};
        if (!matchingPreset) {
            currentSettings.cubeColorSchemes.push({id: 'legacy', name: 'Previous custom colors', colors: oldColors});
        }
        saveSettings();
    }
    if (!currentSettings.puzzleCubeSchemes) currentSettings.puzzleCubeSchemes = {};
}

function saveSettings() {
    saveLocal('ALLGSettings', JSON.stringify(currentSettings));
}


function clip(x, l, h) {
    return Math.min(Math.max(x, l), h);
}

function toSrgb(color) {
    return color.toGamut({'space': 'sRGB'});
}

function toOklchStr(color) {
    var s = 'oklch(';
    s += Math.round(100 * color.oklch.l) + '% ';
    s += Math.round(100 * color.oklch.c) / 100 + ' ';
    var h = Number.isNaN(color.oklch.h) ? 0 : color.oklch.h;
    s += Math.round(h) + ')'
    return s;
}

function computeColors() {
    var body = document.body;
    var contrastAlg = 'WCAG21';
    var minContrast = 3;
    var textColor = toSrgb(new Color(currentSettings.colors['--text']));
    var backgroundColor = toSrgb(new Color(currentSettings.colors['--background']));
    var secondaryColor = toSrgb(new Color(currentSettings.colors['--secondary']));
    var primaryColor = toSrgb(new Color(currentSettings.colors['--primary']));
    var accentColor = toSrgb(new Color(currentSettings.colors['--accent']));
    currentSettings.colors['--text'] = toOklchStr(textColor);
    currentSettings.colors['--background'] = toOklchStr(backgroundColor);
    currentSettings.colors['--secondary'] = toOklchStr(secondaryColor);
    currentSettings.colors['--primary'] = toOklchStr(primaryColor);
    currentSettings.colors['--accent'] = toOklchStr(accentColor);
    for (const [key, color] of Object.entries(currentSettings['colors'])) {
        try {
            document.getElementById(key).value = color;
        } catch (e) {}
        body.style.setProperty(key, color);
    }

    var contrastBackText = backgroundColor.contrast(textColor, contrastAlg);
    var contrastBacksecondary = backgroundColor.contrast(secondaryColor, contrastAlg);
    var contrastBackprimary = backgroundColor.contrast(primaryColor, contrastAlg);
    var contrastBackAccent = backgroundColor.contrast(accentColor, contrastAlg);
    var contrastTextsecondary = textColor.contrast(secondaryColor, contrastAlg);
    var contrastTextprimary = textColor.contrast(primaryColor, contrastAlg);
    var contrastTextAccent = textColor.contrast(accentColor, contrastAlg);
    var buttonText = contrastBacksecondary > contrastTextsecondary ? 'var(--background)' : "var(--text)";
    var primaryText = contrastBackprimary > contrastTextprimary ? "var(--background)" : "var(--text)";
    var accentText = contrastBackAccent > contrastTextAccent ? "var(--background)" : "var(--text)";
    body.style.setProperty('--secondaryText', buttonText);
    body.style.setProperty('--primaryText', primaryText);
    body.style.setProperty('--accentText', accentText);

    if (contrastBackprimary < minContrast) {
        var sign = primaryColor.oklch.l > backgroundColor.oklch.l ? 1 : -1;
        primaryColor.oklch.l += sign * 0.3;
    }
    body.style.setProperty('--primaryBG', toOklchStr(primaryColor));
    primaryColor.oklch.l += backgroundColor.oklch.l > primaryColor.oklch.l ? 0.1 : -0.1;
    body.style.setProperty('--primaryBGHover', toOklchStr(primaryColor));
    secondaryColor.oklch.l += backgroundColor.oklch.l > secondaryColor.oklch.l ? 0.1 : -0.1;
    body.style.setProperty('--secondaryHover', toOklchStr(secondaryColor));
    accentColor.oklch.l += backgroundColor.oklch.l > accentColor.oklch.l ? 0.1 : -0.1;
    body.style.setProperty('--accentHover', toOklchStr(accentColor));
    backgroundColor.oklch.l -= 0.1;
    body.style.setProperty('--backgroundDarker', toOklchStr(backgroundColor));
    try {
        document.getElementById('timer').style.color = toOklchStr(textColor);
    } catch(e) {}
}

function changeUiElementColor(target, color) {
    var contrastAlg = 'WCAG21';
    var color = toSrgb(new Color(color));
    var textColor = toSrgb(new Color(currentSettings.colors['--text']));
    var backgroundColor = toSrgb(new Color(currentSettings.colors['--background']));
    var contrastText = color.contrast(textColor, contrastAlg);
    var contrastBack = color.contrast(backgroundColor, contrastAlg);
    if (contrastText > contrastBack) {
        target.style.color = textColor.toString({format: "oklch"});
    } else {
        target.style.color = backgroundColor.toString({format: "oklch"});
    }
    target.style.backgroundColor = toOklchStr(color);
    target.value = toOklchStr(color);
}

function applySettings() {
    try {    
        document.getElementById('timer').style.fontSize = currentSettings['timerSize'] + "em";
        document.getElementById('scramble').style.fontSize = currentSettings['scrambleSize'] + "em";
        document.getElementById("hold_delay_on_off").checked = currentSettings['enableHoldDelay'];
        document.getElementById("weighted_choice_on_off").checked = currentSettings['weightedChoice'];
        document.getElementById("weighted_choice_time_on_off").checked = currentSettings['weightedChoiceTime']
        document.getElementById('cornerSchemeInput').value = currentSettings['letterSchemeCorners'];
        document.getElementById('edgeSchemeInput').value = currentSettings['letterSchemeEdges'];
        document.getElementById('edges_letter_pairs_on_off').checked = currentSettings['useEdgesLetterPairs'];
        document.getElementById('corners_letter_pairs_on_off').checked = currentSettings['useCornersLetterPairs'];
    } catch(e) {}
    applyTrainerScrambleOverrides();
    document.body.style.fontSize = currentSettings['baseSize'] + "em";
    computeColors();
}

function applyTrainerScrambleOverrides() {
    var trainerOverrides = currentSettings['trainerScrambleOverrides'][selectionArrayKey] || {};
    for (const key of trainerScrambleOptionNames) {
        var override = trainerOverrides[key];
        window[key] = Array.isArray(override) && override.length > 0
            ? [...override]
            : [...defaultTrainerScrambleOptions[key]];

        var input = document.getElementById(`trainerScramble-${key}`);
        if (input) {
            input.value = Array.isArray(override) ? override.join('\n') : '';
        }
    }
}

function changeTrainerScrambleOverride(event, key) {
    var value = event.target.value;
    var trainerOverrides = currentSettings['trainerScrambleOverrides'][selectionArrayKey];
    if (!trainerOverrides) {
        trainerOverrides = {};
        currentSettings['trainerScrambleOverrides'][selectionArrayKey] = trainerOverrides;
    }

    if (value.trim() === '') {
        delete trainerOverrides[key];
    } else {
        trainerOverrides[key] = value.split(/\r?\n/).map(option => option.trim());
    }
    if (Object.keys(trainerOverrides).length === 0) {
        delete currentSettings['trainerScrambleOverrides'][selectionArrayKey];
    }

    applyTrainerScrambleOverrides();
    saveSettings();
}


function adjustSize(item, inc) {
    if (item == 'timer') {
        currentSettings['timerSize'] += inc;
        document.getElementById('timer').style.fontSize = currentSettings['timerSize'] + "em";
    }

    if (item == 'scramble') {
        currentSettings['scrambleSize'] += inc;
        document.getElementById('scramble').style.fontSize = currentSettings['scrambleSize'] + "em";
    }
    if (item == 'body') {
        currentSettings['baseSize'] += inc
        document.body.style.fontSize = currentSettings['baseSize'] + "em";
        resize();
    }
    saveSettings();
}

function changeColor(event) {
    var newColor = event.target.value;
    var id = event.target.id;
    currentSettings['colors'][id] = newColor;
    document.body.style.setProperty(id, newColor);
    computeColors();
    saveSettings();
}

function changeCubeColor(event, key) {
    var scheme = currentSettings.cubeColorSchemes.find(item => item.id === editingCubeScheme);
    if (!scheme) return;
    try {
        changeUiElementColor(event.target, event.target.value);
    } catch (error) {
        alert('Please enter a valid color.');
        changeUiElementColor(event.target, scheme.colors[key]);
        return;
    }
    scheme.colors[key] = event.target.value;
    saveSettings();
    refreshCubeImages();
}

function changeSettingsKey(event, key) {
    currentSettings[key] = event.srcElement.value;
    saveSettings();
}

function cubeSchemeColors(id) {
    var custom = currentSettings.cubeColorSchemes.find(item => item.id === id);
    return custom ? custom.colors : defaultCubeColors[id] || defaultCubeColors.default;
}

function currentTrainerLocation() {
    var parts = window.location.pathname.split('/').filter(Boolean);
    if (parts[parts.length - 1] === 'index.html') parts.pop();
    return parts[parts.length - 1];
}

function activeCubeColors() {
    var trainer = currentTrainerLocation();
    var puzzle = Object.values(currentSettings.puzzleCubeSchemes).find(override => override.trainers.includes(trainer));
    var id = currentSettings.trainerCubeSchemes[trainer] || (puzzle && puzzle.scheme) || currentSettings.defaultCubeScheme;
    return cubeSchemeColors(id);
}

function cubeSchemeOptions(select, fallbackLabel) {
    select.replaceChildren();
    function option(value, label) {
        var element = document.createElement('option');
        element.value = value;
        element.textContent = label;
        select.appendChild(element);
    }
    if (fallbackLabel) option('', fallbackLabel);
    for (const [id, name] of Object.entries(defaultCubeSchemeNames)) option(id, name);
    for (const scheme of currentSettings.cubeColorSchemes) option(scheme.id, scheme.name);
}

function renderCubeSchemeSettings() {
    var global = document.getElementById('cube-scheme-global');
    var editor = document.getElementById('cube-scheme-editor');
    cubeSchemeOptions(global);
    cubeSchemeOptions(editor);
    global.value = currentSettings.defaultCubeScheme;
    editor.value = editingCubeScheme;
    var scheme = currentSettings.cubeColorSchemes.find(item => item.id === editingCubeScheme);
    var name = document.getElementById('cube-scheme-name');
    name.value = scheme ? scheme.name : defaultCubeSchemeNames[editingCubeScheme];
    name.disabled = !scheme;
    document.getElementById('cube-scheme-delete').disabled = !scheme;
    for (const [key, value] of Object.entries(cubeSchemeColors(editingCubeScheme))) {
        var input = document.getElementById(`cc-${key}`);
        if (input) {
            input.disabled = !scheme;
            changeUiElementColor(input, value);
        }
    }
    var list = document.getElementById('cube-trainers');
    if (!cubeTrainerList) return;
    list.replaceChildren();
    for (const [puzzle, entries] of Object.entries(cubeTrainerList)) {
        var group = document.createElement('div');
        group.className = 'cubeTrainerGroup';
        var heading = document.createElement('label');
        heading.className = 'settingsEntry cubeTrainerEntry cubePuzzleEntry';
        var title = document.createElement('b');
        title.textContent = puzzle;
        let puzzleSelect = document.createElement('select');
        puzzleSelect.setAttribute('aria-label', `${puzzle} puzzle scheme`);
        cubeSchemeOptions(puzzleSelect, 'Use global scheme');
        puzzleSelect.value = currentSettings.puzzleCubeSchemes[puzzle]?.scheme || '';
        puzzleSelect.addEventListener('change', () => {
            if (puzzleSelect.value) {
                currentSettings.puzzleCubeSchemes[puzzle] = {
                    scheme: puzzleSelect.value,
                    trainers: entries.map(entry => entry.location)
                };
            } else {
                delete currentSettings.puzzleCubeSchemes[puzzle];
            }
            saveSettings();
            if (entries.some(entry => entry.location === currentTrainerLocation())) refreshCubeImages();
        });
        heading.append(title, puzzleSelect);
        group.appendChild(heading);
        for (const entry of entries) {
            var row = document.createElement('label');
            row.className = 'settingsEntry cubeTrainerEntry';
            var label = document.createElement('span');
            label.textContent = entry.name;
            let select = document.createElement('select');
            select.setAttribute('aria-label', `${puzzle} ${entry.name} scheme`);
            cubeSchemeOptions(select, 'Use puzzle scheme');
            select.value = currentSettings.trainerCubeSchemes[entry.location] || '';
            select.addEventListener('change', () => {
                if (select.value) currentSettings.trainerCubeSchemes[entry.location] = select.value;
                else delete currentSettings.trainerCubeSchemes[entry.location];
                saveSettings();
                if (entry.location === currentTrainerLocation()) refreshCubeImages();
            });
            row.append(label, select);
            group.appendChild(row);
        }
        list.appendChild(group);
    }
}

async function openCubeSchemeSettings() {
    renderCubeSchemeSettings();
    if (cubeTrainerList) return;
    try {
        var response = await fetch('../index.json');
        if (!response.ok) throw new Error('Unable to load trainer list');
        cubeTrainerList = await response.json();
        var changed = false;
        for (const [puzzle, override] of Object.entries(currentSettings.puzzleCubeSchemes)) {
            if (!cubeTrainerList[puzzle]) continue;
            var trainers = cubeTrainerList[puzzle].map(entry => entry.location);
            if (JSON.stringify(override.trainers) !== JSON.stringify(trainers)) {
                override.trainers = trainers;
                changed = true;
            }
        }
        if (changed) {
            saveSettings();
            refreshCubeImages();
        }
        renderCubeSchemeSettings();
    } catch (error) {
        console.error('Unable to load trainer list:', error);
        document.getElementById('cube-trainers').textContent = 'Unable to load trainer list. Reopen settings to retry.';
    }
}

function selectGlobalCubeScheme(value) {
    currentSettings.defaultCubeScheme = value;
    saveSettings();
    refreshCubeImages();
}

function addCubeScheme() {
    var id;
    do {
        id = 'custom-' + Date.now().toString(36) + '-' + Math.random().toString(36).slice(2);
    } while (currentSettings.cubeColorSchemes.some(item => item.id === id));
    currentSettings.cubeColorSchemes.push({
        id: id,
        name: 'Custom scheme ' + (currentSettings.cubeColorSchemes.length + 1),
        colors: {...cubeSchemeColors(editingCubeScheme)}
    });
    editingCubeScheme = id;
    saveSettings();
    renderCubeSchemeSettings();
}

function renameCubeScheme(value) {
    var scheme = currentSettings.cubeColorSchemes.find(item => item.id === editingCubeScheme);
    if (!scheme) return;
    if (!value.trim()) {
        alert('Please enter a scheme name.');
        renderCubeSchemeSettings();
        return;
    }
    scheme.name = value.trim();
    saveSettings();
    renderCubeSchemeSettings();
}

function deleteCubeScheme() {
    var scheme = currentSettings.cubeColorSchemes.find(item => item.id === editingCubeScheme);
    if (!scheme || !confirm(`Delete "${scheme.name}"? Overrides using it will fall back to their inherited scheme.`)) return;
    currentSettings.cubeColorSchemes = currentSettings.cubeColorSchemes.filter(item => item.id !== scheme.id);
    if (currentSettings.defaultCubeScheme === scheme.id) currentSettings.defaultCubeScheme = 'default';
    for (const [trainer, id] of Object.entries(currentSettings.trainerCubeSchemes)) {
        if (id === scheme.id) delete currentSettings.trainerCubeSchemes[trainer];
    }
    for (const [puzzle, override] of Object.entries(currentSettings.puzzleCubeSchemes)) {
        if (override.scheme === scheme.id) delete currentSettings.puzzleCubeSchemes[puzzle];
    }
    editingCubeScheme = 'default';
    saveSettings();
    renderCubeSchemeSettings();
    refreshCubeImages();
}

function resetStyle(dark) {
    var body = document.body;
    Object.assign(currentSettings['colors'], defaultColors[dark]);
    for (const [key, value] of Object.entries(currentSettings['colors'])) {
        document.getElementById(key).value = value;
        body.style.setProperty(key, value);
    }
    computeColors();
    saveSettings();
}

function toggleSetting(element, key) {
    currentSettings[key] = !currentSettings[key];
    element.checked = currentSettings[key];
    saveSettings();
}

function uploadLetterPairs() {
    var files = document.getElementById('uploadLP').files;
    if (files.length != 1) {
        return false;
    }

    var fr = new FileReader();
    
    fr.onload = function(e) { 
        try {
            var result = JSON.parse(e.target.result);
            currentSettings['letterpairs'] = result;
            saveSettings();
            window.alert("Letter pair words changed successfully!")
        } catch (e) {
            console.error(e);
            window.alert("Something went wrong while parsing your letter pairs. The file should be a json mapping from your letter pair to a word.")
        }
        
    }
    fr.readAsText(files.item(0));
}