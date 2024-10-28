const timesArrayKey = "octaminxL3TTimesArray";
const selectionArrayKey = "octaminxL3TSelection";
var trainerTitle = "Octaminx L3T Trainer"
var maxAlgsPerRow = 6;
var preRotations = [''];
var postRotations = [''];
var preMoves = ["", "U", "U'"];
var postMoves = preMoves;
var selCases = [];
var selectedAlgSets = {"1L3T": true}

var algsGroups = {"1L3T OLP7 Even": [1, 2, 3], "1L3T OLP7 Odd": [4, 5, 6], "1L3T OLP8 Even": [7, 8, 9], "1L3T OLP8 Odd": [10, 11, 12]};

var algsets = {"1L3T": ["1L3T OLP7 Even", "1L3T OLP7 Odd", "1L3T OLP8 Even", "1L3T OLP8 Odd"]}

var algsInfo = {"1": {"a": ["R' L R L' U R' L R L' U L R' L' R U' R B' R' B"], "name": "7 E1", "group": "OLP7 Even", "algset": "1L3T", "s": "H' U H U' H' U' S'"}, "2": {"a": ["U' L R' L' R U' L R' L' R U' R' L R L' U L R' L' R"], "name": "7 E2", "group": "OLP7 Even", "algset": "1L3T", "s": "H U' H' U S U S"}, "3": {"a": ["U R' L R L' U R' L R L' U L R' L' R U' L R' L' R"], "name": "7 E3", "group": "OLP7 Even", "algset": "1L3T", "s": "H U S' U' S U' H'"}, "4": {"a": ["B' R B R' U R' L R L' U' L R' L' R U' L R' L' R"], "name": "7 O1", "group": "OLP7 Odd", "algset": "1L3T", "s": "H U' H U' H' U S"}, "5": {"a": ["U' L R' L' R U R' L R L' U' L R' L' R U' L R' L' R"], "name": "7 O2", "group": "OLP7 Odd", "algset": "1L3T", "s": "S U H U H' U' S"}, "6": {"a": ["U R' L R L' U' L R' L' R U R' L R L' U R' L R L'"], "name": "7 O3", "group": "OLP7 Odd", "algset": "1L3T", "s": "S' U' S' U' H U H'"}, "7": {"a": ["U' L R' L' R U R B' R' B U R B' R' B"], "name": "8 E1", "group": "OLP8 Even", "algset": "1L3T", "s": "H' U' H' U' S"}, "8": {"a": ["{U,BL} L R' L' R U' R' L R L' U' R' L R L'"], "name": "8 E2", "group": "OLP8 Even", "algset": "1L3T", "s": "H' U S' U H"}, "9": {"a": ["{U,BR} R' L R L' U L R' L' R U L R' L' R"], "name": "8 E3", "group": "OLP8 Even", "algset": "1L3T", "s": "H' U S' U S"}, "10": {"a": ["{U,BL} U' L R' L' R U L R' L' R U R B' R' B"], "name": "8 O1", "group": "OLP8 Odd", "algset": "1L3T", "s": "S' U H U H"}, "11": {"a": ["{U,BL} R' L R L' U' R' L R L' U' L R' L' R"], "name": "8 O2", "group": "OLP8 Odd", "algset": "1L3T", "s": "S' U H U S"}, "12": {"a": ["{U,BR} L R' L' R U R' L R L' U R' L R L'"], "name": "8 O3", "group": "OLP8 Odd", "algset": "1L3T", "s": "H U' H U' H'"}};