
use crate::types::BitBoard;

use lazy_static::lazy_static;
use bitflags::bitflags;

bitflags! {
    /// Table layout flags.
    struct Layout: u8 {
        /// Two sided table for non-symmetrical material configuration.
        const SPLIT = 1;
        /// Table with pawns. Has subtables for each leading pawn file (a-d).
        const HAS_PAWNS = 2;
    }
}

bitflags! {
    /// Subtable format flags.
    struct Flag: u8 {
        /// DTZ table stores black to move.
        const STM = 1;
        /// Use `DtzMap`.
        const MAPPED = 2;
        /// DTZ table has winning positions on the edge of the 50-move rule and
        /// therefore stores exact plies rather than just full moves.
        const WIN_PLIES = 4;
        /// DTZ table has losing positions on the edge of the 50-move rule and
        /// therefore stores exact plies rather than just full moves.
        const LOSS_PLIES = 8;
        /// DTZ table contains very long endgames, so that values require 16
        /// bits rather than just 8.
        const WIDE_DTZ = 16;
        /// Table stores only a single value.
        const SINGLE_VALUE = 128;
    }
}

/// Maximum size in bytes of a compressed block.
const MAX_BLOCK_SIZE: usize = 1024;

/// Maps squares into the a1-d1-d4 triangle.
const TRIANGLE: [u64; 64] = [
    6, 0, 1, 2, 2, 1, 0, 6,
    0, 7, 3, 4, 4, 3, 7, 0,
    1, 3, 8, 5, 5, 8, 3, 1,
    2, 4, 5, 9, 9, 5, 4, 2,
    2, 4, 5, 9, 9, 5, 4, 2,
    1, 3, 8, 5, 5, 8, 3, 1,
    0, 7, 3, 4, 4, 3, 7, 0,
    6, 0, 1, 2, 2, 1, 0, 6,
];

/// Inverse of `TRIANGLE`.
const INV_TRIANGLE: [usize; 10] = [1, 2, 3, 10, 11, 19, 0, 9, 18, 27];

/// Maps the b1-h1-h7 triangle to `0..=27`.
const LOWER: [u64; 64] = [
    28,  0,  1,  2,  3,  4,  5,  6,
    0, 29,  7,  8,  9, 10, 11, 12,
    1,  7, 30, 13, 14, 15, 16, 17,
    2,  8, 13, 31, 18, 19, 20, 21,
    3,  9, 14, 18, 32, 22, 23, 24,
    4, 10, 15, 19, 22, 33, 25, 26,
    5, 11, 16, 20, 23, 25, 34, 27,
    6, 12, 17, 21, 24, 26, 27, 35,
];

/// Used to initialize `Consts::mult_idx` and `Consts::mult_factor`.
const MULT_TWIST: [u64; 64] = [
    15, 63, 55, 47, 40, 48, 56, 12,
    62, 11, 39, 31, 24, 32,  8, 57,
    54, 38,  7, 23, 16,  4, 33, 49,
    46, 30, 22,  3,  0, 17, 25, 41,
    45, 29, 21,  2,  1, 18, 26, 42,
    53, 37,  6, 20, 19,  5, 34, 50,
    61, 10, 36, 28, 27, 35,  9, 58,
    14, 60, 52, 44, 43, 51, 59, 13,
];

/// Unused entry. Initialized to `-1`, so that most uses will cause noticable
/// overflow in debug mode.
const Z0: u64 = u64::max_value();

/// Encoding of all 461 configurations of two not-connected kings.
const KK_IDX: [[u64; 64]; 10] = [[
    Z0,  Z0,  Z0,   0,   1,   2,   3,   4,
    Z0,  Z0,  Z0,   5,   6,   7,   8,   9,
    10,  11,  12,  13,  14,  15,  16,  17,
    18,  19,  20,  21,  22,  23,  24,  25,
    26,  27,  28,  29,  30,  31,  32,  33,
    34,  35,  36,  37,  38,  39,  40,  41,
    42,  43,  44,  45,  46,  47,  48,  49,
    50,  51,  52,  53,  54,  55,  56,  57,
], [
    58,  Z0,  Z0,  Z0,  59,  60,  61,  62,
    63,  Z0,  Z0,  Z0,  64,  65,  66,  67,
    68,  69,  70,  71,  72,  73,  74,  75,
    76,  77,  78,  79,  80,  81,  82,  83,
    84,  85,  86,  87,  88,  89,  90,  91,
    92,  93,  94,  95,  96,  97,  98,  99,
    100, 101, 102, 103, 104, 105, 106, 107,
    108, 109, 110, 111, 112, 113, 114, 115,
], [
    116, 117,  Z0,  Z0,  Z0, 118, 119, 120,
    121, 122,  Z0,  Z0,  Z0, 123, 124, 125,
    126, 127, 128, 129, 130, 131, 132, 133,
    134, 135, 136, 137, 138, 139, 140, 141,
    142, 143, 144, 145, 146, 147, 148, 149,
    150, 151, 152, 153, 154, 155, 156, 157,
    158, 159, 160, 161, 162, 163, 164, 165,
    166, 167, 168, 169, 170, 171, 172, 173,
], [
    174,  Z0,  Z0,  Z0, 175, 176, 177, 178,
    179,  Z0,  Z0,  Z0, 180, 181, 182, 183,
    184,  Z0,  Z0,  Z0, 185, 186, 187, 188,
    189, 190, 191, 192, 193, 194, 195, 196,
    197, 198, 199, 200, 201, 202, 203, 204,
    205, 206, 207, 208, 209, 210, 211, 212,
    213, 214, 215, 216, 217, 218, 219, 220,
    221, 222, 223, 224, 225, 226, 227, 228,
], [
    229, 230,  Z0,  Z0,  Z0, 231, 232, 233,
    234, 235,  Z0,  Z0,  Z0, 236, 237, 238,
    239, 240,  Z0,  Z0,  Z0, 241, 242, 243,
    244, 245, 246, 247, 248, 249, 250, 251,
    252, 253, 254, 255, 256, 257, 258, 259,
    260, 261, 262, 263, 264, 265, 266, 267,
    268, 269, 270, 271, 272, 273, 274, 275,
    276, 277, 278, 279, 280, 281, 282, 283,
], [
    284, 285, 286, 287, 288, 289, 290, 291,
    292, 293,  Z0,  Z0,  Z0, 294, 295, 296,
    297, 298,  Z0,  Z0,  Z0, 299, 300, 301,
    302, 303,  Z0,  Z0,  Z0, 304, 305, 306,
    307, 308, 309, 310, 311, 312, 313, 314,
    315, 316, 317, 318, 319, 320, 321, 322,
    323, 324, 325, 326, 327, 328, 329, 330,
    331, 332, 333, 334, 335, 336, 337, 338,
], [
    Z0,  Z0, 339, 340, 341, 342, 343, 344,
    Z0,  Z0, 345, 346, 347, 348, 349, 350,
    Z0,  Z0, 441, 351, 352, 353, 354, 355,
    Z0,  Z0,  Z0, 442, 356, 357, 358, 359,
    Z0,  Z0,  Z0,  Z0, 443, 360, 361, 362,
    Z0,  Z0,  Z0,  Z0,  Z0, 444, 363, 364,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 445, 365,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 446,
], [
    Z0,  Z0,  Z0, 366, 367, 368, 369, 370,
    Z0,  Z0,  Z0, 371, 372, 373, 374, 375,
    Z0,  Z0,  Z0, 376, 377, 378, 379, 380,
    Z0,  Z0,  Z0, 447, 381, 382, 383, 384,
    Z0,  Z0,  Z0,  Z0, 448, 385, 386, 387,
    Z0,  Z0,  Z0,  Z0,  Z0, 449, 388, 389,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 450, 390,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 451,
], [
    452, 391, 392, 393, 394, 395, 396, 397,
    Z0,  Z0,  Z0,  Z0, 398, 399, 400, 401,
    Z0,  Z0,  Z0,  Z0, 402, 403, 404, 405,
    Z0,  Z0,  Z0,  Z0, 406, 407, 408, 409,
    Z0,  Z0,  Z0,  Z0, 453, 410, 411, 412,
    Z0,  Z0,  Z0,  Z0,  Z0, 454, 413, 414,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 455, 415,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 456,
], [
    457, 416, 417, 418, 419, 420, 421, 422,
    Z0, 458, 423, 424, 425, 426, 427, 428,
    Z0,  Z0,  Z0,  Z0,  Z0, 429, 430, 431,
    Z0,  Z0,  Z0,  Z0,  Z0, 432, 433, 434,
    Z0,  Z0,  Z0,  Z0,  Z0, 435, 436, 437,
    Z0,  Z0,  Z0,  Z0,  Z0, 459, 438, 439,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 460, 440,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 461,
]];

/// Encoding of a pair of identical pieces.
const PP_IDX: [[u64; 64]; 10] = [
[
    0,  Z0,   1,   2,   3,   4,   5,   6,
    7,   8,   9,  10,  11,  12,  13,  14,
    15,  16,  17,  18,  19,  20,  21,  22,
    23,  24,  25,  26,  27,  28,  29,  30,
    31,  32,  33,  34,  35,  36,  37,  38,
    39,  40,  41,  42,  43,  44,  45,  46,
    Z0,  47,  48,  49,  50,  51,  52,  53,
    54,  55,  56,  57,  58,  59,  60,  61,
], [
    62,  Z0,  Z0,  63,  64,  65,  Z0,  66,
    Z0,  67,  68,  69,  70,  71,  72,  Z0,
    73,  74,  75,  76,  77,  78,  79,  80,
    81,  82,  83,  84,  85,  86,  87,  88,
    89,  90,  91,  92,  93,  94,  95,  96,
    Z0,  97,  98,  99, 100, 101, 102, 103,
    Z0, 104, 105, 106, 107, 108, 109,  Z0,
    110,  Z0, 111, 112, 113, 114,  Z0, 115,
], [
    116,  Z0,  Z0,  Z0, 117,  Z0,  Z0, 118,
    Z0, 119, 120, 121, 122, 123, 124,  Z0,
    Z0, 125, 126, 127, 128, 129, 130,  Z0,
    131, 132, 133, 134, 135, 136, 137, 138,
    Z0, 139, 140, 141, 142, 143, 144, 145,
    Z0, 146, 147, 148, 149, 150, 151,  Z0,
    Z0, 152, 153, 154, 155, 156, 157,  Z0,
    158,  Z0,  Z0, 159, 160,  Z0,  Z0, 161,
], [
    162,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 163,
    Z0, 164,  Z0, 165, 166, 167, 168,  Z0,
    Z0, 169, 170, 171, 172, 173, 174,  Z0,
    Z0, 175, 176, 177, 178, 179, 180,  Z0,
    Z0, 181, 182, 183, 184, 185, 186,  Z0,
    Z0,  Z0, 187, 188, 189, 190, 191,  Z0,
    Z0, 192, 193, 194, 195, 196, 197,  Z0,
    198,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 199,
], [
    200,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 201,
    Z0, 202,  Z0,  Z0, 203,  Z0, 204,  Z0,
    Z0,  Z0, 205, 206, 207, 208,  Z0,  Z0,
    Z0, 209, 210, 211, 212, 213, 214,  Z0,
    Z0,  Z0, 215, 216, 217, 218, 219,  Z0,
    Z0,  Z0, 220, 221, 222, 223,  Z0,  Z0,
    Z0, 224,  Z0, 225, 226,  Z0, 227,  Z0,
    228,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 229,
], [
    230,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 231,
    Z0, 232,  Z0,  Z0,  Z0,  Z0, 233,  Z0,
    Z0,  Z0, 234,  Z0, 235, 236,  Z0,  Z0,
    Z0,  Z0, 237, 238, 239, 240,  Z0,  Z0,
    Z0,  Z0,  Z0, 241, 242, 243,  Z0,  Z0,
    Z0,  Z0, 244, 245, 246, 247,  Z0,  Z0,
    Z0, 248,  Z0,  Z0,  Z0,  Z0, 249,  Z0,
    250,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 251,
], [
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 259,
    Z0, 252,  Z0,  Z0,  Z0,  Z0, 260,  Z0,
    Z0,  Z0, 253,  Z0,  Z0, 261,  Z0,  Z0,
    Z0,  Z0,  Z0, 254, 262,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0, 255,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0, 256,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 257,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 258,
], [
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 268,  Z0,
    Z0,  Z0, 263,  Z0,  Z0, 269,  Z0,  Z0,
    Z0,  Z0,  Z0, 264, 270,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0, 265,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0, 266,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0, 267,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
], [
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0, 274,  Z0,  Z0,
    Z0,  Z0,  Z0, 271, 275,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0, 272,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0, 273,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
], [
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0, 277,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0, 276,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,
    Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0,  Z0
]
];

/// The a7-a5-c5 triangle.
const TEST45: BitBoard = BitBoard(0x1_0307_0000_0000);

lazy_static! {
    // Ideally these would be compile time constants.
    static ref CONSTS: Consts = Consts::new();
}

struct Consts {
    mult_idx: [[u64; 10]; 5],
    mult_factor: [u64; 5],

    map_pawns: [u64; 64],
    lead_pawn_idx: [[u64; 64]; 6],
    lead_pawns_size: [[u64; 4]; 6],
}

impl Consts {
    fn new() -> Consts {
        let mut mult_idx = [[0; 10]; 5];
        let mut mult_factor = [0; 5];

        for i in 0..5 {
            let mut s = 0;
            for j in 0..10 {
                mult_idx[i][j] = s;
                s += if i == 0 { 1 } else { binomial(MULT_TWIST[INV_TRIANGLE[j]], i as u64) };
            }
            mult_factor[i] = s;
        }

        let mut available_squares = 48;

        let mut map_pawns = [0; 64];
        let mut lead_pawn_idx = [[0; 64]; 6];
        let mut lead_pawns_size = [[0; 4]; 6];

        // for lead_pawns_cnt in 1..=5 {
        //     for file in (0..4).map(File::new) {
        //         let mut idx = 0;
        //         for rank in (1..7).map(Rank::new) {
        //             let sq = Square::from_coords(file, rank);
        //             if lead_pawns_cnt == 1 {
        //                 available_squares -= 1;
        //                 map_pawns[usize::from(sq)] = available_squares;
        //                 available_squares -= 1;
        //                 map_pawns[usize::from(sq.flip_horizontal())] = available_squares;
        //             }
        //             lead_pawn_idx[lead_pawns_cnt][usize::from(sq)] = idx;
        //             idx += binomial(map_pawns[usize::from(sq)], lead_pawns_cnt as u64 - 1);
        //         }
        //         lead_pawns_size[lead_pawns_cnt][usize::from(file)] = idx;
        //     }
        // }

        Consts {
            mult_idx,
            mult_factor,
            map_pawns,
            lead_pawn_idx,
            lead_pawns_size,
        }
    }
}


