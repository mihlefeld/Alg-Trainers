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
