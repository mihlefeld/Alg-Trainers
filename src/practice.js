recapArray = [];
/// \param m = mode: 0 = selection, 1 = practicing, 2 = recap
function changeMode(m) {
    if (window.history.state == 'select' && m != 'select' && m != null) {
        window.history.pushState(m, '', "?" + m);
    } else {
        window.history.replaceState(m, '', "?" + m);
    }
    // if (m == 'select') {
    //     window.history.replaceState(m, '', "?" + m);
    // }
    showMode(m);
}

function showMode(m)
{
    if (window.running) {
        timerAbort();
    }
    if (m == null) {
        m = 'select'
    }
    var pr = document.getElementById("practiceLayout");
    var se = document.getElementById('selectionLayout');
    document.getElementById('bodyid').style.background = m == 'select' ? 'var(--background)' : 'var(--backgroundDarker)'; 
    pr.style.display = (m != 'select') ? 'flex' : 'none';
    se.style.display = (m == 'select') ? 'flex' : 'none';

    if (m == 'recap' || m == 'train') {
        document.documentElement.style.setProperty("overscroll-behavior", "none");
        document.documentElement.style.setProperty("overflow-x", "hidden");
    } else {
        document.documentElement.style.setProperty("overscroll-behavior", "auto");
        document.documentElement.style.setProperty("overflow-x", "auto");
    }

    if (m == 'recap') {
        // recap
        window.allowStartingTimer = true;
        var casesAmount = selCases.length;
        recapArray = selCases.slice();
        showScramble();
    }
    else if (m == 'train') {
        // practice
        window.allowStartingTimer = true;
        recapArray = [];
        showScramble();
    }
    else if (m == 'select') { 
        // select
        window.allowStartingTimer = false;
        recapArray = [];
        renderSelection();
    }
    resize('');
}

/// \returns random integer from 0 to h
function randomNum(h) {
    return Math.floor(Math.random() * h);
}

// binary search, from https://stackoverflow.com/questions/69335458/weighted-probability-random-choice-array
function find(arr, x , start=0, end=arr.length) {
    if(end < start) return -1;
    else if(end == start) return end;
    const mid = Math.floor((start + end) / 2);  
    if(arr[mid] === x) return mid+1;
    else if(arr[mid] < x) return find(arr, x, mid+1, end);
    else
      return find(arr, x, start, mid);
}

// weighted random choice from array "items". weights has to be the same length as items and weights have to be cumulative
// weights do not have to be normalized
function weightedRandomElement(items, weights) {
    return items[find(weights, Math.random()*weights[weights.length-1])];
};

const MAX_WEIGHT_RATIO = 5;

function isWeightedChoiceActive() {
    return currentSettings['weightedChoice'] || currentSettings['weightedChoiceTime'] || currentSettings['weightedChoiceStd'];
}

/// scales weights to a mean of 1 and raises small weights so that max <= MAX_WEIGHT_RATIO * min
function normalizeMetricWeights(weights) {
    const max = Math.max(...weights);
    if (!(max > 0)) return weights.map(() => 1);
    const floor = max / MAX_WEIGHT_RATIO;
    const clamped = weights.map(w => Math.max(Number.isFinite(w) ? w : 0, floor));
    const mean = clamped.reduce((a, b) => a + b, 0) / clamped.length;
    return clamped.map(w => w / mean);
}

/// \returns the probability of each case in \param cases being chosen next, based on the enabled weighting metrics
function computeCaseProbabilities(cases) {
    const n = cases.length;
    if (n == 0) return [];
    var combined = new Array(n).fill(1);
    if (isWeightedChoiceActive()) {
        const times = {};
        for (const c of cases) times[c] = [];
        for (const r of window.timesArray) {
            if (times[r["case"]] !== undefined) times[r["case"]].push(0.001 * r["ms"]);
        }
        const counts = cases.map(c => times[c].length);
        // unseen cases get the largest value of the metric until they are timed
        const fillMax = values => {
            const finite = values.filter(Number.isFinite);
            const max = finite.length ? Math.max(...finite) : 0;
            return values.map(v => Number.isFinite(v) ? v : max);
        };
        var metrics = [];
        if (currentSettings['weightedChoice'])
            metrics.push(counts.map(c => (c + 0.1) ** -0.8));
        if (currentSettings['weightedChoiceTime'])
            metrics.push(fillMax(cases.map(c => times[c].length ? meanArray(times[c]) : NaN)).map(m => m ** 0.8));
        if (currentSettings['weightedChoiceStd'])
            metrics.push(fillMax(cases.map(c => times[c].length > 1 ? meanStdArray(times[c]).std : NaN)));
        for (const metric of metrics) {
            const normalized = normalizeMetricWeights(metric);
            for (var i = 0; i < n; i++) combined[i] *= normalized[i];
        }
    }
    const total = combined.reduce((a, b) => a + b, 0);
    return combined.map(w => w / total);
}
