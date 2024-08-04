const timesArrayKey = "3x3cmllTimes";
const selectionArrayKey = "3x3cmllselection";
var trainerTitle = "3x3 CMLL Trainer";
var preRotations = [''];
var postRotations = [''];
var preMoves = ["", "U", "U'", "U2"];
var postMoves = preMoves;
var selCases = [];
var selectedAlgSets = {"CMLL": true}
var maxAlgsPerRow = 8;
var algsGroups = {"CMLL Sune": [1, 2, 3, 4, 5, 6], "CMLL Anti-Sune": [7, 8, 9, 10, 11, 12], "CMLL Pi": [13, 14, 15, 16, 17, 18], "CMLL U": [19, 20, 21, 22, 23, 24], "CMLL L": [25, 26, 27, 28, 29, 30], "CMLL T": [31, 32, 33, 34, 35, 36], "CMLL H": [37, 38, 39, 40], "CMLL O": [41, 42]}

var algsets = {"CMLL": ["CMLL Sune", "CMLL Anti-Sune", "CMLL Pi", "CMLL U", "CMLL L", "CMLL T", "CMLL H", "CMLL O"]}

var algsInfo = {"1": {"a": ["(U) R U R' U R U2' R'", "R' U2' R U R' U R"], "name": "S1", "group": "Sune", "algset": "CMLL", "s": "R U2 R' U' R U' R'"}, "2": {"a": ["(U) F R' F' R U2 R U2' R'"], "name": "S2", "group": "Sune", "algset": "CMLL", "s": "R U2 R' U2 R' F R F'"}, "3": {"a": ["(U2) R' F2 R2 U2' R' F R U2' R2' F2 R", "R U R' U R U' R D R' U' R D' R2", "(U) R U R' U' R' F R F' R U R' U R U2 R'"], "name": "S3", "group": "Sune", "algset": "CMLL", "s": "R' F U2 F' R F R' U2 R F'"}, "4": {"a": ["(U) r U' r' F R' F' R"], "name": "S4", "group": "Sune", "algset": "CMLL", "s": "R' F R U R' F' U' F' U R"}, "5": {"a": ["(U) L' U2 L U2' r U' r' F", "(U) F U R U2 R' U' R U R' F'"], "name": "S5", "group": "Sune", "algset": "CMLL", "s": "R' F R2 U F R' U' R F' U' R'"}, "6": {"a": ["(U') R U R' U R' F R F' R U2' R'"], "name": "S6", "group": "Sune", "algset": "CMLL", "s": "R' U' F' U F U' R U' R' U2 R"}, "7": {"a": ["(U) R' U' R U' R' U2' R", "(U2) R U2' R' U' R U' R'"], "name": "AS1", "group": "Anti-Sune", "algset": "CMLL", "s": "R U R' U R U2 R'"}, "8": {"a": ["(U') F' r U r' U2' L' U2 L", "(U2) F R U' R' U R U2 R' U' F'"], "name": "AS2", "group": "Anti-Sune", "algset": "CMLL", "s": "R U F R' U R F' U' R2 F' R"}, "9": {"a": ["(U') R U2' R' U2 R' F R F'"], "name": "AS3", "group": "Anti-Sune", "algset": "CMLL", "s": "F R' F' R U2 R U2 R'"}, "10": {"a": ["(U') R' F R F' r U r'"], "name": "AS4", "group": "Anti-Sune", "algset": "CMLL", "s": "F R' U R U' F' U R' U' R"}, "11": {"a": ["(U2) R' F2 R2 U2' R' F' R U2' R2' F2 R", "(U') R2 D R' U R D' R' U R' U' R U' R'"], "name": "AS5", "group": "Anti-Sune", "algset": "CMLL", "s": "F R' U2 R F' R' F U2 F' R"}, "12": {"a": ["R U2' R' U' R U R2' F R F' R U2' R'", "R' U' R U' R' U R' F R F' U R", "(U) R U2' R' F R' F' R U' R U' R'"], "name": "AS6", "group": "Anti-Sune", "algset": "CMLL", "s": "R' U' F R' F' R U' R U R' U R"}, "13": {"a": ["R U2 R2 U' R2 U' R2 U2 R", "F R U R' U' R U R' U' F'", "(U2) F U R U' R' U R U' R' F'", "r' U r2 U' r2 U' r2 U r'"], "name": "Pi1", "group": "Pi", "algset": "CMLL", "s": "F U R U' R' U R U' R' F'"}, "14": {"a": ["(U) F R' F' R U2 R U' R' U R U2' R'", "(U) F U R U' R2' F' R U2 R U2' R'", "R' F2 R U2 R U2 R' F2 U' R U' R'"], "name": "Pi2", "group": "Pi", "algset": "CMLL", "s": "R U2 R' U2 R' F R2 U R' U' F'"}, "15": {"a": ["(U') R' F R U F U' R U R' U' F'", "R' F2 D R2 U' R2 D' F2 R", "(U) D' F R2 U' R2' U R2 U R2' F' D"], "name": "Pi3", "group": "Pi", "algset": "CMLL", "s": "F R U R' U' R U R' F' U' R' F' R"}, "16": {"a": ["R U2' R' U2(') R' F R2 U R' U' F'", "R U2' R' U' R U R' U2' R' F R F'"], "name": "Pi4", "group": "Pi", "algset": "CMLL", "s": "F U R U' R2 F' R U2 R U2 R'"}, "17": {"a": ["(U') r U' r2' D' r U r' D r2 U r'", "(U2) R' U R U' R2' F R2 U R' U' F' R"], "name": "Pi5", "group": "Pi", "algset": "CMLL", "s": "R' F R2 F R' U R F' R2 F' R"}, "18": {"a": ["(U') R' U' R' F R F' R U' R' U2 R", "(U2) R U R' U R U' B U' B' R'", "R' F' U' F U' R U R' U R"], "name": "Pi6", "group": "Pi", "algset": "CMLL", "s": "R' F' U' F U' R U R' U R"}, "19": {"a": ["R' U' R U' R' U2' R2 U R' U R U2' R'", "(U2) R U R' U R U2' R2' U' R U' R' U2' R", "R2' F U' F U F2 R2 U' R' F R", "(U') R U R' U' R U' R' U2 R U' R' U2 R U R'"], "name": "U1", "group": "U", "algset": "CMLL", "s": "R' F' R U R2 F2 U' F' U F' R2"}, "20": {"a": ["(U') F R2 D R' U R D' R2' U' F'", "(U') R2' F R F' R U2' R' F' U2' F R"], "name": "U2", "group": "U", "algset": "CMLL", "s": "R2 U R' U' R' F R U R' U' R F' R'"}, "21": {"a": ["(U2) R2 D R' U2 R D' R' U2 R'"], "name": "U3", "group": "U", "algset": "CMLL", "s": "R' F' R U R' U' R' F R2 U' R' U2 R"}, "22": {"a": ["(U2) r U' r' U r' D' r U' r' D r", "(U') R' D R U' R U' R' U R' D' R"], "name": "U4", "group": "U", "algset": "CMLL", "s": "R F R' U' R F' R' U R' F' R"}, "23": {"a": ["R2' D' R U2 R' D R U2 R"], "name": "U5", "group": "U", "algset": "CMLL", "s": "R F R' U2 R F' R' U2 R' F' R"}, "24": {"a": ["(U') F R U R' U' F'", "(U) F U R U' R' F'", "(U') f R U R' U' f'"], "name": "U6", "group": "U", "algset": "CMLL", "s": "F U R U' R' F'"}, "25": {"a": ["(U2) R U R' U R U' R' U R U' R' U R U2' R'"], "name": "L1", "group": "L", "algset": "CMLL", "s": "R U R' U R U' R' U R U' R' U R U2 R'"}, "26": {"a": ["R U2 R D R' U2 R D' R2'", "(U') R' F' R U R' U' R' F R2 U' R' U2 R"], "name": "L2", "group": "L", "algset": "CMLL", "s": "R U R' U' R' F U' R F' R' U F' R"}, "27": {"a": ["F R' F' R U R U' R'", "F R' F' r U R U' r'"], "name": "L3", "group": "L", "algset": "CMLL", "s": "R U R' U' R' F R F'"}, "28": {"a": ["F R U' R' U' R U R' F'", "(U) F' r U R' U' r' F R"], "name": "L4", "group": "L", "algset": "CMLL", "s": "F R U R' F' U' R' F' R"}, "29": {"a": ["(U') R' U2 R' D' R U2 R' D R2"], "name": "L5", "group": "L", "algset": "CMLL", "s": "R' F R U2 R F R' U2 R F' R'"}, "30": {"a": ["R U2' R2' F R F' R U2' R'"], "name": "L6", "group": "L", "algset": "CMLL", "s": "R U2 R2 F R F' R U2 R'"}, "31": {"a": ["R U2' R' U' R U' R2' U2' R U R' U R", "(U2) R' U2' R U R' U R2 U2' R' U' R U' R'", "F R' F R2 U' R' U' R U R' F2"], "name": "T1", "group": "T", "algset": "CMLL", "s": "R' F' R F' U' R' U' R F R' U R U F"}, "32": {"a": ["(U2) r U' r' U' F R' F' R2 U' R'", "r' U r U2' R2' F R F' R"], "name": "T2", "group": "T", "algset": "CMLL", "s": "R' F R U F' U' R' F' R2 U' R'"}, "33": {"a": ["(U) L' U' L U r U' r' F", "(U') F R U' R' U R U R' F'", "(U) l' U' L U R U' r' F", "(U2) R' U' R' D' R U R' D R2"], "name": "T3", "group": "T", "algset": "CMLL", "s": "R' F R U R' U' F' U R"}, "34": {"a": ["r' D' r U r' D r U' r U r'", "R' U' R' F R2 U' R' U2' R U R' F' R", "(U) R' D R U' R U R' U R' D' R"], "name": "T4", "group": "T", "algset": "CMLL", "s": "R' F R U' R F R' U R F' R'"}, "35": {"a": ["(U') R U R' U' R' F R F'"], "name": "T5", "group": "T", "algset": "CMLL", "s": "F R' F' R U R U' R'"}, "36": {"a": ["(U2) r U' r2' D' r U2 r' D r2 U r'", "(U2) r2 D' r U r' D r2 U' r' U' r"], "name": "T6", "group": "T", "algset": "CMLL", "s": "R' F R2 F R' U2 R F' R2 F' R"}, "37": {"a": ["(U) R U R' U R U' R' U R U2' R'", "R U2' R' U' R U R' U' R U' R'", "R' U2' R U R' U' R U R' U R"], "name": "H1", "group": "H", "algset": "CMLL", "s": "R' F R U R' F' R U' R' F' R"}, "38": {"a": ["(U2) r U' r2' D' r U' r' D r2 U r'", "r U' r' F U2' r2' F r U' r"], "name": "H2", "group": "H", "algset": "CMLL", "s": "R' F R2 F R' U' R F' R2 F' R"}, "39": {"a": ["(U2) R' F' R U2' R' F2 R U' F U F'", "(U) R U2' R2' F R F' U2 R' F R F'"], "name": "H3", "group": "H", "algset": "CMLL", "s": "R' F R U' R' F' U' R' F' R F' U R"}, "40": {"a": ["F U R U' R' U R U' R' U R U' R' F'", "F R U R' U' R U R' U' R U R' U' F'"], "name": "H4", "group": "H", "algset": "CMLL", "s": "F U R U' R' U R U' R' U R U' R' F'"}, "41": {"a": ["R U R' F' R U R' U' R' F R2 U' R'", "R U R' U' R' F R2 U' R' U' R U R' F'"], "name": "O1", "group": "O", "algset": "CMLL", "s": "R U' R F2 R' U R F2 R2"}, "42": {"a": ["r2 D r' U r D' r2 B r U' r' B'", "F R U' R' U' R U R' F' R U R' U' R' F R F'", "F R' F R2 U' R' U' R U R' F' R U R' U' F'"], "name": "O2", "group": "O", "algset": "CMLL", "s": "R2 U' R2 U' R2 U F U F' R2 F U' F'"}}