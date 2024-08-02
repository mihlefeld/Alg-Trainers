const timesArrayKey = "octaminxTCPTimesArray";
const selectionArrayKey = "octaminxTCPSelection";
var trainerTitle = "Octaminx TCP Trainer"
var maxAlgsPerRow = 6;
var preRotations = [''];
var postRotations = [''];
var preMoves = ["", "U", "U'"];
var postMoves = preMoves;
var selCases = [];
var selectedAlgSets = {"TCP": true}

var algsGroups = {"TCP Even": [1, 2, 3, 4, 5, 6], "TCP Odd": [7, 8, 9, 10, 11, 12], "TCP 2-Flip": [13, 14, 15, 16, 17, 18]};

var algsets = {"TCP": ["TCP Even", "TCP Odd", "TCP 2-Flip"]};

var algsInfo = {"1": {"a": ["B' R B R'"], "name": "E1", "group": "Even", "algset": "TCP", "s": "R B' R' B"}, "2": {"a": ["{U,BR} R B' R' B"], "name": "E2", "group": "Even", "algset": "TCP", "s": "L' B L B'"}, "3": {"a": ["{B,U} U' R' D R' U R D' R"], "name": "E3", "group": "Even", "algset": "TCP", "s": "R U' B' R' B' R B' U B R' B'"}, "4": {"a": ["{B,BL} U R D'  R U' R' D R2"], "name": "E4", "group": "Even", "algset": "TCP", "s": "L B L' U' L B' L B L U B'"}, "5": {"a": ["{B,BL} U R' U' R' D' R U R' D R U' R"], "name": "E5", "group": "Even", "algset": "TCP", "s": "L R L' B' R' U' B R' B' R U B"}, "6": {"a": ["{B,U} U' R U R D R' U' R D' R' U R'"], "name": "E6", "group": "Even", "algset": "TCP", "s": "R' L' R B L U B' L B L' U' B'"}, "7": {"a": ["U {B,BL} R D R' U R D' R U' R"], "name": "O1", "group": "Odd", "algset": "TCP", "s": "B' L' B U' B' L B L' U B L B'"}, "8": {"a": ["U' {B,U} R' D' R U' R' D R' U R'"], "name": "O2", "group": "Odd", "algset": "TCP", "s": "B R B' U B R' B' R U' B' R' B"}, "9": {"a": ["U {B,BL} R D R' U R D R U' R' D R2"], "name": "O3", "group": "Odd", "algset": "TCP", "s": "R L R' L' U L' B L B R' B R"}, "10": {"a": ["U' {B,U} R' D' R U' R' D' R' U R D' R"], "name": "O4", "group": "Odd", "algset": "TCP", "s": "L' R' L R U' L B L' B R B R'"}, "11": {"a": ["{L,U} U' D R' U R  D' {L,D} R' U R U'"], "name": "O5", "group": "Odd", "algset": "TCP", "s": "L R' L' B U' R' B R B' U B' R"}, "12": {"a": ["{R,BR} U D' R U' R' D {BR,F} R B' R' B"], "name": "O6", "group": "Odd", "algset": "TCP", "s": "R' L R B' U L B' L' B U' B L'"}, "13": {"a": ["R B' R' B R' L R L'"], "name": "T1", "group": "2-Flip", "algset": "TCP", "s": "B L B' R B' R' B L'"}, "14": {"a": ["{R,BR} R' U' Rw' R U' R U Rw R' U"], "name": "T2", "group": "2-Flip", "algset": "TCP", "s": "B U R' B' L' B L R U' B'"}, "15": {"a": ["{L,U} R U Rw R' U R' U' Rw' R U'"], "name": "T3", "group": "2-Flip", "algset": "TCP", "s": "B' U' L B R B' R' L' U B"}, "16": {"a": ["{B,BR} R' U' R D' R U' R' D R' U' R"], "name": "T4", "group": "2-Flip", "algset": "TCP", "s": "R U R' L R' L' R' U' R B' R B"}, "17": {"a": ["{R,BR} U Rw' U' R U Rw R U' R"], "name": "T5", "group": "2-Flip", "algset": "TCP"}, "18": {"a": ["{L,U} U' Rw U R' U' Rw' R' U R'"], "name": "T6", "group": "2-Flip", "algset": "TCP"}};