const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');

function fakeIndexedDB() {
    const stores = new Map();
    let failNextWrite = false;
    const database = {
        createObjectStore(name) { stores.set(name, new Map()); },
        close() {},
        transaction(names, mode) {
            const snapshots = new Map([...stores].map(([name, data]) => [name, new Map(data)]));
            const transaction = {
                error: null,
                pending: 0,
                aborted: false,
                abort() {
                    if (this.aborted) return;
                    this.aborted = true;
                    for (const [name, data] of snapshots) stores.set(name, data);
                    setImmediate(() => this.onabort?.());
                },
                enqueue(callback) {
                    this.pending++;
                    setImmediate(() => {
                        if (!this.aborted) callback();
                        this.pending--;
                        if (!this.pending && !this.aborted) setImmediate(() => {
                            if (!this.pending && !this.aborted) this.oncomplete?.();
                        });
                    });
                },
                objectStore(name) {
                    return {
                        get(key) {
                            const request = {};
                            transaction.enqueue(() => {
                                request.result = stores.get(name).get(key);
                                request.onsuccess?.();
                            });
                            return request;
                        },
                        put(value, key) {
                            const request = {};
                            transaction.enqueue(() => {
                                if (failNextWrite) {
                                    failNextWrite = false;
                                    transaction.error = new Error('write failed');
                                    transaction.abort();
                                    return;
                                }
                                stores.get(name).set(key, value);
                                request.onsuccess?.();
                            });
                            return request;
                        },
                        openCursor() {
                            const request = {};
                            const entries = [...stores.get(name)];
                            let index = 0;
                            function next() {
                                transaction.enqueue(() => {
                                    const entry = entries[index++];
                                    request.result = entry ? {
                                        key: entry[0], value: entry[1], continue: next
                                    } : null;
                                    request.onsuccess?.();
                                });
                            }
                            next();
                            return request;
                        }
                    };
                }
            };
            return transaction;
        }
    };
    return {
        stores,
        failWrite() { failNextWrite = true; },
        open() {
            const request = { result: database };
            setImmediate(() => {
                if (!stores.size) request.onupgradeneeded?.();
                request.onsuccess?.();
            });
            return request;
        }
    };
}

function setup(initial = {}) {
    const values = new Map(Object.entries(initial));
    const localStorage = {
        get length() { return values.size; },
        key(index) { return [...values.keys()][index]; },
        getItem(key) { return values.get(key) ?? null; },
        setItem(key, value) { values.set(key, value); }
    };
    const indexedDB = fakeIndexedDB();
    const alerts = [];
    const context = vm.createContext({
        indexedDB, localStorage, console, alert: message => alerts.push(message),
        setImmediate, Map, Promise
    });
    vm.runInContext(fs.readFileSync('src/utils.js', 'utf8'), context);
    return { context, indexedDB, localStorage, alerts };
}

test('migrates all legacy values once and persists subsequent changes', async () => {
    const legacy = {
        ALLGSettings: '{"baseSize":2}',
        '2x2egTimes': '[{"ms":1234}]',
        '2x2egSelectioncustomAlgs': '{"1":{"a":["R U"]}}',
        '2x2egSelectionPresets': '{"Bookmarks":{}}',
        'otherTrainerTimes': '[]'
    };
    const { context, indexedDB, localStorage } = setup(legacy);
    await context.initializeStorage();
    for (const [key, value] of Object.entries(legacy)) {
        assert.equal(context.loadLocal(key), value);
    }
    await context.saveLocal('2x2egTimes', '[{"ms":5678}]');
    localStorage.setItem('2x2egTimes', 'stale legacy value');
    context.storageCache.clear();
    await context.initializeStorage();
    assert.equal(context.loadLocal('2x2egTimes'), '[{"ms":5678}]');
    assert.equal(indexedDB.stores.get('metadata').get('localStorageMigrated'), true);
    assert.equal(localStorage.getItem('ALLGSettings'), legacy.ALLGSettings);
});

test('failed migration rolls back and can be retried', async () => {
    const { context, indexedDB } = setup({ ALLGSettings: '{"baseSize":2}' });
    indexedDB.failWrite();
    await assert.rejects(context.initializeStorage(), /write failed/);
    assert.equal(indexedDB.stores.get('metadata').has('localStorageMigrated'), false);
    assert.equal(indexedDB.stores.get('data').size, 0);
    await context.initializeStorage();
    assert.equal(context.loadLocal('ALLGSettings'), '{"baseSize":2}');
});

test('reports failed writes instead of silently losing changes', async () => {
    const { context, indexedDB, alerts } = setup();
    await context.initializeStorage();
    indexedDB.failWrite();
    await context.saveLocal('time', '123');
    assert.equal(alerts.length, 1);
    assert.equal(indexedDB.stores.get('data').has('time'), false);
});

test('exports IndexedDB data and imports legacy backups atomically', async () => {
    const { context, indexedDB, alerts } = setup({ old: 'legacy' });
    await context.initializeStorage();
    await context.saveLocal('new', 'indexedDB');

    let exported;
    let reloads = 0;
    let fileText;
    let readFinished;
    context.File = class {
        constructor(parts, name) { this.parts = parts; this.name = name; }
    };
    context.URL = {
        createObjectURL(file) { exported = file; return 'blob:test'; },
        revokeObjectURL() {}
    };
    context.window = { URL: context.URL };
    context.document = {
        getElementById() { return { files: { length: 1, item: () => fileText } }; },
        createElement() { return { click() {} }; },
        body: { appendChild() {}, removeChild() {} }
    };
    context.location = { reload() { reloads++; } };
    context.FileReader = class {
        readAsText(file) {
            readFinished = new Promise(resolve => setImmediate(async () => {
                await this.onload({ target: { result: file } });
                resolve();
            }));
        }
    };

    await context.downloadLocalStorage();
    assert.deepEqual(JSON.parse(exported.parts[0]), { old: 'legacy', new: 'indexedDB' });
    fileText = JSON.stringify({ old: 'restored', imported: '[1,2]' });
    context.uploadLocalStorage();
    await readFinished;
    assert.equal(reloads, 1);
    assert.equal(indexedDB.stores.get('data').get('old'), 'restored');
    assert.equal(indexedDB.stores.get('data').get('imported'), '[1,2]');
    assert.equal(indexedDB.stores.get('data').get('new'), 'indexedDB');

    fileText = JSON.stringify({ old: 'broken', imported: 12 });
    context.uploadLocalStorage();
    await readFinished;
    assert.equal(reloads, 1);
    assert.equal(alerts.length, 1);
    assert.equal(indexedDB.stores.get('data').get('old'), 'restored');

    indexedDB.failWrite();
    fileText = JSON.stringify({ old: 'not saved', another: 'value' });
    context.uploadLocalStorage();
    await readFinished;
    assert.equal(reloads, 1);
    assert.equal(alerts.length, 2);
    assert.equal(indexedDB.stores.get('data').get('old'), 'restored');
    assert.equal(indexedDB.stores.get('data').has('another'), false);
});

async function setupCubeSettings(legacySettings) {
    const state = setup(legacySettings ? { ALLGSettings: JSON.stringify(legacySettings) } : {});
    await state.context.initializeStorage();
    Object.assign(state.context, {
        window: { selectedAlgSets: {}, location: { pathname: '/Alg-Trainers/3x3-PLL-Trainer/index.html' } },
        isMobile: () => false,
        getColorScheme: () => 'light',
        preRotations: [''], postRotations: [''], preMoves: [''], postMoves: [''],
        selectionArrayKey: '3x3pllselection',
        confirm: () => true
    });
    vm.runInContext(fs.readFileSync('src/settings.js', 'utf8'), state.context);
    state.context.loadSettings();
    return state;
}

test('migrates a legacy palette globally and retains it across reloads', async () => {
    const { context, indexedDB } = await setupCubeSettings({
        cubecolors: { red: '#123456', blue: '#654321' },
        baseSize: 1.5
    });
    assert.equal(context.currentSettings.defaultCubeScheme, 'legacy');
    assert.equal(context.activeCubeColors().red, '#123456');
    assert.equal(context.activeCubeColors().white, '#fafafa');
    context.window.location.pathname = '/Alg-Trainers/2x2-EG-Trainer/';
    assert.equal(context.activeCubeColors().blue, '#654321');
    await new Promise(resolve => setTimeout(resolve, 20));
    const saved = JSON.parse(indexedDB.stores.get('data').get('ALLGSettings'));
    assert.equal(saved.baseSize, 1.5);
    assert.equal(saved.cubeColorSchemes[0].colors.red, '#123456');
    context.loadSettings();
    assert.equal(context.currentSettings.cubeColorSchemes.length, 1);
    assert.equal(context.activeCubeColors().red, '#123456');
});

test('migrates a legacy color-blind preset without creating a custom scheme', async () => {
    const { context } = await setupCubeSettings();
    const redgreen = JSON.parse(JSON.stringify(context.defaultCubeColors.redgreen));
    context.saveLocal('ALLGSettings', JSON.stringify({ cubecolors: redgreen }));
    context.loadSettings();
    assert.equal(context.currentSettings.defaultCubeScheme, 'redgreen');
    assert.equal(context.currentSettings.cubeColorSchemes.length, 0);
});

test('manages independent schemes and trainer overrides, including deletion', async () => {
    const { context } = await setupCubeSettings();
    context.renderCubeSchemeSettings = () => {};
    context.refreshCubeImages = () => {};
    context.addCubeScheme();
    const first = context.editingCubeScheme;
    context.renameCubeScheme('First');
    context.currentSettings.cubeColorSchemes[0].colors.red = '#123456';
    context.addCubeScheme();
    const second = context.editingCubeScheme;
    context.currentSettings.cubeColorSchemes[1].colors.red = '#abcdef';
    assert.equal(context.currentSettings.cubeColorSchemes[0].colors.red, '#123456');
    context.selectGlobalCubeScheme(first);
    context.currentSettings.trainerCubeSchemes['3x3-PLL-Trainer'] = second;
    context.currentSettings.puzzleCubeSchemes['3x3'] = { scheme: second, trainers: ['3x3-PLL-Trainer'] };
    assert.equal(context.activeCubeColors().red, '#abcdef');
    context.window.location.pathname = '/Alg-Trainers/2x2-EG-Trainer/index.html';
    assert.equal(context.activeCubeColors().red, '#123456');
    context.deleteCubeScheme();
    assert.equal(context.currentSettings.trainerCubeSchemes['3x3-PLL-Trainer'], undefined);
    assert.equal(context.currentSettings.puzzleCubeSchemes['3x3'], undefined);
    assert.equal(context.currentSettings.defaultCubeScheme, first);
    context.editingCubeScheme = first;
    context.deleteCubeScheme();
    assert.equal(context.currentSettings.defaultCubeScheme, 'default');
});

test('trainer controls assign and clear an override without affecting other trainers', async () => {
    const { context, indexedDB } = await setupCubeSettings();
    class Element {
        constructor() { this.children = []; this.value = ''; this.listeners = {}; }
        replaceChildren() { this.children = []; }
        appendChild(child) { this.children.push(child); }
        append(...children) { this.children.push(...children); }
        setAttribute(name, value) { this[name] = value; }
        addEventListener(name, callback) { this.listeners[name] = callback; }
    }
    const elements = {};
    for (const id of ['cube-scheme-global', 'cube-scheme-editor', 'cube-scheme-name',
        'cube-scheme-delete', 'cube-trainers']) elements[id] = new Element();
    for (const key of Object.keys(context.defaultCubeColors.default)) elements[`cc-${key}`] = new Element();
    context.document = {
        getElementById: id => elements[id],
        createElement: () => new Element()
    };
    context.changeUiElementColor = (element, value) => { element.value = value; };
    let refreshes = 0;
    context.refreshCubeImages = () => { refreshes++; };
    context.cubeTrainerList = {
        '3x3': [{ name: 'PLL', location: '3x3-PLL-Trainer' }, { name: 'OLL', location: '3x3-OLL-Trainer' }],
        '2x2': [{ name: 'EG', location: '2x2-EG-Trainer' }]
    };
    context.renderCubeSchemeSettings();
    const rows = elements['cube-trainers'].children[0].children;
    const puzzle = rows[0].children[1];
    const otherPuzzle = elements['cube-trainers'].children[1].children[0].children[1];
    const pll = rows[1].children[1];
    const oll = rows[2].children[1];
    assert.equal(pll['aria-label'], '3x3 PLL scheme');
    puzzle.value = 'redgreen';
    puzzle.listeners.change();
    assert.equal(context.activeCubeColors().red, context.defaultCubeColors.redgreen.red);
    assert.equal(otherPuzzle.value, '');
    assert.equal(refreshes, 1);
    otherPuzzle.value = 'blueyellow';
    otherPuzzle.listeners.change();
    assert.equal(context.currentSettings.puzzleCubeSchemes['2x2'].scheme, 'blueyellow');
    assert.equal(refreshes, 1);
    context.window.location.pathname = '/Alg-Trainers/3x3-OLL-Trainer/';
    assert.equal(context.activeCubeColors().red, context.defaultCubeColors.redgreen.red);
    context.window.location.pathname = '/Alg-Trainers/2x2-EG-Trainer/';
    assert.equal(context.activeCubeColors().blue, context.defaultCubeColors.blueyellow.blue);
    context.window.location.pathname = '/Alg-Trainers/3x3-PLL-Trainer/index.html';
    pll.value = 'blueyellow';
    pll.listeners.change();
    assert.equal(context.activeCubeColors().blue, context.defaultCubeColors.blueyellow.blue);
    assert.equal(oll.value, '');
    assert.equal(refreshes, 2);
    pll.value = '';
    pll.listeners.change();
    assert.equal(context.currentSettings.trainerCubeSchemes['3x3-PLL-Trainer'], undefined);
    assert.equal(context.activeCubeColors().red, context.defaultCubeColors.redgreen.red);
    assert.equal(refreshes, 3);
    await new Promise(resolve => setTimeout(resolve, 20));
    assert.equal(JSON.parse(indexedDB.stores.get('data').get('ALLGSettings')).puzzleCubeSchemes['3x3'].scheme, 'redgreen');
    context.loadSettings();
    assert.equal(context.activeCubeColors().red, context.defaultCubeColors.redgreen.red);
    puzzle.value = '';
    puzzle.listeners.change();
    assert.equal(context.activeCubeColors().red, context.defaultCubeColors.default.red);
});

test('refreshes stored puzzle membership when the trainer list changes', async () => {
    const { context } = await setupCubeSettings();
    context.currentSettings.puzzleCubeSchemes['3x3'] = { scheme: 'redgreen', trainers: ['3x3-OLL-Trainer'] };
    context.fetch = async () => ({
        ok: true,
        json: async () => ({ '3x3': [{ name: 'PLL', location: '3x3-PLL-Trainer' }] })
    });
    context.renderCubeSchemeSettings = () => {};
    let refreshes = 0;
    context.refreshCubeImages = () => { refreshes++; };
    await context.openCubeSchemeSettings();
    assert.equal(context.activeCubeColors().red, context.defaultCubeColors.redgreen.red);
    assert.equal(refreshes, 1);
    assert.deepEqual(Array.from(context.currentSettings.puzzleCubeSchemes['3x3'].trainers), ['3x3-PLL-Trainer']);
});

test('recolors existing images and revokes replaced blob URLs', async () => {
    const { context } = await setupCubeSettings();
    const blobs = new Map();
    const revoked = [];
    const image = { src: 'old-url' };
    Object.assign(context, {
        combined: { 1: "<svg fill='#E15759'/>" },
        document: { querySelectorAll: () => [image] },
        Blob: class { constructor(parts) { this.parts = parts; } },
        URL: {
            createObjectURL(blob) { const url = `new-${blobs.size}`; blobs.set(url, blob); return url; },
            revokeObjectURL(url) { revoked.push(url); }
        }
    });
    const main = fs.readFileSync('src/main.js', 'utf8').split('Promise.all([initializeStorage()')[0];
    vm.runInContext(main, context);
    context.blobUrls = { 1: 'old-url' };
    context.currentSettings.defaultCubeScheme = 'redgreen';
    context.refreshCubeImages();
    assert.equal(image.src, 'new-0');
    assert.match(blobs.get(image.src).parts[0], /#E157BD/);
    assert.deepEqual(revoked, ['old-url']);
});
