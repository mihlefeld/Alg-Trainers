const timesArrayKey = "l4eTimesArray";
const selectionArrayKey = "l4eSelection";
var trainerTitle = "Pyraminx L4E Trainer";
var preRotations = ['', "y", "y'", "x", "x y", "x z'", "z", "z'", "y z", "y z'"];
var postRotations = [''];
var preMoves = ["", "U", "U'"];
var postMoves = preMoves;
var maxAlgsPerRow = 8;
var selCases = [];
var selectedAlgSets = {
    "L4E": true,
}

var algsGroups = {
    "Last Layer": [1, 2, 3, 4],
    "Last Three Edges": [5, 6, 7, 8, 9, 10, 11, 12],
    "Flipped Edges": [13, 14, 15],
    "Polish Flip": [16, 17, 18, 19],
    "Separated Bar": [20, 21, 22, 23, 24, 25],
    "Connected Bar": [26, 27, 28, 29],
    "No bar": [30, 31, 32, 33, 34, 35]
};

var algsets = {
    "L4E": [
        "Last Layer",
        "Last Three Edges",
        "Flipped Edges",
        "Polish Flip",
        "Separated Bar",
        "Connected Bar",
        "No bar"
    ]
}

var algsInfo = {
    "1": {
        "name": "Sune",
        "a": [
            "L' U L U L' U L",
            "L U L' U L U L'",
            "R U R' U R U R'",
            "R' U R U R' U R"
        ],
        "s": "R U' R' U' R U' R'"
    },
    "2": {
        "name": "Anti-Sune",
        "a": [
            "R U' R' U' R U' R'",
            "R' U' R U' R' U' R",
            "L' U' L U' L' U' L",
            "L U' L' U' L U' L'"
        ],
        "s": "R U R' U R U R'"
    },
    "3": {
        "name": "Lefty Bars",
        "a": [
            "(U) R' U' L' U L R",
            "(U) U L R U R' U' L'",
            "(U) U' L U S U' L'",
            "(U) U' R' U' S U R"
        ],
        "s": "R U B U' B' R'"
    },
    "4": {
        "name": "Righty Bars",
        "a": [
            "L U R U' R' L'",
            "(U') R' L' U' L U R",
            "(U) L U H U' L'",
            "(U) R' U' H U R"
        ],
        "s": "R B U B' U' R'"
    },
    "5": {
        "name": "Sledge",
        "a": [
            "S = R' L R L'",
            "(U) L' U' L R U' R'"
        ],
        "s": "L' B' U B U' L"
    },
    "6": {
        "name": "Hedge",
        "a": [
            "H = L R' L' R",
            "(U') R U R' L' U L"
        ],
        "s": "L B L B' U' L"
    },
    "7": {
        "name": "Cycle+",
        "a": [
            "H R U' R'",
            "(U') R U' R' H",
            "(U) L' U' L H",
            "(U') H L' U' L",
            "R U' R' U' L' U' L"
        ],
        "s": "B R B R B' R B'"
    },
    "8": {
        "name": "Cycle-",
        "a": [
            "(U') S L' U L",
            "L' U L S",
            "(U) R U R' S",
            "S R U R'",
            "(U') L' U L U R U R'"
        ],
        "s": "B R' B R' B' R' B'"
    },
    "9": {
        "name": "Righty",
        "a": [
            "R U' R'"
        ],
        "s": "L U' B' U B L'"
    },
    "10": {
        "name": "Lefty",
        "a": [
            "L' U L"
        ],
        "s": "R B U' B' U' R'"
    },
    "11": {
        "name": "Sexy",
        "a": [
            "R U R'"
        ],
        "s": "L B' U' B U L'"
    },
    "12": {
        "name": "Ugly",
        "a": [
            "(U') L' U' L"
        ],
        "s": "R U B U B' R'"
    },
    "13": {
        "name": "2-Flip",
        "a": [
            "H U' R U R'",
            "S U L' U' L",
            "R' U L' U' L U' R",
            "L U' R U R' U L'",
            "(U') R' U L' U L U' R"
        ],
        "s": "L U' R U' R' U L'"
    },
    "14": {
        "name": "D-Flip",
        "a": [
            "S R U' R'",
            "(U) H L' U L",
            "(U') R U R' U L' U' L"
        ],
        "s": "L R' L' R L' U L"
    },
    "15": {
        "name": "4-Flip",
        "a": [
            "R U' R' L' U L X2",
            "L' U L R U' R' X2"
        ],
        "s": "R U' R' L' U B' U B L"
    },
    "16": {
        "name": "Right Polish Flip",
        "a": [
            "(U') R U' R' L' U' L",
            "L' U' L U R U' R'",
            "R U' R' U' H",
            "(U) H U' L' U' L"
        ],
        "s": "L' U L U R' L R L'"
    },
    "17": {
        "name": "Left Polish Flip",
        "a": [
            "(U') L' U L R U R'",
            "(U) R U R' U' L' U L",
            "(U) L' U L U S",
            "S U R U R'"
        ],
        "s": "R' L R L B' U' B L"
    },
    "18": {
        "name": "SUS",
        "a": [
            "S U' S",
            "R U' R' S L' U' L",
            "L' U' L R U' R' S"
        ],
        "s": "R' L R' L U L' R' L'"
    },
    "19": {
        "name": "Anti-SUS",
        "a": [
            "H U H",
            "L' U L H R U R'",
            "R U R' L' U L H"
        ],
        "s": "R U' B' R B L R L'"
    },
    "20": {
        "name": "Good Niky",
        "a": [
            "R U' R' L' U L",
            "(U) R U R' L' U' L",
            "L' B' U B L",
            "R B U B' R'"
        ],
        "s": "B L B' U' B L' B'"
    },
    "21": {
        "name": "Good Sochi",
        "a": [
            "(U') L' U L R U' R'",
            "(U) L' U' L R U R'",
            "(U') L' B' U' B L",
            "(U') R B U' B' R'"
        ],
        "s": "B L B' U B L' B'"
    },
    "22": {
        "name": "Super Sledge",
        "a": [
            "(U) R U' R' S",
            "H R U R'",
            "(U) R' L R2 U R' U' L'",
            "(U) L U R U' R2' L' R"
        ],
        "s": "R' L R' U R' U' L'"
    },
    "23": {
        "name": "Super Hedge",
        "a": [
            "(U') L' U L H",
            "S L' U' L",
            "(U') R' U' L' U L2 R L'",
            "(U') L R' L2' U' L U R"
        ],
        "s": "R' U' L' U L' R L'"
    },
    "24": {
        "name": "Bad Niky",
        "a": [
            "(U') R U' R' U' L' U L",
            "R U R' U' L' U' L"
        ],
        "s": "L' U' L U R U R'"
    },
    "25": {
        "name": "Bad Sochi",
        "a": [
            "(U) L' U L U R U' R'",
            "L' U' L U R U R'"
        ],
        "s": "R U B U' B' U' R'"
    },
    "26": {
        "name": "Right Spam",
        "a": [
            "R U R' U S",
            "(U) R U' R' L' U L R U' R'",
            "(U') L' U' L H R U R'",
            "(U') H R U R' L' U' L"
        ],
        "s": "R' B L R' B' U R' L'"
    },
    "27": {
        "name": "Left Spam",
        "a": [
            "(U') L' U' L U' H",
            "(U)  L' U L R U' R' L' U L",
            "R U R' S L' U' L",
            "S L' U' L R U R'"
        ],
        "s": "B' R B' U' B' L R' L'"
    },
    "28": {
        "name": "Bad Sledge",
        "a": [
            "H U' R U' R'",
            "(U) R U R' L' U' L R U R'",
            "R U' R2' L R L2' U L",
            "(U') H R U R' U' L' U L",
            "L' U L R U' R' S"
        ],
        "s": "R B' L' R' B' U L B'"
    },
    "29": {
        "name": "Bad Hedge",
        "a": [
            "(U) S U L' U L",
            "(U) L' U L2 R' L' R2 U' R'",
            "(U') S L' U' L U R U' R'",
            "(U) R U' R' L' U L H"
        ],
        "s": "L' B R L B U' R' B"
    },
    "30": {
        "name": "Bad Sexy",
        "a": [
            "(U') L' U' L U' R U' R'"
        ],
        "s": "B' L B' L U B L B"
    },
    "31": {
        "name": "Bad Ugly",
        "a": [
            "R U R' U L' U L"
        ],
        "s": "B' L' B' U' L' B L' B"
    },
    "32": {
        "name": "Bad Righty",
        "a": [
            "(U') L' U L U' R U R'",
            "(U') S U S"
        ],
        "s": "B R B L' U L R' B"
    },
    "33": {
        "name": "Bad Lefty",
        "a": [
            "R U' R' U L' U' L",
            "H U' H"
        ],
        "s": "B' L' B' R U' R' L B'"
    },
    "34": {
        "name": "Double Sexy",
        "a": [
            "R U R' U R U' R'",
            "L' U' L U S",
            "(U') R U' R' U R U R'",
            "(U) R U R' U' R U' R'",
            "H U' L' U L"
        ],
        "s": "R U' R' U R U R'"
    },
    "35": {
        "name": "Double Ugly",
        "a": [
            "(U') L' U' L U' L' U L",
            "(U') R U R' U' H",
            "L' U L U' L' U' L",
            "(U) L' U' L U L' U L",
            "(U') S U R U' R'"
        ],
        "s": "L' U' L U L' U L"
    }
};