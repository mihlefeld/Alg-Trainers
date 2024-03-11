const timesArrayKey = "skewbTimesArray";
const selectionArrayKey = "skewbSelection";
var trainerTitle = "Skewb Alg Trainer";
var preRotations = ['', 'y', 'y2', "y'", "x", "x y", "x y2", "x y'", "z'", "z' y", "z' y2", "z' y'", "z", "z y", "z y2", "z y'", "z2 y'", "z2", "x2", "z2 y"];
var postRotations = [''];
var preMoves = [""];
var postMoves = preMoves;
var selCases = [];
var selectedAlgSets = {
    "NS2": true,
    "EG2": true
}
var maxAlgsPerRow = 8;
var algsGroups = {
    "Pi + Swirl": [1, 2, 3, 4, 5, 6, 7, 8],
    "Pi + Wat": [9, 10, 11, 12, 13, 14, 15, 16],
    "Pi + X": [17, 18, 19, 20, 21, 22, 23, 24],
    "Pi + HU": [25, 26, 27, 28, 29, 30, 31, 32],
    "Pi + VU": [33, 34, 35, 36],
    "Pi + O": [37, 38, 39, 40, 41, 42, 43, 44],
    "Pi + Z Conj": [45, 46, 47, 48, 49, 50, 51, 52],
    "Pi + Triple Sledge": [53, 54],
    "Pi + H/Z": [55, 56, 57],
    "Peanut + Swirl": [58, 59, 60, 61, 62, 63, 64, 65],
    "Peanut + Wat": [66, 67, 68, 69, 70, 71, 72, 73],
    "Peanut + X": [74, 75, 76, 77, 78, 79, 80, 81],
    "Peanut + HU": [82, 83, 84, 85, 86, 87, 88, 89],
    "Peanut + VU": [90, 91, 92, 93],
    "Peanut + O": [94, 95, 96, 97, 98, 99, 100, 101],
    "Peanut + Z Conj": [102, 103, 104, 105, 106, 107, 108, 109],
    "Peanut + Triple Sledge": [110, 111, 112, 113],
    "Peanut + H/Z/Pure": [114, 115, 116, 117],
    "L4C": [118, 119, 120, 121, 122, 123, 124, 125],
    "L5C": [126, 127, 128, 129, 130, 131],
    "EG2 Ori": [132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147],
    "EG2 Pi U": [148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159],
    "EG2 Pi BR": [160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171],
    "EG2 Pi FL": [172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183],
    "EG2 Pi BL": [184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195],
    "EG2 Pi FR": [196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207],
    "EG2 Peanut U": [208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219],
    "EG2 Peanut BR": [220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231],
    "EG2 Peanut FL": [232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243],
    "EG2 Peanut BL": [244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255],
    "EG2 Peanut FR": [256, 257, 258, 259, 260, 261, 262, 263, 264, 265, 266, 267]
};

var algsets = {
    "NS2": [
        "Pi + Swirl",
        "Pi + Wat",
        "Pi + X",
        "Pi + HU",
        "Pi + VU",
        "Pi + O",
        "Pi + Z Conj",
        "Pi + Triple Sledge",
        "Pi + H/Z",
        "Peanut + Swirl",
        "Peanut + Wat",
        "Peanut + X",
        "Peanut + HU",
        "Peanut + VU",
        "Peanut + O",
        "Peanut + Z Conj",
        "Peanut + Triple Sledge",
        "Peanut + H/Z/Pure",
        "L4C",
        "L5C"
    ],
    "EG2": [
        "EG2 Ori",
        "EG2 Pi U",
        "EG2 Pi BR",
        "EG2 Pi FL",
        "EG2 Pi BL",
        "EG2 Pi FR",
        "EG2 Peanut U",
        "EG2 Peanut BR",
        "EG2 Peanut FL",
        "EG2 Peanut BL",
        "EG2 Peanut FR"
    ]
}

var algsInfo = {
    "1": {
        "name": "1a",
        "a": [
            "H y S y' H y H",
            "H y H y' S y H",
            "x r' R r' R' B' r' B' r B ",
            "y' x r' R r R' z' R r' R r z R r' R' r'",
            "y x R b' r' R' r z B' r B",
            "y x r' R r' R' r' R r R z R r R' b' ",
            "R r' z' r' z r' R r' R r"
        ],
        "s": "B U' B R L' R B' U L"
    },
    "2": {
        "name": "1b",
        "a": [
            "y' S y' H y S y' S ",
            "y' S y' S y S y' H ",
            "y2 x r' B R r R' B r' B'",
            "x r' R r R r' R' r z' r' R r R' B' ",
            "y x r' R r R z2 r' R r R' z' R r' R' r'",
            "y2 x R r R' r' R r R z' r' R' r B",
            "b' r l r l' r b' l'",
            "y2 x R r' R r R r' R' r' z' r' R' r B"
        ],
        "s": "B' U B' L' R L' B U' R'"
    },
    "3": {
        "name": "2a",
        "a": [
            "y2 S y' S y H y' S ",
            "y2 H y' S y S y' S ",
            "y2 x r B R' B' r R r R' r' R r ",
            "y' x b' r' R r R' z' R r' R' r R r R'",
            "y2 x b' r' R r R r' R' r z r B r' B'",
            "y' x r' B' r z r' R r b R'",
            "z' r' R' r R' r b r R'",
            "z' r' R' r R' r z' r R f'"
        ],
        "s": "L' U B' L U R B' L U"
    },
    "4": {
        "name": "2b",
        "a": [
            "y H y' H y S y' H",
            "y S y' H y H y' H",
            "x R b R' z' R r' R' B' r",
            "x z R' r' R' r z R r R' r z' r' R r R'",
            "y2 x B' r' R r R z R r R' r' R' r R'",
            "y x B R r' R' r' R r R' r' B' r B"
        ],
        "s": "U' L' R B R' L' B' R B' R' L"
    },
    "5": {
        "name": "1c",
        "a": [
            "S y S y' S y H",
            "S y H y' S y H",
            "y' x R r' R' z' r' R' r B R/R2' B' R",
            "x y2 r' R r R b' r R' b'",
            "y x R r' R r R' B R' B' r' R' r R",
            "y' x r B' b' r' B' r B b"
        ],
        "s": "L' B R' B R U R' U' R U R' U'"
    },
    "6": {
        "name": "1d",
        "a": [
            "y' H y' H y H y' S ",
            "y' H y' S y H y' H ",
            "x r' R r z R r R' z' r' R' r R'",
            "y x R' r' R' r z R r' R' r' R r R'",
            "y x' R r' R' r' B R' r B"
        ],
        "s": "L' R' B L' R B R B'"
    },
    "7": {
        "name": "2c",
        "a": [
            "y2 S y' H y H y' H",
            "y2 H y' H y S y' H",
            "x r' R r' R' z' r' R r B R B'",
            "y2 x R' r' B' r B R r B'",
            "y2 b R r' b R' r' R' r"
        ],
        "s": "R L' U' B' L' U R U"
    },
    "8": {
        "name": "2d",
        "a": [
            "y H y S y' S y S",
            "y S y S y' H y S",
            "y' x R r' R r B R' B' r' R' r",
            "x b r B r' B' z' r' R' b",
            "y2 z B' b' R r' b r B r'",
            "x2 y B' r' R B' r R r R'"
        ],
        "s": "L' R U B R U' L' U'"
    },
    "9": {
        "name": "3a",
        "a": [
            "y' S y2 H y' S y' S",
            "x r2' R r B R r' R B",
            "y' x R B R' B' R' r' R r R r' R' r",
            "y x r' R r R' r' R' r R B R B' R'",
            "B' r' l r' l' B' r' b'"
        ],
        "s": "R L' B U' L U' L' B U' L"
    },
    "10": {
        "name": "3b",
        "a": [
            "y2 S y2 H y H y S",
            "x B' R' B' r' R' B R' r'",
            "y x R' r' R r R B R' B' R' B R B'",
            "y2 x R r R' r' R' z' r' R r z R r'",
            "y x B R' B' R B R B' R' r' R' r R",
            "y2 x R r' R r' R r R B R' B' r",
            "r' R' B' r' R' r z2 R' r'"
        ],
        "s": "L' R B' U R' U R B' U R'"
    },
    "11": {
        "name": "4a",
        "a": [
            "y S z S y2 S",
            "x r' R' r B R' B' R' r' R r R'",
            "y2 x r2'/r R' r B r' z r R' r' R' r"
        ],
        "s": "L' U' L R U' L' R' B R B R' U"
    },
    "12": {
        "name": "4b",
        "a": [
            "y2 H z' S y2 S",
            "y x r' R r R B' r' R' r B R'",
            "y' x R2'/R B' r' R r B R' r' R' r",
            "x B R B' r' R r R B R' B' R",
            "y' x R r' R' r R r' R r B R' B' R'"
        ],
        "s": "U' L R' U L B' L' U B' U"
    },
    "13": {
        "name": "3c",
        "a": [
            "y S y2 H y' H y' S",
            "y S y S y H y2 S",
            "x r' R' r R' z' r2'/r R' r B r' R",
            "y' x R r' R' r R' r' R r z r R r' R'",
            "y x B R B' R' r' R' r R r' R r R'",
            "y' x R' r R r' R' r R' r' z R r' R'"
        ],
        "s": "U' L' R U L B' L' B' L U' L'"
    },
    "14": {
        "name": "3d",
        "a": [
            "y2 S y2 H y S y S",
            "H y' H y' S y2 H",
            "x r' R r R' r R r' R' z' R' r' R r",
            "x r R' r' R r R' z' R B r' R r",
            "y' x r' R' r R z R r R' r' R r' R' r"
        ],
        "s": "L' U L R' L' B' R L U' R"
    },
    "15": {
        "name": "4c",
        "a": [
            "y H z S y2 S",
            "y' x R r' R r z R r' R' r R r R'",
            "y2 x r' R r R B R' B' R' r' R' r R",
            "x R r' R' r' R r R' z' r' R' r R'"
        ],
        "s": "R U L' B U L R' L R'"
    },
    "16": {
        "name": "4d",
        "a": [
            "y2 S z' S y2 S",
            "y x R r' R ' r' z' r' R r R z R r R' r'",
            "y' x r' R r R r' R' r B R B' R",
            "y x R' B R' B' r' R r R' r' R' r"
        ],
        "s": "L' U' R B' U' R' L R' L"
    },
    "17": {
        "name": "5a",
        "a": [
            "y2 S y S S S",
            "x' S y2 S z2 x' H",
            "x z' r' R z' r' R r z r R r' R",
            "x R' r' R r R z B R' B' R r'",
            "x z' r' R' r B r' R r z R r' R r"
        ],
        "s": "R U L' B R' B' R U B' R L"
    },
    "18": {
        "name": "5b",
        "a": [
            "y H y' S S S",
            "z y' S y2 S x z2 S",
            "y x b' r' R r R' z2 r' R' r R' ",
            "y2 x R r' B R' B' R' r' R r'",
            "r R' r B x r' l r l' r"
        ],
        "s": "L R' L U R U' R' B R"
    },
    "19": {
        "name": "6a",
        "a": [
            "y H y2 x' S y2 S",
            "y x R r' R r z2 R r' R' r b ",
            "y x R r' R r z B R' B' R r/r2'",
            "x r2' R' r z r R r R' b r'"
        ],
        "s": "R U R' L R' U L' B L"
    },
    "20": {
        "name": "6b",
        "a": [
            "y2 S z2 x' S y2 S",
            "y' x R' r R' r' z' r' R' r z R' r ",
            "y2 x r' R r' R' z2 r' R r R' B'"
        ],
        "s": "L' U' L R' L U' R B' R'"
    },
    "21": {
        "name": "5c",
        "a": [
            "y2 H y S S S",
            "x' S y2 S z2 x' S",
            "y' x r' R r' R' r' R r R' z' R r' R r",
            "x r' R' r R' z2 r' R r R' b'",
            "y' x R r R' r R r' R' r z' r' R r",
            "x b' R r' R r R' b' R' r' R' r"
        ],
        "s": "L B' L' R B L' B' L R' L B' U"
    },
    "22": {
        "name": "5d",
        "a": [
            "y S y' S S S",
            "z y' S y2 S y2 x' H",
            "x r' R' r R' r' R r R' B R' B' ",
            "x z R r R' z' R r z' r' R' r B",
            "x R r' R r R r' R' r z r' R r' R'",
            "y' x B r' R r' R' r B r R r R'"
        ],
        "s": "L' R B' U R' B' L R U' B"
    },
    "23": {
        "name": "6c",
        "a": [
            "y S y2 z' S y2 S",
            "y' x B' r' R r R' z2 r' R r' R'",
            "x r' R r R' z' R' r' R r R B R' B' ",
            "y2 x B R B' R r' R' r R r' R r"
        ],
        "s": "L' B L' R B' R' L R B' R"
    },
    "24": {
        "name": "6d",
        "a": [
            "y2 H z2 x' S y2 S",
            "y x r B R' B' R r z' r' R r",
            "x z R r' R' r R B R' B' R' r' R r",
            "x z r R' B r z' r2' R r B' R'",
            "y2 r' R' r z r' R r R' r2/r' R r' R'"
        ],
        "s": "R B' R L' B L R' L' B L'"
    },
    "25": {
        "name": "7a",
        "a": [
            "y S S z2 x' S y2 S",
            "y' x B r' R r' R' r B r'"
        ],
        "s": "R B R' L U' L' U' L R L'"
    },
    "26": {
        "name": "7b",
        "a": [
            "y S S z' S y2 S",
            "y2 x r' z R r' R r z r' B' r",
            "y x b' R r' R r R' b' R"
        ],
        "s": "L' B' R B R U' R' L R' B"
    },
    "27": {
        "name": "7c",
        "a": [
            "y' S S y2 z' S y2 S",
            "y' x r B' r' R r R' r B'"
        ],
        "s": "R' B' L B L' B' L R U' R' L'"
    },
    "28": {
        "name": "7d",
        "a": [
            "S S z S y2 S",
            "y2 x B' r z R r' R' r z' B' r",
            "y z' r' R r B' r' b B' l"
        ],
        "s": "U' L R' L U L' B' L"
    },
    "29": {
        "name": "8a",
        "a": [
            "S y' S y' S",
            "x R' r R' B b r' R r' ",
            "y2 x r' R r B R' B' R' r' R' r R'"
        ],
        "s": "L R' U B' R U' B' L"
    },
    "30": {
        "name": "8b",
        "a": [
            "S y' H y' S",
            "x R' r B R' B' r2' R' r",
            "y x r' B R B' z' r' R r B'",
            "y' x B R' B r' R' r B R'",
            "y' x r' R r R z' r' R' r B r' R r ."
        ],
        "s": "R L' U' L U R' B' U"
    },
    "31": {
        "name": "8c",
        "a": [
            "S y H y S",
            "H y H y H",
            "x r/r2' R' r b' B' R r' R",
            "y2 x R r' R' z' r' R r R z R r R' r"
        ],
        "s": "R' L U' B R L' R L'"
    },
    "32": {
        "name": "8d",
        "a": [
            "S y S y S",
            "H y S y H",
            "y x R/ R2' B' r' R r z R' r R'",
            "x B r' R' r z B R' B' r",
            "y2 x r' R r'/r2 B R B' r' R"
        ],
        "s": "L' B' L U L' U' R' U R U"
    },
    "33": {
        "name": "9a",
        "a": [
            "y2 S S y' S y2 S",
            "y' S y2 S y S S",
            "y' x B' r' R r R B' R' r' R' r B' ",
            "y' x R r R' r B r' z' r' R r R' b",
            "y' r' R r B' R B r' R' r R'",
            "r' l r l' r' R' F r' R r",
            "y x R r' R r z r' R r R' z' R r' R' r R "
        ],
        "s": "L' B' L' R B' L' B' L' B R'"
    },
    "34": {
        "name": "9b",
        "a": [
            "y S S y' S y2 S",
            "S y2 S y' S S",
            "y' x B r' R r z R r' R' r R' r'",
            "x z r B R' B' z' r' R' r B",
            "y2 x r' B R' B' r' R r R' r R",
            "y2 x B' r' R r z B R B' r'",
            "x z' r' R r z R' r B r' R B",
            "x R' r' R r' R' r x' r B r' R"
        ],
        "s": "R B L' U' B' R' B L"
    },
    "35": {
        "name": "10a",
        "a": [
            "S y2 H",
            "x z r B r B' r' B' ",
            "y' z' B R r R' B' R'"
        ],
        "s": "L B R B' L' B'"
    },
    "36": {
        "name": "10b",
        "a": [
            "y2 H y2 S",
            "x B' r' B' r B r",
            "x B' r' y' r' R r b"
        ],
        "s": "R' B' R' L U R"
    },
    "37": {
        "name": "11a",
        "a": [
            "y2 H y' S y2 S",
            "y' x R r' R' r' R r R' z' r' R r"
        ],
        "s": "L' U R' B' R B L' B L R'"
    },
    "38": {
        "name": "11b",
        "a": [
            "y S y S y2 S",
            "",
            "x r' R r R r' R' r z R r' R'"
        ],
        "s": "L B' R U' R' U' L' U L U"
    },
    "39": {
        "name": "12a",
        "a": [
            "y2 S y2 S y S",
            "y S y' x' S y2 S",
            "y' x r' R' r z R r' R' r R r R'",
            "y' x r' R' r B' R z R r R' r' R",
            "y2 x R r' R' r b r' R r R' z' R r' R'"
        ],
        "s": "R U L' U' L B' L' U' R U' R'"
    },
    "40": {
        "name": "12b",
        "a": [
            "y S y2 S y' H",
            "y2 H y x S y2 S",
            "y x B R B' r' R r R' r' R' r",
            "y2 x R r' z r R r R' z2 r' R r"
        ],
        "s": "L' U' R U R' B R U L' U L"
    },
    "41": {
        "name": "11c",
        "a": [
            "y2 S y' S y2 S (Cancel)",
            "",
            "y x r/r2' R' r B r' R B"
        ],
        "s": "B' L' R U' R' L U'"
    },
    "42": {
        "name": "11d",
        "a": [
            "y H y S y2 S",
            "y H y H y2 H (Cancel)",
            "y2 x R' r R' z' r' B R' r'"
        ],
        "s": "B R L' U L R' U"
    },
    "43": {
        "name": "12c",
        "a": [
            "y2 S y2 S y H ",
            "y H z' y' S y2 S",
            "x B' R' r B' r' R r'"
        ],
        "s": "U L' U B U' L R"
    },
    "44": {
        "name": "12d",
        "a": [
            "y S y2 S y S",
            "y2 S x z' S y2 S",
            "x r R B' r B R' B"
        ],
        "s": "U' R U' B' U R' L'"
    },
    "45": {
        "name": "13a",
        "a": [
            "H x' z S y2 S",
            "y x r' R' r' R' z' r' R r R B' ",
            "y x r' R' r B' r' R r B "
        ],
        "s": "L R' B' L' R L' R L' R L"
    },
    "46": {
        "name": "13b",
        "a": [
            "y' S x' z S y2 S",
            "y' x B R B' r B R' B' r' ",
            "y' r B r' R r B' r' R'"
        ],
        "s": "B U R L' U' L R' U'"
    },
    "47": {
        "name": "13c",
        "a": [
            "H x' z S y2 S",
            "x r B R B' r' B R' B'"
        ],
        "s": "L' B' L U' R' L' U' R U"
    },
    "48": {
        "name": "13d",
        "a": [
            "y' S x' z S y2 S",
            "x B' r' R' r B r' R r"
        ],
        "s": "L' R B' U R B' U R' L' R"
    },
    "49": {
        "name": "14a",
        "a": [
            "y' S y' S y H (Cancel)",
            "y' H y' S y S",
            "x R' r' R' r R r' R' r R r' R r ",
            "x z R' r' R r B' r' R' r B R",
            "y x r R r R' r' R r' R' r R r' R'",
            "y x r' R' z' r' R r z R r R r R' r",
            "y' z R r R' r' R r R r' R' r R' r'"
        ],
        "s": "L' U' R L' R U R B'"
    },
    "50": {
        "name": "14b",
        "a": [
            "y S y' H y H  (Cancel)",
            "y H y' H y S",
            "y x R' B' r' R r B r' R' r R",
            "y2 x B R' B' r' R r R' z' R r' R' r B"
        ],
        "s": "L U' R' U' B L' U B"
    },
    "51": {
        "name": "14c",
        "a": [
            "y' H y H y' S (Cancel)",
            "y' S y H y' H",
            "y2 x r R r R' r' R' r R r R' r' R r",
            "y2 x R' r' R r' R' r R r' R r R' r",
            "y z r' R' r R r' R' r' R r R' r R "
        ],
        "s": "L' B' L R L' U L R' L' R U R'"
    },
    "52": {
        "name": "14d",
        "a": [
            "y H y S y' S (Cancel)",
            "y S y S y' H",
            "y' x R r R' B R B' R r2' R r ",
            "x R r R' r R r' R' r R r' R' r'",
            "y2 x r z r R' B r' R' r' z' r",
            "y x R r' R r R r B R B' r' R'",
            "y' z R' r' R r' R' r R r R' r' R r"
        ],
        "s": "L' R L' B L R' L R' B R U' L'"
    },
    "53": {
        "name": "15a",
        "a": [
            "S S z S S S ",
            "x r' R r R r' z' r' R r z r R r'",
            "x R r' R' r' R B R' B' R' r' R",
            "y2 x R r R' B' R r' R' r R r R' B' "
        ],
        "s": "L R' B R U' L' R' U L U' B L'"
    },
    "54": {
        "name": "15b",
        "a": [
            "z' S S z' S S S",
            "x r R' r' z' r' R' r z r R' r' R' r",
            "x R' r' R r z r b r' R r R' z' R r' R",
            "y2 x b' r' R r R r' R' r b' r' R r",
            "y2 x r B r' B' r' R r R' r B r' B' "
        ],
        "s": "R' L R B' R' L' U B' L U R U'"
    },
    "55": {
        "name": "16",
        "a": [
            "y' S y2 S y2 S",
            "y S y2 S y2 H",
            "x r B R B' r2' R r R",
            "y2 x R' r' R' r'/r2 B R' B' r'",
            "y x B' r' R' r B'/B2 R' B' R' ",
            "y' x R/R2' B R B/ B2' r' R r B"
        ],
        "s": "R U R' L' R B' U R' B' U"
    },
    "56": {
        "name": "17a",
        "a": [
            "y S y' H y H y' S y H",
            "y2 x r' R' r R B R' B' r' R r",
            "x r' R' r B R B' R' r' R r"
        ],
        "s": "L B R' U B' L' B L U L' R"
    },
    "57": {
        "name": "17b",
        "a": [
            "y S y H y' H y S y' H",
            "y2 x R r R' r' z' r' R r z R r' R'",
            "y x R r2' R' r B r' R r' R' r B",
            "x R r R' z' r' R' r R z R r' R'",
            "x r' R r R' z' r' R r' R' z' r' R r R",
            "y' x r R r R' z R r' R' r b r' R r R'",
            "y x R r' R' r b' r' R r R' z' R r' R' r'"
        ],
        "s": "R' U' B' L B L' B L U' R"
    },
    "58": {
        "name": "18a",
        "a": [
            "H y H y' H y H (Cancel)",
            "S y' H x z S y2 S",
            "y2 H y H x S y2 S (Cancel)",
            "y x b' r' R r R' z' R r' R' r'",
            "y' x R' r' R' r z' r' R r R' B'"
        ],
        "s": "L' U' L R L' U L R' L B R' B'"
    },
    "59": {
        "name": "18b",
        "a": [
            "y' S y' S y S y' S (Cancel)",
            "y' H y S x' z S y2 S",
            "y S y' S z' S y2 S (Cancel)",
            "x r/r2' R r R' z R r' R' r b",
            "y2 x B R r' R' r z r' R r R"
        ],
        "s": "B' L' U' R' L R' B' R L' B'"
    },
    "60": {
        "name": "18c",
        "a": [
            "y2 S y' H y S y' H ",
            "S y' H x z S y2 S",
            "y2 H y H x S y2 S (Cancel)",
            "x R r' R r z R r' R' r'",
            "y2 x B' R r' R' r' z' r' R' r b",
            "y x B' r' R' r B r' R' r R'"
        ],
        "s": "L R' L' B U' L U L' U B'"
    },
    "61": {
        "name": "18d",
        "a": [
            "y H y S y' H y S",
            "y' H y S z S y2 S",
            "y S y' S x' z S y2 S (Cancel)",
            "x z r' R r' R' z' r' R r R",
            "y x b r' R r z r R r R' B'"
        ],
        "s": "R' L R B' U R' U' R U' B"
    },
    "62": {
        "name": "18e",
        "a": [
            "S y S y' S y S (Cancel)",
            "y2 S y S z S y2 S",
            "H y' S x z S y2 S",
            "y' x B r' R r R' z R r' R r",
            "y' x R r' R r R' z' R r' R' r B'"
        ],
        "s": "R B L' B' U R' B' R B R U' R'"
    },
    "63": {
        "name": "18f",
        "a": [
            "y' H y' H y H y' H (Cancel)",
            "y H y' H z' S y2 S",
            "y' S y H x' z S y2 S",
            "y2 x B r' R r R' z R r' R' r R'",
            "x r' R' r R' z' R r' R' r B'"
        ],
        "s": "L' B' R B U' L B L' B' L' U L"
    },
    "64": {
        "name": "18g",
        "a": [
            "y2 H y' S y H y' S",
            "H y' S z' S y2 S",
            "y2 S y' S x' z S y2 S",
            "x R' r' R' r z R r R' r",
            "y x r' R' r' R' z' r' R r B' R",
            "x B R r' R' r' z' r' R' r b'"
        ],
        "s": "L' U' R B U' R' L R' U' B L B"
    },
    "65": {
        "name": "18h",
        "a": [
            "y S y H y' S y H ",
            "y H y' H x' z S y2 S",
            "y' S y H z S y2 S",
            "x z r R r R' z' r' R' r R'",
            "y2 x R r' R r' R' z' r' R r R B'",
            "y' x b' r' R r z r R r R' B"
        ],
        "s": "L B' U L' R B L' U' B R' B' R"
    },
    "66": {
        "name": "19a",
        "a": [
            "y' H y S z' S y2 S",
            "y2 S y2 S y' H y' H",
            "S y S y S y2 S",
            "y' x r/r2' R r R' B R' B' R' r R' ",
            "x z' R r' z r2' R r R' z' R r' R' r'"
        ],
        "s": "R' L B L' B L U' B L' U L"
    },
    "67": {
        "name": "19b",
        "a": [
            "S y' H z S y2 S",
            "y S y2 S y S y S",
            "y' H y' H y' S y2 S",
            "y2 x r' R r z R r R' z R r' R' r b",
            "y2 x b' r' R r R' z' R r' R' z' r' R' r",
            "x R' r' R' r z' r' R r z r R' r"
        ],
        "s": "L R' B' R B' R' U B' R U' R'"
    },
    "68": {
        "name": "19c",
        "a": [
            "y S y' S z S y2 S",
            "y2 S y2 S y S y H",
            "S y' H y' S y2 S",
            "x r2' R r R' z' r' R r",
            "y' x r' R' r B R' B' R' ",
            "x r R r R' z y R' r R",
            "x R' r B R' B' r' R'"
        ],
        "s": "U' L' R U L' R' L'"
    },
    "69": {
        "name": "19d",
        "a": [
            "y2 H y' H z' S y2 S",
            "y S y2 S y' H y' S",
            "y' H y S y S y2 S",
            "x R r R' z' r' R r R",
            "y' x R' r' R' r B R' B'",
            "y x r' R' z' r' R' r B R' ",
            "y' r R' z' r' R r B R "
        ],
        "s": "B' L' R' B' L U B'"
    },
    "70": {
        "name": "19e",
        "a": [
            "y' S y H z' S y2 S",
            "y2 S y2 S y' S y' S (Cancel)",
            "H y H y S y2 S",
            "y x B' R r R' r' R r R' B' r ",
            "y2 x b r' R' r z2 r' R r b'",
            "y2 B l' B' l r' y r R r'",
            "y r R' r' y' r l' B R r'",
            "y r R' r' y' r l' B l B'"
        ],
        "s": "L B L' R U R' L B' R' L'"
    },
    "71": {
        "name": "19f",
        "a": [
            "H y' S z S y2 S",
            "y S y2 S y H y H (Cancel)",
            "S y' S y' S y2 S",
            "y r' R r b' B z' R' r' R",
            "x z' B' R r R' z2 R r' R' B",
            "y2 B' r B r' l y r' R' r"
        ],
        "s": "R' B' R L' U' L R' B L R"
    },
    "72": {
        "name": "19g",
        "a": [
            "y H y' H z S y2 S",
            "y2 S y2 S y H y S",
            "H y S y' S y2 S",
            "x z R' r' R r z r R r R' r",
            "x r' R r' R' r' z' r' R' r R  "
        ],
        "s": "B L R' U' R L' R U B' L'"
    },
    "73": {
        "name": "19h",
        "a": [
            "y2 S y S z' S y2 S",
            "y S y2 S y' S y' H ",
            "S y H y S y2 S",
            "x r R r' R' z' R' r' R' r R'",
            "y' x R r' R r R z R r R' r' "
        ],
        "s": "R' L' U' R B L B' L' U' L' U"
    },
    "74": {
        "name": "20a",
        "a": [
            "y S y' S (Cancel)",
            "x r' R r R' z' r' R r R'"
        ],
        "s": "B L' U' B L R' B U'"
    },
    "75": {
        "name": "20b",
        "a": [
            "S y S",
            "x r' R r R' z r' R r R'"
        ],
        "s": "U L' U' B' L' U' L"
    },
    "76": {
        "name": "20c",
        "a": [
            "y2 H y H (Cancel)",
            "x R r' R' r z R r' R' r"
        ],
        "s": "U' B L B' L' B U' R"
    },
    "77": {
        "name": "20d",
        "a": [
            "y' H y' H",
            "y x r' B r' R r R' r2 B"
        ],
        "s": "U L' R B L U' L' B' L U'"
    },
    "78": {
        "name": "20e",
        "a": [
            "S y' H",
            "y2 x B' R r' R' r z r' R r"
        ],
        "s": "B' L R L' B L' U' L"
    },
    "79": {
        "name": "20f",
        "a": [
            "y S y H",
            "y' x r' R' r z' r' R r R' B "
        ],
        "s": "U L' R U R' L R U R'"
    },
    "80": {
        "name": "20g",
        "a": [
            "y' H y S",
            "x y b r' R r R' z' R r' R' ",
            "y x R' r' R r R' z' R r' R' r B"
        ],
        "s": "L R' U' R L' B L B'"
    },
    "81": {
        "name": "20h",
        "a": [
            "y2 H y' S",
            "x R r R' z R r' R' r b'",
            "x B' r' R r R' z R r' R' r R"
        ],
        "s": "U' R L' U' L R' L' U' L"
    },
    "82": {
        "name": "21a",
        "a": [
            "y H y H y S (cancel)",
            "y x b' r' R r R z' r' B' r B r'",
            "y' x R r' R' z' r' R r z r R r' R' r' R r R'"
        ],
        "s": "L' R U' R' U' R U R' B' R B L"
    },
    "83": {
        "name": "21b",
        "a": [
            "y H y' S y' S (cancel)",
            "y' x R r R' z R r' R' r z' r2' R r R'",
            "x z R r R r' R' r' z' r' R r B' ",
            "y2 x B r' R r R z R r R' B R' B'"
        ],
        "s": "R B R' L' B R B R' L"
    },
    "84": {
        "name": "21c",
        "a": [
            "y2 S y' S y' H (cancel)",
            "x B' r R' r' B r R r'",
            "x B' r' B r B' z r' R' r z' r'",
            "x r' R z R r' z' r' R' r' R' r R' B"
        ],
        "s": "R' B' R L R' B' U R U' R' B'"
    },
    "85": {
        "name": "21d",
        "a": [
            "y2 S y H y H (cancel)",
            "x r' R' r' R r R B R' B' r",
            "x r' R' r z' r' R r R' z R' r' R' r",
            "y x b' R r' R' r' z' r' R' r z' r' R r",
            "y2 x r R' r' B' r R r' B"
        ],
        "s": "L' B' L R B' L' B' L R'"
    },
    "86": {
        "name": "21e",
        "a": [
            "y' S y' H y' H",
            "y x R' r R' r' z' r' R' r z r R'",
            "y2 x R r R' B R B' r' R r R r'",
            "y2 x R r R' r' R' z' r' R' r B R"
        ],
        "s": "L' B R B R' L R B R'"
    },
    "87": {
        "name": "21f",
        "a": [
            "y' S y S y H",
            "y' x R r' z' r' R r z r R r' R",
            "x r' R' z' r' R r z R r R r' R'"
        ],
        "s": "L' R' B L U' R B' R' U'"
    },
    "88": {
        "name": "21g",
        "a": [
            "H y S y S",
            "x R' B' R r' y' r' R' z' R r R'",
            "x R' r' R' r B R' B' R r' R' r",
            "y2 x r/r2' R' r z r R r R' r' z' r"
        ],
        "s": "R B' L' B' L R' L' B' L"
    },
    "89": {
        "name": "21h",
        "a": [
            "H y' H y' S",
            "x r' R B R' B' R' r' R r'",
            "y2 x r' R r R' r' R' z' r' R' r B R'"
        ],
        "s": "R L B' R' U L' B L U"
    },
    "90": {
        "name": "22a",
        "a": [
            "x2 r R r' R' z R r' R' r",
            "x r R' r' R r R B R' B' r'",
            "x z2 r B R B' R' r' R' r R r'"
        ],
        "s": "R L' R' U R L B L' B' R'"
    },
    "91": {
        "name": "22b",
        "a": [
            "y' R' r' R r z' r' R r R'",
            "x B' R B R' B' R' r' R r B ",
            "x z2 B' r' R' r R B R B' R' B"
        ],
        "s": "L' R L U' L' R' B' R B L"
    },
    "92": {
        "name": "22c",
        "a": [
            "y' x R' r' z' r' R r z r R r'",
            "y r' R' r R z' R r' R' r",
            "y x r R' r' z r' R' r z r R"
        ],
        "s": "B U' R' L' B' U L R"
    },
    "93": {
        "name": "22d",
        "a": [
            "x r R z R r' R' z' R' r' R",
            "y r R r' R' x r' R r R'",
            "z2 R r R' r' z r' R r R'"
        ],
        "s": "B' U L R B U' R' L'"
    },
    "94": {
        "name": "23a",
        "a": [
            "y2 H y' H y2 S ",
            "x z r R r' R' z' R' r' R r"
        ],
        "s": "L' R L R' U' R' U R"
    },
    "95": {
        "name": "23b",
        "a": [
            "y S y S y2 H",
            "x R' r' R r z r R r' R'",
            "y2 x B r R' r' B' r R r'"
        ],
        "s": "R U R' B' R B R U' R' U'"
    },
    "96": {
        "name": "23c",
        "a": [
            "y H y H y2 S (cancel)",
            "x r' R' r R z R r R' r'"
        ],
        "s": "B U L' U' B' L' U L"
    },
    "97": {
        "name": "23d",
        "a": [
            "y2 S y' S y2 H (cancel)",
            "x z R r R' r' z' r' R' r R"
        ],
        "s": "L U L' U' L' R L R'"
    },
    "98": {
        "name": "23e",
        "a": [
            "S y H y2 S",
            "y' R r R B R' B' R' r'",
            "x z2 r R r R' r' R' z' r' R' r B"
        ],
        "s": "U L R B U' R' L' B'"
    },
    "99": {
        "name": "23f",
        "a": [
            "y' H y' S y2 H",
            "y x B' r' R r R' z/z3' R' r' R r R",
            "y x R B' r' B' r B r B R' B'",
            "y r R B R B' R' r' R'",
            "x2 r' R' z' R' r' R r z r R"
        ],
        "s": "B L R U B' R' L' U'"
    },
    "100": {
        "name": "23g",
        "a": [
            "H y S y2 H",
            "y2 x b r' R r R' z' R r2'/r R' r ",
            "y' z' r' R' r l' r' R r R",
            "y x r B' r' R r R' r z r' R' r"
        ],
        "s": "R' U R L' B' R U B' L B'"
    },
    "101": {
        "name": "23h",
        "a": [
            "y' S y' H y2 S",
            "y r' R' r' R r z' r' R r",
            "",
            "y x r' R r' R' z R r' R' r b'"
        ],
        "s": "L U' L' R B L' U' B R' B"
    },
    "102": {
        "name": "24a",
        "a": [
            "y' S y' H y S",
            "x z R r R' r' R r R' r z' r' R r"
        ],
        "s": "R L' B L R B' R' L U' L'"
    },
    "103": {
        "name": "24b",
        "a": [
            "H y S y' H",
            "x r' R' r R r' R' r R' z R r' R'",
            "x R B R' B' r' R' r R r' R' r R ",
            "y x B r' R r' R' r B z' r' R' r",
            "y' x R' r' R' r' z' r' R' r R r' R' r B'",
            "x z2 R r R' r' R r R' z' R r' R' r R'"
        ],
        "s": "L' R B' R' L' B L R' U R"
    },
    "104": {
        "name": "24c",
        "a": [
            "y' H y' S y H",
            "x r' R' r R' z R r' R' r R r' R'",
            "x r' B R' B' R B R' B' R' r' R' r'",
            "x z2 r/r2' R r z B r' B' z' r' R' r' z' r "
        ],
        "s": "L U L' R B R' L' B' L R'"
    },
    "105": {
        "name": "24d",
        "a": [
            "S y H y' S",
            "y' x R r' R r R' z R r' R' r R r' R'",
            "y x R r R' r z' r' R r R' r' R r",
            "x B r' R r R' r' R r z r R r.R"
        ],
        "s": "R' U' R L' B' L R B R' L"
    },
    "106": {
        "name": "25a",
        "a": [
            "y2 S y' S y S",
            "y' x r' R' r' z' r' R' r R r' R' r B",
            "y2 x r R r R' r' R r R' z' R r' R' r R ",
            "x z' r B r' R' r B r' R r' B'"
        ],
        "s": "R' L U L' U' R L' U L U'"
    },
    "107": {
        "name": "25b",
        "a": [
            "y' S y S y' S",
            "y2 x r B R' B' R B R' B' R' r' R' "
        ],
        "s": "L' B' L' R B' R L' U' B L' B"
    },
    "108": {
        "name": "25c",
        "a": [
            "y H y H y' H",
            "y' x b' R r R' r' R r R' r z' r' R r'"
        ],
        "s": "L R' U' R U L' R U' R' U"
    },
    "109": {
        "name": "25d",
        "a": [
            "H y' H y H",
            "y2 x B' r' R r R' r' R r z r R r",
            "x z r' R r R' z' b' r' R r R' z' R r' R' r'"
        ],
        "s": "L B' L' R' L U L' B' U L R B'"
    },
    "110": {
        "name": "26a",
        "a": [
            "S y' S S (Cancel)",
            "S y H H",
            "x r' R r R' z R r' R' r R r' R' r"
        ],
        "s": "L' B R' B' L R L' R B' L"
    },
    "111": {
        "name": "26b",
        "a": [
            "y2 S S y' S (Cancel)",
            "S y' H H ",
            "x r R r' R' r B R' B' R' r' R'",
            "x B r' R r R z R r R' r' R r R"
        ],
        "s": "B' R' U R B' R' U R U' R' B'"
    },
    "112": {
        "name": "26c",
        "a": [
            "y' H y H H (Cancel)",
            "y2 S S y H",
            "x R r' R' r z R r' R' r R r' R' r"
        ],
        "s": "L' U L R' L R' U' R L' R"
    },
    "113": {
        "name": "26d",
        "a": [
            "y2 H y S S ",
            "y H H y H (Cancel)",
            "x z' r' B R' B' R' r' R' r R r' R' r'"
        ],
        "s": "U B' R' U' R B' L' R L' U' L"
    },
    "114": {
        "name": "27",
        "a": [
            "y' S S y' S S (cancel)",
            "y2 S S y S S",
            "y2 B r' R r R' B' z' R r' R' r",
            "y2 x r' R' z' r B r' R r y' r' R' r",
            "x z' B' r' R r R' B' z2 R r' R' r b'",
            "y' x r' R F' r' R r y2 z' r' R r R'",
            "y x B r' R' r B r' R r f' R B' R'",
            "x z r B R' B' R z2 B r' R r R' B"
        ],
        "s": "L R' B R L' R' L B' L' R"
    },
    "115": {
        "name": "28",
        "a": [
            "r' R' r z Sledge z' Hedge R",
            "S y' S S y' S y2 S (cancel)",
            "x z' r' R' r z r' R r R' r z' r' R' r R",
            "x z R' r' R r R' z R r' R' r z' r' R r",
            "x z r' R r x R r' R z R r' R' r",
            "x R r' R' x' r' R r z' r' R r R'",
            "x z r R' r' R r R' z' R r' R r B",
            "y2 x r B R B' R r' R r R' r' R"
        ],
        "s": "L' B' L R' L B' L B' L R"
    },
    "116": {
        "name": "29a",
        "a": [
            "r' R' r Hedge z' Hedge B",
            "y' S y' S y' H ",
            "y x r' R' r R r' R' r' z' r' R' r B",
            "y x r R r' R' z' r' R' r z r' R r' R'",
            "x z' B' r' R r R z R r R' r' R r",
            "x z2 R r R' r' R r R z R r R' b'",
            "x r' R' r R' B R' B' r' R' r R",
            "x z R r R' r z' r' R r z R r R' r'",
            "x z2 R' r' R r B R B' R r' R r"
        ],
        "s": "L' U' L R' B U' L R U R' B' R"
    },
    "117": {
        "name": "29b",
        "a": [
            "y S y' H y S y' H y H (Cancel)",
            "x R r' R r' R' z' r' R r B' R",
            "y' r' R r y' r l B r' l B'",
            "y z' r' R' r y z' r' R r b r' R' r"
        ],
        "s": "B' L R' U' R L U' B L' R"
    },
    "118": {
        "name": "30a",
        "a": [
            "Setup to triple sledge (Cancel)",
            "y2 x R r R' B' r' B' r B",
            "y2 x r B r' z' b' r' R' r b",
            "y2 x B' r' B r B R r' R'",
            "y' x R' r' R' r b z r B r' z' b' R",
            "y' x R' r' B' r B R r R' B' "
        ],
        "s": "L B' R' U B' R L B'"
    },
    "119": {
        "name": "30b",
        "a": [
            "Setup to triple sledge (Cancel)",
            "x z' r' R' r b z r B r' z' b' ",
            "x B' r B r' B' z' r' R' r b'",
            "y x B r' B' z' r' R' r b R",
            "y' x R r R' r' R r R' r2 R r R' r'",
            "x B r B' r' B' z' r' R r",
            "y2 x r R' B' r' B' r B R",
            "x z' r' B' r B R r R' B' "
        ],
        "s": "L' R U' L U' R' L U' L'"
    },
    "120": {
        "name": "31",
        "a": [
            "S S S",
            "y2 x b' r' R' r b z r B r' ",
            "y2 x b z r B r' B' z' r' R' r",
            "x z2 B R r R' B' r' B' r",
            "x z2 B' r' B' r B R r R'",
            "y' x R' B' r' B' r B R r",
            "y x r B R r R' B' r' B' "
        ],
        "s": "L' B' L' U R B R U'"
    },
    "121": {
        "name": "U-Perm",
        "a": [
            "y' x S y2 S",
            "y x b' r' R r R' z2 r' R r ",
            "y x' r' R' r z2 R r' R' r b"
        ],
        "s": "B' L' U R U' R' B R"
    },
    "122": {
        "name": "32",
        "a": [
            "y R L' U' L U L R L' R   ",
            "y x r' B r' B' z' r' R' r z B r'",
            "y x r/r2' B' r B R r R' B' r",
            "y x R2' B' r' B' r B R r R",
            "x r' B' r' B' r B R r R r",
            "y r l' B' l B l r l' r   ",
            "z r2' R' r b z r B r' z' b' r"
        ],
        "s": "L' R U R U' R' L' R' L'"
    },
    "123": {
        "name": "33",
        "a": [
            "S y2 H y2 S",
            "r2'/r R' r R' b' r2'/r B' r ",
            "r/r2' R' r R' x' B' R r' B",
            "r R' r R' y L' l L' l",
            "x r' R r R' B' r' B' r B r",
            "x R r' B' r B R r R' B' R'"
        ],
        "s": "L' B' U L R B R' L' U' L"
    },
    "124": {
        "name": "34a",
        "a": [
            "Setup to U Perm",
            "x b r' R r R' z2 r' R r R"
        ],
        "s": "L' R U B' U R' L R L' B U' R'"
    },
    "125": {
        "name": "34b",
        "a": [
            "Setup to U perm",
            "x B' r' R r R' z2 r' R r R' r ",
            "y x R' r' R' r z2 R r' R' r b'"
        ],
        "s": "U' R B R' B' R U R' U' L R' U"
    },
    "126": {
        "name": "35a",
        "a": [
            "y' S y2 S x z' S y2 S",
            "y' x r' R r R' r R y' r' R r B' r",
            "y' x b R r' b r R' b' r R' r' ",
            "y x r' R r R' b' R r' R r R' b' R"
        ],
        "s": "L B U' R U' R' U' B U' B"
    },
    "127": {
        "name": "35b",
        "a": [
            "S y2 S z' y' S y2 S",
            "y2 x R r' R' r B r' R r' R' r B r'",
            "x r B' r' R r R' r B' r' R r R'",
            "y x B' r' R r R' r R y' r' R r B' r'",
            "x z R r' b z' R B' r' R B' R' r "
        ],
        "s": "L U B' L' R' L R U' L' R' L B"
    },
    "128": {
        "name": "36a",
        "a": [
            "S y2 S y S y2 S",
            "H y2 H y H y2 H (cancel)",
            "y' x r' R' r B r' R r' R' r B r' R",
            "y x R' r B r' R r' R' r B r' R r'",
            "y x r' R r R' r B' r' R r R' r B'"
        ],
        "s": "L' R B' R' L' U L' R B R"
    },
    "129": {
        "name": "36b",
        "a": [
            "y' S y2 S y' S y2 S (cancel)",
            "S y2 S x' z S y2 S",
            "y' x B r' R r' R' r B r' R r' R' r",
            "x B' r' R' r B r' R r z' r' R r R'",
            "y' x B r' R r R' r R y' r' R r B'",
            "y2 x r' R r R' r z2 r' R r' R' r B r'"
        ],
        "s": "R L' B L R U' R L' B' L'"
    },
    "130": {
        "name": "37a",
        "a": [
            "H y H y' S y S (Cancel)",
            "S y S y' H y H (Cancel)",
            "x B' r' R r R' r B' r' R r R' r",
            "y' x r' R' r B r' R r z' r' R r R' z' r' ",
            "y2 x R r' R r R' z' r2' R' r B r' R"
        ],
        "s": "U' R' B' U L' U B' L R B' R'"
    },
    "131": {
        "name": "37b",
        "a": [
            "y' S y' H y H y' S (Cancel)",
            "y' H y' H y S y' S (Cancel)",
            "x r B r' R r' R' r B r' R r' R'",
            "y2 x r' R r' R' r B r' R r' R' r B ",
            "x r' R r z' r B' R r' R' B' r"
        ],
        "s": "L' B' R B R' B L' U' R U B' R"
    },
    "132": {
        "name": "EG2 H",
        "a": [
            "z' B R B' b R' r' R' r b' R",
            "y' z' b' r' R' r b' R B r R' r"
        ],
        "s": "L B' R' U L' U' L R U' B"
    },
    "133": {
        "name": "EG2 Z",
        "a": [
            "x' B R r R' r z R b z r B r'",
            "x' b' r' R' r R' z' r' B' z' R' b' R"
        ],
        "s": "L U B' R' U' R U B' R' L' B'"
    },
    "134": {
        "name": "EG2 U1",
        "a": [
            "y r' R r b R' z' r R r R'",
            "x z2 B R B' z' R r R' B' R' B' R"
        ],
        "s": "L' B' L' B' R B' U R U'"
    },
    "135": {
        "name": "EG2 TS2",
        "a": [
            "x R B' R r R r R' B' R"
        ],
        "s": "L' U' R U R' L R U R' L' U' L"
    },
    "136": {
        "name": "EG2 TS1",
        "a": [
            "x z R' r R' B' R' B' R r R'"
        ],
        "s": "R' L' U L U R U' R' U' R"
    },
    "137": {
        "name": "EG2 ZC2",
        "a": [
            "x b' R r R' B z' r' R r'",
            "x z' r2' R' r b' B R' z2 r' B"
        ],
        "s": "U' R' B R L' R L' U R B'"
    },
    "138": {
        "name": "EG2 ZC1",
        "a": [
            "x B' R B' b r' R r B'",
            "x B r' R' r b' B R' z2 r2'"
        ],
        "s": "R' U' L U B' R B' R' L' U B'"
    },
    "139": {
        "name": "EG2 O1",
        "a": [
            "x z' b' R r2' R r b' R r R'",
            "x r' R r B' R r R r B'"
        ],
        "s": "L U' R' U' L' R L' U' L"
    },
    "140": {
        "name": "EG2 O2",
        "a": [
            "x z2 B r' R' r' R' B r' R' r"
        ],
        "s": "R' U L U R L' R U R'"
    },
    "141": {
        "name": "EG2 U2",
        "a": [
            "x z r R' B' R B' r' z' r' R r R",
            "x z' r2' R r R B' R b' r' b"
        ],
        "s": "L' R B' L B L' R U' R' L' R U"
    },
    "142": {
        "name": "EG2 X1",
        "a": [
            "x z' B' R' B' R r R' B R B R' B' R",
            "x z' r' R' r B' r' R' r' R' B r'"
        ],
        "s": "L R' B' R U' L' R L' B U"
    },
    "143": {
        "name": "EG2 X2",
        "a": [
            "x z r R r R' B' R r' R' r' R r R'",
            "x z r B' R r2' R r B r' R r"
        ],
        "s": "B' L R U L' U R' B' R B'"
    },
    "144": {
        "name": "EG2 W1",
        "a": [
            "x r R' B r' R r' R' r R r'",
            "x r R' r' R r R' r B' R r'"
        ],
        "s": "L B U' R' L' R L R' B' R"
    },
    "145": {
        "name": "EG2 W2",
        "a": [
            "x z' r' R' r b' r' R r B R B'",
            "x R' r' R' r b' B R' z2 r2' R r"
        ],
        "s": "B' L' R L B L' U' B R B'"
    },
    "146": {
        "name": "EG2 S1",
        "a": [
            "x z' R' r B r' B' R r R' B' R",
            "x z' R' r' R r R b R B R B' R'"
        ],
        "s": "L' U L U' L' U R U' R' L"
    },
    "147": {
        "name": "EG2 S2",
        "a": [
            "x z R' r' R r b' B R B' R b'",
            "x R' B' R r2' R r B' r' R r R"
        ],
        "s": "R B R L' R' U' R B' U R' L' R"
    },
    "148": {
        "name": "EG2 Pi U + skip",
        "a": [
            "z r' R' r R' z' r2' R' r R",
            "x z2 R B r' R b' B r' R'"
        ],
        "s": "L' U' L R' U B' L U"
    },
    "149": {
        "name": "EG2 Pi U + H",
        "a": [
            "y' R' r' R r2 z R r' R r",
            "x z2 B' r' R r R r' z r2' R r R r'"
        ],
        "s": "L' U' B L' R L' R B"
    },
    "150": {
        "name": "EG2 Pi U + Z1",
        "a": [
            "x z' r B r' R r R' B z b' R",
            "x z r B r' B' r B r' R r R B R"
        ],
        "s": "L' U' R B L B' L' U' B U' L B"
    },
    "151": {
        "name": "EG2 Pi U + Z2",
        "a": [
            "x z2 R' r' R r z r B R r' R' r'",
            "x r R r' R' r' z' b' r' R r R"
        ],
        "s": "R L' U L' R' L R U L' R' B' R"
    },
    "152": {
        "name": "EG2 Pi U + U2r",
        "a": [
            "x z r' B' r B r' R' B' R B'",
            "x z' R r' R r z B r' B' r B"
        ],
        "s": "R' U' B R L R' U' R U' B"
    },
    "153": {
        "name": "EG2 Pi U + U2b",
        "a": [
            "x z r B' r' z' r' R' r R b",
            "x z r B' r' B r R r' R'"
        ],
        "s": "L' B' L' U L B L B'"
    },
    "154": {
        "name": "EG2 Pi U + U1b",
        "a": [
            "x z2 B r' R r R' B r' z r' R r R",
            "x z' r' R r' R' B' r B r' B'"
        ],
        "s": "R B' L' B' L' B L U' R L' U'"
    },
    "155": {
        "name": "EG2 Pi U + U1l",
        "a": [
            "x z B r R r' R' r' B' r",
            "x r' R' r R z B r' B' r"
        ],
        "s": "L R' B U' R U' B R U' R'"
    },
    "156": {
        "name": "EG2 Pi U + U1f",
        "a": [
            "x z' r' R' r R r z B r' B' r b'",
            "x R r' R r b' r' R r B R B'"
        ],
        "s": "L' B L R L' R' B R U' R'"
    },
    "157": {
        "name": "EG2 Pi U + U1r",
        "a": [
            "x z2 r' B r B' r' z' r' R r",
            "x z2 R r' R r z R' B' R r'"
        ],
        "s": "L' R' U R U B' U' R"
    },
    "158": {
        "name": "EG2 Pi U + U2l",
        "a": [
            "x z' b' R r z R r R' B' b'",
            "x y' r R r R' B' r' R' b"
        ],
        "s": "R' L' R B R' U' R U B U'"
    },
    "159": {
        "name": "EG2 Pi U + U2f",
        "a": [
            "x z2 R r R' r' B' r B r'",
            "x z b' R' r' R r R b R'"
        ],
        "s": "U' L' R' U' L' R B' R'"
    },
    "160": {
        "name": "EG2 Pi BR + X1",
        "a": [
            "x z' R r B' R r' B r' R r",
            "x z B R' B' R' r' R r b' B R' B'"
        ],
        "s": "L U B' R' B' R B R' U' R"
    },
    "161": {
        "name": "EG2 Pi BR + O2",
        "a": [
            "x r B R' B' R r B' R r' R'",
            "x R' B' R B' r B' r' B r'"
        ],
        "s": "L B' R B R' U' R U B' R'"
    },
    "162": {
        "name": "EG2 Pi BR + ZC2",
        "a": [
            "x z R' r' R' r b r' R r"
        ],
        "s": "B L U' R B' R' B U' L' B"
    },
    "163": {
        "name": "EG2 Pi BR + W2",
        "a": [
            "x R' r' R' r' b r' R"
        ],
        "s": "L' R B U B' L B' L' U L'"
    },
    "164": {
        "name": "EG2 Pi BR + O1",
        "a": [
            "x B R B' R b' z r B r' B",
            "x z B' r' R r R' B' r z r' R r"
        ],
        "s": "L U' L' R' U B' L' U L'"
    },
    "165": {
        "name": "EG2 Pi BR + X2",
        "a": [
            "x z B R r R' r' R r R' r B' r",
            "x z' b' B R' r b' r' R' B' R"
        ],
        "s": "R U' R' L' U' R B' U R' U L'"
    },
    "166": {
        "name": "EG2 Pi BR + W1",
        "a": [
            "x z2 r2' R r R B' R r'"
        ],
        "s": "L' B' R U R' U B' R B U'"
    },
    "167": {
        "name": "EG2 Pi BR + ZC1",
        "a": [
            "x z r R r R' B' R r' R'"
        ],
        "s": "L' U' L' R L B' R U' L'"
    },
    "168": {
        "name": "EG2 Pi BR + S1",
        "a": [
            "x B' R r' R' r z' r' R' r b'",
            "x z2 b r' R r R' z R r R' B"
        ],
        "s": "L' B' L' R U R L' U' L' U' R"
    },
    "169": {
        "name": "EG2 Pi BR + S2",
        "a": [
            "x z' B' r' R r R r' R' r' R r R B'",
            "y R r z r R r R' B r' B'"
        ],
        "s": "R L' U' L U' R U' L R B'"
    },
    "170": {
        "name": "EG2 Pi BR + TS1",
        "a": [
            "x R B' R r R r B'",
            "x z' r' R r' R' B z' r' R r"
        ],
        "s": "R L' U' L' R' U B'"
    },
    "171": {
        "name": "EG2 Pi BR + TS2",
        "a": [
            "x z R2 r R' B' R' B' r",
            "x z' R r' R r b' B R' B'"
        ],
        "s": "L U R' B L' U' B L'"
    },
    "172": {
        "name": "EG2 Pi FL + O1",
        "a": [
            "x z' B R' B' r R' B' R' B' R r R'",
            "z' B R' B' R' r R B R B' R'"
        ],
        "s": "U' L U L' B' U L U L' U'"
    },
    "173": {
        "name": "EG2 Pi FL + X2",
        "a": [
            "x r' R' r B' r R' B r' R'",
            "x z R B' R B' r' b' B r' R'"
        ],
        "s": "R' U' B R L' R L' B U' L"
    },
    "174": {
        "name": "EG2 Pi FL + W1",
        "a": [
            "x z r R' B R B' r' B' r B r'",
            "y2 z' R' r B b R' z' R r' R'"
        ],
        "s": "L' R L' U' L U B' L' U'"
    },
    "175": {
        "name": "EG2 Pi FL + ZC1",
        "a": [
            "x z2 R r R' r' z' b r' R r b",
            "x z b' R r R' b' r' R' r b' R r'"
        ],
        "s": "R B L' U' B L' U R L"
    },
    "176": {
        "name": "EG2 Pi FL + X1",
        "a": [
            "x z2 R' B' R' r' R' r R r R' B r'",
            "x z B R' r z' b' R z R' r B R' r'"
        ],
        "s": "L' U' R U R L R' L' B' L U' L"
    },
    "177": {
        "name": "EG2 Pi FL + O2",
        "a": [
            "z2 r' R r R B' R' r' R' r R",
            "x z2 r' R r B' R r R r R' B' R"
        ],
        "s": "L B L' U' B L B L' B' U R"
    },
    "178": {
        "name": "EG2 Pi FL + ZC2",
        "a": [
            "x z' b B' R r R' b' B r'",
            "x r' R' r R z B' R r' R' B'"
        ],
        "s": "L' B' L R' B' R L' R U' L' B"
    },
    "179": {
        "name": "EG2 Pi FL + W2",
        "a": [
            "x z' R r R' B' r B r' R' B' R",
            "x z B R' r' B' r B' R B"
        ],
        "s": "R' B R L' B' R U R B' R' L' R"
    },
    "180": {
        "name": "EG2 Pi FL + S1 perm",
        "a": [
            "x z2 r' R B R r' R r B r'",
            "x z2 R' r R r B' R r' R r' R r"
        ],
        "s": "L' B' R L' B' L R' B L R"
    },
    "181": {
        "name": "EG2 Pi FL + S2 perm",
        "a": [
            "x z B r' R' r b' r' B R' r'",
            "x z b' R r z r' R r y R' r B"
        ],
        "s": "L U' L' R B' R' L' B' R L' B"
    },
    "182": {
        "name": "EG2 Pi FL + TS2",
        "a": [
            "x z2 b' r' R r R' r' R' r b'"
        ],
        "s": "R U L' U' B L B' L' U' B' L'"
    },
    "183": {
        "name": "EG2 Pi FL + TS1",
        "a": [
            "x B R r' R' r R r R' B"
        ],
        "s": "L R' L' R L' B' R U' R'"
    },
    "184": {
        "name": "EG2 Pi BL + X1",
        "a": [
            "x r' R r' R' B R r' R' r'",
            "x z2 r R' r' R r R z B b' R r R'"
        ],
        "s": "L B U' R' B L' U L'"
    },
    "185": {
        "name": "EG2 Pi BL + O2",
        "a": [
            "x z b R r' R r b' R",
            "x z' r R r' B r' B' R' r"
        ],
        "s": "R B R' B R U' R' L'"
    },
    "186": {
        "name": "EG2 Pi BL + W1",
        "a": [
            "x z2 b' r' R' r b' z r R r R' r",
            "x b' r' R' r b' R B R' B'"
        ],
        "s": "L' B' R U' L U' L' R' U"
    },
    "187": {
        "name": "EG2 Pi BL + ZC1",
        "a": [
            "x z2 R r' R' B' z r' R r R b R'"
        ],
        "s": "R' U' B R' U L B U'"
    },
    "188": {
        "name": "EG2 Pi BL + O1",
        "a": [
            "x z B R B' r b r' R",
            "x z R B' b B r' R r"
        ],
        "s": "L' R' U L' B' L R'"
    },
    "189": {
        "name": "EG2 Pi BL + X2",
        "a": [
            "x z' r R r R' B' R r R' r"
        ],
        "s": "R U R' U' R L' B' L' U'"
    },
    "190": {
        "name": "EG2  Pi BL + ZC2",
        "a": [
            "x r' R' r b' r' R r R"
        ],
        "s": "B' L' U' L U L' U L"
    },
    "191": {
        "name": "EG2 Pi BL + W2",
        "a": [
            "x z R' r b' r2' R r R"
        ],
        "s": "R L' U' L R' L' R U B' L' U'"
    },
    "192": {
        "name": "EG2 Pi BL + S1",
        "a": [
            "x z2 r R r R' z' b' r' R r R",
            "x z2 B' r' R r R' B' R r R' B"
        ],
        "s": "L' B R' U' R B L R' L' U' L R"
    },
    "193": {
        "name": "EG2 Pi BL + S2",
        "a": [
            "x R' r B r' B' R r B'",
            "x z' B r' R' B r B' r' R"
        ],
        "s": "R' U L U' L' R L B'"
    },
    "194": {
        "name": "EG2 Pi BL + TS1",
        "a": [
            "x z' B r' R' r' R' B R'",
            "x z' r' R' r b' B R B' R"
        ],
        "s": "L R' B R L' R U R L' B R'"
    },
    "195": {
        "name": "EG2 Pi BL + TS2",
        "a": [
            "x z' b r' R r R r' R' r b",
            "y x' r B R' B' R B R B' r"
        ],
        "s": "L' U' L' R B R' B L' R U"
    },
    "196": {
        "name": "EG2 Pi FR + O1",
        "a": [
            "x z2 R' r' R r z B' r B r'",
            "x z B' r' R r' R' B r'"
        ],
        "s": "R B R U' R' L' R B R"
    },
    "197": {
        "name": "EG2 Pi FR + X2",
        "a": [
            "x z2 R r' R r b' r' R r R",
            "x R' r' R r R' z2 R r' R r b'"
        ],
        "s": "L U' L' B U' B L U R"
    },
    "198": {
        "name": "EG2 Pi FR + W1",
        "a": [
            "x z r R' B R' r' R' r'",
            "x z2 r' R' r b' r B R' B' r'"
        ],
        "s": "B L R L B' U B'"
    },
    "199": {
        "name": "EG2 Pi FR + ZC1",
        "a": [
            "x z2 R r R' B R r' R' r'"
        ],
        "s": "R U R L' B' L R' U'"
    },
    "200": {
        "name": "EG2 Pi FR + X1",
        "a": [
            "x z' R' r' R' r b r' R' r R'",
            "x z' b r' R' r R' z2 R r' R' r R"
        ],
        "s": "L' B' R L R' L' U' R B' L' B"
    },
    "201": {
        "name": "EG2 Pi FR + O2",
        "a": [
            "x r' R' r z2 r' R' r b'",
            "x z' B' R r' R' z B R' B'"
        ],
        "s": "U L U' R B U' B"
    },
    "202": {
        "name": "EG2 Pi FR + ZC2",
        "a": [
            "x z r R' B' r B r' R B'",
            "x' r' R' r B r' R r R' z' R r' R' r"
        ],
        "s": "L' R L' B' L R' L R"
    },
    "203": {
        "name": "EG2 Pi FR + W2",
        "a": [
            "y z B' R' b r' R r R B'",
            "x z' R' r' R b' R b r R'"
        ],
        "s": "L' U' R L' U R L' R B' U"
    },
    "204": {
        "name": "EG2 Pi FR + S1",
        "a": [
            "x z' r B R' B' R r R' B' R r'",
            "x R' r' R' r b B R' B' R'"
        ],
        "s": "L' B' R U R' U' R U R' U"
    },
    "205": {
        "name": "EG2 Pi FR + S2",
        "a": [
            "x r b' B R' z2 r' B r' R' r B",
            "x' R b' r' R r R' z' R2 r"
        ],
        "s": "B' U R U' L' U L R'"
    },
    "206": {
        "name": "EG2 Pi FR + TS2",
        "a": [
            "x z' b' R r2' R r b' r",
            "x z2 B R B' b r' R' r R'"
        ],
        "s": "B' L R' U' L' U' R"
    },
    "207": {
        "name": "EG2 Pi FR + TS1",
        "a": [
            "x z' B' R r' R' r' R r R' B'",
            "x z' b R r R' r' R r R r' R' b"
        ],
        "s": "L B' L' R B' R' L' R B R' L R"
    },
    "208": {
        "name": "EG2 Peanut U + skip",
        "a": [
            "x z r R' r' R r z R r R r' R' b",
            "x z2 b' R r R' r' R' z' r' R' r R r'"
        ],
        "s": "R B' R' L' R L' B L R L'"
    },
    "209": {
        "name": "EG2 Peanut U + H",
        "a": [
            "x r' R' r b' B R' B'",
            "x z R r R' B z' r' R r"
        ],
        "s": "L' U' L U' R U' R'"
    },
    "210": {
        "name": "EG2 Peanut U + Z1",
        "a": [
            "x z2 b' r' R r B R B' b'",
            "x z' B R r' R' z' r' R' r b"
        ],
        "s": "L R' L' U' L U L' B' L B"
    },
    "211": {
        "name": "EG2 Peanut U + Z2",
        "a": [
            "y' z' R' r' R' r b' R' z' R r' R",
            "x z B R B' R' r' R r b r' R r"
        ],
        "s": "L' B' L' R L' B' L' R B' R'"
    },
    "212": {
        "name": "EG2 Peanut U + U2r",
        "a": [
            "x z r' R' r R r R' r' B' r B r' R",
            "r' R r B' R B b' R r"
        ],
        "s": "B' L' R L' B' L' B' R'"
    },
    "213": {
        "name": "EG2 Peanut U + U2b",
        "a": [
            "x R' B' r R' B' R' B' r'",
            "y2 x' B R' B' z' R' r B r' R"
        ],
        "s": "L B R' U' R U' R' U B' R"
    },
    "214": {
        "name": "EG2 Peanut U + U1b",
        "a": [
            "x R r B' R r2' R r B",
            "x z B' b R r R' b' B r'"
        ],
        "s": "R' B' L U L' U L U' B L'"
    },
    "215": {
        "name": "EG2 Peanut U + U1l",
        "a": [
            "x z2 B' r' R' r' R' B r' R'",
            "y' z' B R' r R B' R' r' R"
        ],
        "s": "R L' U B' R' U R B U' R'"
    },
    "216": {
        "name": "EG2 Peanut U + U1f",
        "a": [
            "x z R r R' B' R' B' R r"
        ],
        "s": "L R' U R B' U B' L' B' L"
    },
    "217": {
        "name": "EG2 Peanut U + U1r",
        "a": [
            "x z2 b' r' R r R z' R r' R'"
        ],
        "s": "L' U' R U R' L R U' R' L' U L"
    },
    "218": {
        "name": "EG2 Peanut U + U2l",
        "a": [
            "x z' B R r' R' r' z r' R r"
        ],
        "s": "R L U' L' R' U' R U R' B' R B"
    },
    "219": {
        "name": "EG2 Peanut U + U2f",
        "a": [
            "x r' R' r z' r R r R' B'"
        ],
        "s": "B R B' L' U' L' U L"
    },
    "220": {
        "name": "EG2 Peanut BR + X1",
        "a": [
            "x z' r B r' B' R r R' B'",
            "x z2 r' R' r b' R' r' R' r R"
        ],
        "s": "U' R U L' U' L U R'"
    },
    "221": {
        "name": "EG2 Peanut BR + O2",
        "a": [
            "x z r2' R r B' r' R r R r'",
            "x z' B' R r2' R r B' r' R r"
        ],
        "s": "R L B' L' B' L U L' B' L' R'"
    },
    "222": {
        "name": "EG2 Peanut BR + ZC2",
        "a": [
            "x z R' r R' B R' r' R' r R r",
            "x z' r B r' B' r z2 R r' R'"
        ],
        "s": "R L' B' R' U' R U R' L' R B L"
    },
    "223": {
        "name": "EG2 Peanut BR + W2",
        "a": [
            "x R r R' r z R r' R r b'"
        ],
        "s": "L' R' L B U' L' U L U B'"
    },
    "224": {
        "name": "EG2 Peanut BR + O1",
        "a": [
            "x z2 R' r' R' B r' R' r' B'",
            "x z B R' B' z' R r' R' r' R r R' B'"
        ],
        "s": "U L B L' B U B L"
    },
    "225": {
        "name": "EG2 Peanut BR + X2",
        "a": [
            "x R r R' B R r' R' r R r R'",
            "x z2 R' B' R B' r' z' r' R r R b"
        ],
        "s": "R' B' R B L' R L' U' L'"
    },
    "226": {
        "name": "EG2 Peanut BR + W1",
        "a": [
            "z x2 r B R B' r2' z' b' R r R'",
            "x z' B' r' R r R r' z B R B' R r"
        ],
        "s": "L U' R' U' R B' U' B R' U' R"
    },
    "227": {
        "name": "EG2 Peanut BR + ZC1",
        "a": [
            "x z2 R' r b' B R' B' r'",
            "x z2 r B R B' z' r R' B"
        ],
        "s": "U' R U' B L' B' R'"
    },
    "228": {
        "name": "EG2 Peanut BR + S1",
        "a": [
            "x' r' B' r B r' R' r R r R'",
            "x z2 R' r' R r R b' R b R' b'"
        ],
        "s": "R B L' R' U' R U B' R' U"
    },
    "229": {
        "name": "EG2 Peanut BR + S2",
        "a": [
            "y' z' r B R' B' R r R' b' B r'",
            "x z' R r R' B R B R r' R B"
        ],
        "s": "L' B' R U R' U' R B U' L"
    },
    "230": {
        "name": "EG2 Peanut BR + TS1",
        "a": [
            "x r' R' r' R' B r' R' r'"
        ],
        "s": "R B R' B R L' B L U'"
    },
    "231": {
        "name": "EG2 Peanut BR + TS2",
        "a": [
            "x z' r' R' r b' B R' z2 r2' R r R'",
            "x z B R' B' r b' r' B R' B'"
        ],
        "s": "L' B' L' R' L R' B U' L"
    },
    "232": {
        "name": "EG2 Peanut FL + O1",
        "a": [
            "x z' r R' B' R' B' R r R",
            "x z' r R' r' R r R r' b r' R r"
        ],
        "s": "L' R' L' R L U L U'"
    },
    "233": {
        "name": "EG2 Peanut FL + X2",
        "a": [
            "x z' r' R r R r' R' r b r' R r",
            "x z r' B R r' R r B r' R"
        ],
        "s": "L R' L' U' B L B' L' R L"
    },
    "234": {
        "name": "EG2 Peanut FL + W1",
        "a": [
            "x z' R' r z r' R' r R r R' b' R",
            "x r' R' r R' z2 r' R' r b' R"
        ],
        "s": "L' B' R U' L U' L' U' B U' L"
    },
    "235": {
        "name": "EG2 Peanut FL + ZC1",
        "a": [
            "x z2 b r' R' r R r R' b' R r'"
        ],
        "s": "L' U' L B' L' B' U B' U"
    },
    "236": {
        "name": "EG2 Peanut FL + X1",
        "a": [
            "x z2 b' r' R r b' R' b R",
            "x z' r R r' R' r' B' R r R'"
        ],
        "s": "L' B' R U' R' U R B L U'"
    },
    "237": {
        "name": "EG2 Peanut FL + O2",
        "a": [
            "x z' r' B' r B' r B' r'",
            "x z' R' r b' r2' R r B R B'"
        ],
        "s": "L U L' R' U B' U R L U' B"
    },
    "238": {
        "name": "EG2 Peanut FL + ZC2",
        "a": [
            "x z R r R' B' r B r' B'",
            "x R r R' r' R' r b' r' R r R r'"
        ],
        "s": "U B L' B' U R U' R'"
    },
    "239": {
        "name": "EG2 Peanut FL + W2",
        "a": [
            "x z2 r R' B' R B' r' R r' R'",
            "x B r' B' r B r' R r R B' R"
        ],
        "s": "L' R B U' R' U R L U' L' B' L"
    },
    "240": {
        "name": "EG2 Peanut FL + S1",
        "a": [
            "x z2 R r' R' r R' r b' r2' R r R",
            "x z' r2' R r R B' R r' R r' R' r"
        ],
        "s": "R L' U R B' R' L R' B' R L B'"
    },
    "241": {
        "name": "EG2 Peanut FL + S2",
        "a": [
            "x r' R r R B' R r' z r' R r R",
            "x z2 b r' R' r R' z R r' R' r B"
        ],
        "s": "U' R L' B' R B' R B' R'"
    },
    "242": {
        "name": "EG2 Peanut FL + TS2",
        "a": [
            "x z' B R B' b r' R r R r' R' r",
            "x r' R r B R' B' z B r' B' r"
        ],
        "s": "L' R B L U' L' B' U R' U' R L"
    },
    "243": {
        "name": "EG2 Peanut FL + TS1",
        "a": [
            "x z2 r R r B' R r R r",
            "x z' r' R r R r R' B' R' B' r"
        ],
        "s": "L' B L B' L' U' R L' R'"
    },
    "244": {
        "name": "EG2 Peanut BL + X1",
        "a": [
            "x z' r' R' r z' r2' R r R B' R",
            "x z r' R' r b' r' R r R' r' R' r"
        ],
        "s": "L B L' B' R L' R U R"
    },
    "245": {
        "name": "EG2 Peanut BL + O2",
        "a": [
            "x B r2' R r B' R r R",
            "x z' B R B' r R' B r' R' r R r"
        ],
        "s": "L' B' U' B' L B' L' U'"
    },
    "246": {
        "name": "EG2 Peanut BL + W1",
        "a": [
            "x z r' R' r R' z' r' R r' R' B",
            "x z2 R r R' r B r' R r R' B r'"
        ],
        "s": "L' R B R' B L U' L B"
    },
    "247": {
        "name": "EG2 Peanut BL + ZC1",
        "a": [
            "x z' r' B' r B r' z r' R r",
            "x z' R r R' r' R r R B' R r' B"
        ],
        "s": "L' R U' R U R' B' R B R' U' L"
    },
    "248": {
        "name": "EG2 Peanut BL + O1",
        "a": [
            "x z R r' R r z B' R r R' r",
            "x z r R' r' R' r B r' R' r'"
        ],
        "s": "L' R' B R B R' U' R B R L"
    },
    "249": {
        "name": "EG2 Peanut BL + X2",
        "a": [
            "x r' R b R' b' R r b'",
            "y' x' r' R r R' z2 r' b R B'"
        ],
        "s": "U L' U' R U R' U' L"
    },
    "250": {
        "name": "EG2 Peanut BL + ZC2",
        "a": [
            "x z2 B' r' R' r b' B R'",
            "x z2 R B' b r' R r B"
        ],
        "s": "U L' U B' R B L"
    },
    "251": {
        "name": "EG2 Peanut BL + W2",
        "a": [
            "x z b' r' R r R' r R' r z' r2' R r",
            "x z R B' r' R r R r' z B R B' R"
        ],
        "s": "U' L U B' U L' B R L"
    },
    "252": {
        "name": "EG2 Peanut BL + S1",
        "a": [
            "x r2' R r R B' R r R r R'",
            "x z b' R r R' B r R r R' r' b"
        ],
        "s": "R B L' U' L U L' B' U R'"
    },
    "253": {
        "name": "EG2 Peanut BL + S2",
        "a": [
            "x z' B' r' R r R' z' R r' R r b'"
        ],
        "s": "L' B' R L U L' U' B L U'"
    },
    "254": {
        "name": "EG2 Peanut BL + TS1",
        "a": [
            "x z' r' R r B' b B r' R r",
            "x z2 r b' B R' z2 B R' B' r' B'"
        ],
        "s": "R B R L R' L B' U R'"
    },
    "255": {
        "name": "EG2 Peanut BL + TS2",
        "a": [
            "x z R r2' R r b' R r R",
            "x z B' R' B R B' R r R' B r'"
        ],
        "s": "L' B' L B' L' R B' R' U"
    },
    "256": {
        "name": "EG2 Peanut FR + O1",
        "a": [
            "x z' r B r' B r' B r",
            "x y' R r R' B x r' R r"
        ],
        "s": "L B' L' R' U L B' L R B'"
    },
    "257": {
        "name": "EG2 Peanut FR + X2",
        "a": [
            "x z2 R' r' R r R b r' R r",
            "x z' B R r' R' B r B' r'"
        ],
        "s": "R B L' U L U' L' B' R' U"
    },
    "258": {
        "name": "EG2 Peanut FR + W1",
        "a": [
            "x z2 B' R r R' r z R r' R r"
        ],
        "s": "R L' B' U L U' L' R' U R B R'"
    },
    "259": {
        "name": "EG2 Peanut FR + ZC1",
        "a": [
            "x z' R' B' R r B' r' B r"
        ],
        "s": "U' B' R B U' L' U L"
    },
    "260": {
        "name": "EG2 Peanut FR + X1",
        "a": [
            "x z2 R r' R' r' R r R' B' R r' R'",
            "x z2 R' B' R' r R' B R B R' B' R r'"
        ],
        "s": "R' L R U B' R' B R L' R'"
    },
    "261": {
        "name": "EG2 Peanut FR + O2",
        "a": [
            "x z B' R r R r R' B' R'",
            "x z' R' r' z r' R r R r b'"
        ],
        "s": "U' L R B R' L R' U' R B'"
    },
    "262": {
        "name": "EG2 Peanut FR + ZC2",
        "a": [
            "x z' B' R r R' r' R' r B r' R",
            "x z' R' r B' r' R r R r' R' B"
        ],
        "s": "R U R' B R B U' B U'"
    },
    "263": {
        "name": "EG2 Peanut FR + W2",
        "a": [
            "x z R B' R r R' r' R' r B r'",
            "x z r R r R' B' R B R B' r' R"
        ],
        "s": "L R' U R B' L' R' L U' B"
    },
    "264": {
        "name": "EG2 Peanut FR + S1",
        "a": [
            "x z' r' R r B R B' z r B r' B' r"
        ],
        "s": "L' R U' L' B L R' L B L' R' B"
    },
    "265": {
        "name": "EG2 Peanut FR + S2",
        "a": [
            "x z' B' R r R' r z' r' R r R' b'"
        ],
        "s": "L' R U' B R' U R U R' L R' B'"
    },
    "266": {
        "name": "EG2 Peanut FR + TS2",
        "a": [
            "x z2 B' R' B' r R' B' R' B'",
            "x z R' r' R r R b r' R' r R'"
        ],
        "s": "R B' R' B R U L' R L"
    },
    "267": {
        "name": "EG2 Peanut FR + TS1",
        "a": [
            "x z r' R' r b' z R r' R' r' R r R'",
            "x R r' R' r R r R' z' b r' R r"
        ],
        "s": "L U' L U' L R' L R U' L'"
    }
};