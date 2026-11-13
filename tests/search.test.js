const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');

function svg(fills) {
    return fills.map(fill => `<polygon fill='${fill}' points='0 0 1 0 1 1'/>`).join('');
}

function createContext(cases, uMove) {
    const context = {
        combined: Object.fromEntries(cases.map((fills, id) => [id, svg(fills)])),
        algsInfo: Object.fromEntries(cases.map((_, id) => [id, {algset: 'S', group: 'G', name: `N${id}`}])),
        algsGroups: {},
        defaultSettings: {cubecolors: {}},
    };
    vm.createContext(context);
    vm.runInContext(fs.readFileSync(`${__dirname}/../src/selection.js`, 'utf8'), context);
    context.uMoveCycles = uMove;
    return context;
}

function search(context, cells, allAngles) {
    context.searchState.cells = cells;
    context.searchState.allAngles = allAngles;
    context.updateSearchMatches();
    return [...context.searchMatches].sort();
}

test('pattern matches equivalent colours at the pattern positions only', () => {
    const context = createContext([
        ['#a', '#a', '#b', '#c'],
        ['#a', '#b', '#a', '#c'],
        ['#a', '#a', '#a', '#c'],
    ], null);
    assert.deepEqual(search(context, {0: 'x', 1: 'x'}, false), [0, 2]);
    assert.deepEqual(search(context, {0: 'x', 1: 'y'}, false), [1]);
});

test('all angles also matches cases that match after U moves', () => {
    const context = createContext([
        ['#a', '#a', '#b', '#c'],
        ['#b', '#c', '#a', '#a'],
        ['#a', '#b', '#c', '#d'],
    ], [[0, 1, 2, 3]]);
    assert.deepEqual(search(context, {0: 'x', 1: 'x'}, false), [0]);
    assert.deepEqual(search(context, {0: 'x', 1: 'x'}, true), [0, 1]);
});

test('stickers outside every cycle stay in place when the U move is applied', () => {
    const context = createContext([['#a', '#b', '#c', '#d']], [[0, 1, 2]]);
    assert.deepEqual([...context.applyUMove(['#a', '#b', '#c', '#d'])], ['#c', '#a', '#b', '#d']);
    assert.equal(context.uMoveOrder(), 3);
});
