var baseUrl;

function resize(event) {
    if (window.history.state != 'select') {
        return;
    }
    var vpWidth = document.documentElement.clientWidth;
    var gnds = document.getElementsByClassName('groupNameDiv');
    var gap = parseFloat(getComputedStyle(gnds[0].nextSibling).gap);
    var maxWidth = document.getElementById('allSelector').getBoundingClientRect().width;
    var itemWidth = gnds[0].nextSibling.firstChild.getBoundingClientRect().width;
    maxWidth = Math.min(maxWidth, maxAlgsPerRow * (itemWidth + gap) - gap);
    document.getElementById('selectionLayout').style.maxWidth = maxAlgsPerRow * (itemWidth + gap) - gap + 0.04 + "px";
    for (var i = 0; i < gnds.length; i++) {
        var gnd = gnds[i];
        var num_items = Math.floor((maxWidth - itemWidth) / (itemWidth + gap) + 1e-4) + 1;
        var new_width = num_items * itemWidth + gap * (num_items - 1);
        gnd.style.maxWidth = new_width + "px";
    }


    var right = (vpWidth - maxWidth) / 2;
    var marginTop = parseFloat(getComputedStyle(document.getElementById('selectionLayout')).marginTop);
    var modeButtonsWidth = document.getElementById('modeButtons').offsetWidth;
    if (right > modeButtonsWidth * 1.5) {
        document.getElementById('settingsButton').style.transform = "translateX(calc(100% + " + marginTop + "px))";
        document.getElementById("modeButtons").style.transform = "translateX(calc(100% + " + marginTop + "px))";
    } else {
        document.getElementById('settingsButton').style.transform = "unset";
        document.getElementById("modeButtons").style.transform = "unset";
    }
    document.getElementById('modeButtons').style.right = right + 'px';
}

function main() {
    loadSettings();
    applySettings();
    timer = document.getElementById('timer');
    timer.innerHTML = "ready";
    document.getElementById("trainerTitle").innerHTML = trainerTitle + ' | <a href="../index.html">Back</a>';
    var splitUrl = window.location.href.split('?');
    baseUrl = splitUrl[0];
    var startState = splitUrl.length > 1 ? splitUrl[1] : 'select';

    window.addEventListener('popstate', (event) => {
        showMode(history.state);
    })

    /// handles keypup and keydown events. Starts timer etc.
    document.getElementById("bodyid").addEventListener("keydown", function (event) {
        if (dialogOpen) {
            if (event.code == "Escape") {
                dialogOpen = false;
                window.allowStartingTimer = true;
            }
            if (event.code == "ArrowLeft") {
                previousCase();
            } else if (event.code == 'ArrowRight') {
                nextCase();
            }
            return;
        }
        // delete hotkey - remove last
        if (event.code == "Delete" && !running) {
            if (!!event.shiftKey)
                confirmClear();
            else
                confirmRemLast();
            return;
        }

        if (event.target.tagName == "INPUT") {
            return;
        }

        if (event.code == "KeyH" && !running) {
            showHint(null, window.lastCase);
        }

        if (event.code == "KeyP" && !running) {
            showHint(null, lastCase);
        }

        if (!allowed || !window.allowStartingTimer)
            return; // preventing auto-repeat and empty scrambles
        if (event.code != "ShiftLeft") // shift
            allowed = false;

        if (running) {
            // stop timer on any button
            timerStop();
            return;
        }
        else if (event.code == "Space") {
            timerSetReady();
            return;
        }
    });

    /// keyup event for starting the timer
    document.getElementById("bodyid").addEventListener("keyup", function (event) {
        allowed = true;
        if (!window.allowStartingTimer)
            return; // preventing auto-repeat
        if (!running && !waiting && (event.code == "Space")) {
            timerStart();
        }
        else {
            timerAfterStop();
        }
    });

    timerDiv = document.getElementById("timerDiv")
    timerDiv.addEventListener("touchstart", handleTouchStart, false);
    timerDiv.addEventListener("touchend", handleTouchEnd, false);
    window.addEventListener('keydown', function (e) {
        if (e.code == 'Space' && (e.target == document.body || e.target.tagName == "DIALOG")) {
            e.preventDefault();
        }
    });

    window.addEventListener('resize', resize);

    loadSelection();
    displayStats();
    window.history.replaceState('select', '', "?select")
    document.getElementById('bodyid').style.display = "flex";
    if (startState != "select" && startState != "") {
        changeMode(startState);
    } else {
        showMode('select');
    }
    document.body.style.display = "unset !important";
}

fetch("../template.html")
    .then((response) => response.text())
    .then((bodyHTML) => {
        loadSettings();
        applySettings();
        document.body.outerHTML = bodyHTML;
        window.requestAnimationFrame(() => {window.requestAnimationFrame(main)})
});