
pub use self::types::*;
use crate::syzygy::sz_errors::*;

use crate::{eprint_self, tables::*};
use crate::types::{
    BitBoard,Coord,Color::{self,*},Piece,Piece::*,Material,Game,
};
use crate::bitboard::DIAG_A1_H8;

use std::io;
use std::path::Path;
use std::marker::PhantomData;

use byteorder::{BE, LE, ByteOrder as _, ReadBytesExt as _};
use itertools::Itertools;
use num_integer::binomial;
use positioned_io::{RandomAccessFile, ReadAt, ReadBytesAtExt as _};
use arrayvec::ArrayVec;

use lazy_static::lazy_static;
use bitflags::bitflags;

mod types {
    use super::*;
    use std::fmt;
    use std::io;

    /// File extension and magic header bytes of Syzygy tables.
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct TableType {
        /// File extension, e.g. `rtbw`.
        pub ext: &'static str,
        /// Magic header bytes.
        pub magic: [u8; 4],
    }

    pub const TBW: TableType = TableType { ext: "rtbw", magic: [0x71, 0xe8, 0x23, 0x5d] };
    pub const TBZ: TableType = TableType { ext: "rtbz", magic: [0xd7, 0x66, 0x0c, 0xa5] };

    /// Syzygy tables are available for up to 7 pieces.
    pub const MAX_PIECES: usize = 7;

    /// List of up to `MAX_PIECES` pieces.
    pub type Pieces = ArrayVec<(Color,Piece), MAX_PIECES>;

    /// Metric stored in a table: WDL or DTZ.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum Metric {
        Wdl,
        Dtz,
    }

    impl fmt::Display for Metric {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                Metric::Wdl => f.write_str("wdl"),
                Metric::Dtz => f.write_str("dtz"),
            }
        }
    }

    /// WDL<sub>50</sub>. 5-valued evaluation of a position in the context of the
    /// 50-move drawing rule.
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[repr(i8)]
    pub enum Wdl {
        /// Unconditional loss for the side to move.
        Loss = -2,
        /// Loss that can be saved by the 50-move rule.
        BlessedLoss = -1,
        /// Unconditional draw.
        Draw = 0,
        /// Win that can be frustrated by the 50-move rule.
        CursedWin = 1,
        /// Unconditional win.
        Win = 2,
    }

    /// 4-valued evaluation of a decisive (not drawn) position in the context of
    /// the 50-move rule.
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub enum DecisiveWdl {
        /// Unconditional loss for the side to move.
        Loss = -2,
        /// Loss that can be saved by the 50-move rule.
        BlessedLoss = -1,
        /// Win that can be frustrated by the 50-move rule.
        CursedWin = 1,
        /// Unconditional win.
        Win = 2,
    }

    impl Wdl {

        // /// Converts `outcome` to a `Wdl` from the given point of view.
        // pub fn from_outcome(outcome: Outcome, pov: Color) -> Wdl {
        //     match outcome {
        //         Outcome::Draw => Wdl::Draw,
        //         Outcome::Decisive { winner } if winner == pov => Wdl::Win,
        //         _ => Wdl::Loss,
        //     }
        // }

        /// Converts `dtz` to a `Wdl`.
        ///
        /// Typically the result would be ambiguous for absolute DTZ values 100.
        /// This conversion assumes that such values were given immediately after
        /// a capture or pawn move, in which case the outcome is an unconditional
        /// win or loss.
        pub fn from_dtz_after_zeroing(dtz: Dtz) -> Wdl {
            match dtz.0 {
                n if n < -100 => Wdl::BlessedLoss,
                n if n < 0 => Wdl::Loss,
                0 => Wdl::Draw,
                n if n <= 100 => Wdl::Win,
                _ => Wdl::CursedWin,
            }
        }

        pub fn decisive(self) -> Option<DecisiveWdl> {
            Some(match self {
                Wdl::Loss => DecisiveWdl::Loss,
                Wdl::BlessedLoss => DecisiveWdl::BlessedLoss,
                Wdl::Draw => return None,
                Wdl::CursedWin => DecisiveWdl::CursedWin,
                Wdl::Win => DecisiveWdl::Win,
            })
        }
    }

    impl std::ops::Neg for Wdl {
        type Output = Wdl;

        fn neg(self) -> Wdl {
            match self {
                Wdl::Loss => Wdl::Win,
                Wdl::BlessedLoss => Wdl::CursedWin,
                Wdl::Draw => Wdl::Draw,
                Wdl::CursedWin => Wdl::BlessedLoss,
                Wdl::Win => Wdl::Loss,
            }
        }
    }

    impl From<DecisiveWdl> for Wdl {
        fn from(wdl: DecisiveWdl) -> Wdl {
            match wdl {
                DecisiveWdl::Loss => Wdl::Loss,
                DecisiveWdl::BlessedLoss => Wdl::BlessedLoss,
                DecisiveWdl::CursedWin => Wdl::CursedWin,
                DecisiveWdl::Win => Wdl::Win,
            }
        }
    }

    /// DTZ<sub>50</sub>′′ with rounding. Based on the distance to zeroing of the
    /// half-move clock.
    ///
    /// Zeroing the half-move clock while keeping the game theoretical result in
    /// hand guarantees making progress.
    ///
    /// Can be off by one due to
    /// [rounding](http://www.talkchess.com/forum3/viewtopic.php?f=7&t=58488#p651293):
    /// `Dtz(-n)` can mean a loss in `n + 1` plies and
    /// `Dtz(n)` can mean a win in `n + 1` plies.
    /// This implies some primary tablebase lines may waste up to 1 ply.
    /// Rounding is never used for endgame phases where it would change the game
    /// theoretical outcome.
    ///
    /// This means users need to be careful in positions that are nearly drawn
    /// under the 50-move rule! Carelessly wasting 1 more ply by not following the
    /// tablebase recommendation, for a total of 2 wasted plies, may change the
    /// outcome of the game.
    ///
    /// | DTZ | WDL | |
    /// | --- | --- | --- |
    /// | `-100 <= n <= -1` | Loss | Unconditional loss (assuming the 50-move counter is zero). Zeroing move can be forced in `-n` plies. |
    /// | `n < -100` | Blessed loss | Loss, but draw under the 50-move rule. A zeroing move can be forced in `-n` plies or `-n - 100` plies (if a later phase is responsible for the blessing). |
    /// | 0 | Draw | |
    /// | `100 < n` | Cursed win | Win, but draw under the 50-move rule. A zeroing move can be forced in `n` or `n - 100` plies (if a later phase is responsible for the curse). |
    /// | `1 <= n <= 100` | Win | Unconditional win (assuming the 50-move counter is zero). Zeroing move can be forced in `n` plies. |
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct Dtz(pub i32);

    impl Dtz {
        /// Converts `wdl` to a DTZ, given that the best move is zeroing.
        ///
        /// | WDL | DTZ |
        /// | --- | --- |
        /// | Loss | -1 |
        /// | Blessed loss | -101 |
        /// | Draw | 0 |
        /// | Cursed win | 101 |
        /// | Win | 1 |
        pub fn before_zeroing<T: Into<Wdl>>(wdl: T) -> Dtz {
            match wdl.into() {
                Wdl::Loss => Dtz(-1),
                Wdl::BlessedLoss => Dtz(-101),
                Wdl::Draw => Dtz(0),
                Wdl::CursedWin => Dtz(101),
                Wdl::Win => Dtz(1),
            }
        }

        /// Increases the absolute value by `plies`.
        pub fn add_plies(self, plies: i32) -> Dtz {
            let new_dtz = self.0.signum() * (self.0.abs() + plies);
            debug_assert!(self.0.signum() == new_dtz.signum());
            Dtz(new_dtz)
        }
    }

    impl std::ops::Neg for Dtz {
        type Output = Dtz;

        #[inline]
        fn neg(self) -> Dtz {
            Dtz(-self.0)
        }
    }

    impl std::ops::Add for Dtz {
        type Output = Dtz;

        #[inline]
        fn add(self, other: Dtz) -> Dtz {
            Dtz(self.0 + other.0)
        }
    }

    impl std::ops::AddAssign for Dtz {
        #[inline]
        fn add_assign(&mut self, other: Dtz) {
            self.0 += other.0;
        }
    }

    impl std::ops::Sub for Dtz {
        type Output = Dtz;

        #[inline]
        fn sub(self, other: Dtz) -> Dtz {
            Dtz(self.0 - other.0)
        }
    }

    impl std::ops::SubAssign for Dtz {
        #[inline]
        fn sub_assign(&mut self, other: Dtz) {
            self.0 -= other.0;
        }
    }

    impl std::ops::Neg for DecisiveWdl {
        type Output = DecisiveWdl;

        fn neg(self) -> DecisiveWdl {
            match self {
                DecisiveWdl::Loss => DecisiveWdl::Win,
                DecisiveWdl::BlessedLoss => DecisiveWdl::CursedWin,
                DecisiveWdl::CursedWin => DecisiveWdl::BlessedLoss,
                DecisiveWdl::Win => DecisiveWdl::Loss,
            }
        }
    }

}

pub trait TableTag {
    const METRIC: Metric;
}

#[derive(Debug)]
pub enum WdlTag {}

impl TableTag for WdlTag {
    const METRIC: Metric = Metric::Wdl;
}

#[derive(Debug)]
pub enum DtzTag {}

impl TableTag for DtzTag {
    const METRIC: Metric = Metric::Dtz;
}

bitflags! {
    /// Table layout flags.
    pub struct Layout: u8 {
        /// Two sided table for non-symmetrical material configuration.
        const SPLIT = 1;
        /// Table with pawns. Has subtables for each leading pawn file (a-d).
        const HAS_PAWNS = 2;
    }
}

bitflags! {
    /// Subtable format flags.
    pub struct Flag: u8 {
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

        for lead_pawns_cnt in 1..=5 {
            for file in 0..4 {
                let mut idx = 0;
                for rank in 1..7 {
                    let c0 = Coord::new(file,rank);
                    if lead_pawns_cnt == 1 {
                        available_squares -= 1;
                        map_pawns[usize::from(c0)] = available_squares;
                        available_squares -= 1;
                        let c1 = Coord::new(7 - c0.file(),c0.rank());
                        map_pawns[usize::from(c1)] = available_squares;
                    }
                    lead_pawn_idx[lead_pawns_cnt][usize::from(c0)] = idx;
                    idx += binomial(map_pawns[usize::from(c0)], lead_pawns_cnt as u64 - 1);
                }
                lead_pawns_size[lead_pawns_cnt][usize::from(file)] = idx;
            }
        }

        Consts {
            mult_idx,
            mult_factor,
            map_pawns,
            lead_pawn_idx,
            lead_pawns_size,
        }
    }
}

/// Read the magic header bytes that identify a tablebase file.
fn read_magic_header<F: ReadAt>(raf: &F) -> ProbeResult<[u8; 4]> {
    let mut buf = [0; 4];
    if let Err(error) = raf.read_exact_at(0, &mut buf) {
        match error.kind() {
            io::ErrorKind::UnexpectedEof => Err(ProbeError::Magic { magic: buf }),
            _ => Err(ProbeError::Read { error }),
        }
    } else {
        Ok(buf)
    }
}

/// Read 3 byte Huffman tree node.
fn read_lr<F: ReadAt>(raf: &F, ptr: u64) -> io::Result<(u16, u16)> {
    let mut buf = [0; 3];
    raf.read_exact_at(ptr, &mut buf)?;
    let left = (u16::from(buf[1] & 0xf) << 8) | u16::from(buf[0]);
    let right = (u16::from(buf[2]) << 4) | (u16::from(buf[1]) >> 4);
    Ok((left, right))
}

/// Header nibble to piece.
fn nibble_to_piece(p: u8) -> Option<(Color,Piece)> {
    // let color = Color::from_white(p & 8 == 0);
    let side = if p & 8 == 0 { White } else { Black };
    match p & !8 {
        1 => Some((side,Pawn)),
        2 => Some((side,Knight)),
        3 => Some((side,Bishop)),
        4 => Some((side,Rook)),
        5 => Some((side,Queen)),
        6 => Some((side,King)),
        _ => None,
    }
}

/// Checks if a square is on the a1-h8 diagonal.
fn offdiag(c0: Coord) -> bool {
    let out = DIAG_A1_H8.is_zero_at(c0);
    // eprintln!("offdiag: {:?} = {:?}", c0, out);
    out
}

/// Parse a piece list.
fn parse_pieces<F: ReadAt>(raf: &F, ptr: u64, count: usize, side: Color) -> ProbeResult<Pieces> {
    let mut buffer = [0; MAX_PIECES];
    let bytes = &mut buffer[..count];
    raf.read_exact_at(ptr, bytes).unwrap();
    let mut pieces = Pieces::new();
    for p in bytes {
        let k0 = *p & 0xf;
        let k1 = *p >> 4;
        // XXX: ??
        let p1 = if side == White { k0 } else { k1 };
        pieces.push(nibble_to_piece(p1).unwrap());
    }
    Ok(pieces)
}

/// Group pieces that will be encoded together.
fn group_pieces(pieces: &Pieces) -> ArrayVec<usize, MAX_PIECES> {
    let mut result = ArrayVec::new();

    let material = Material::from_iter(pieces.clone());

    // For pawnless positions: If there are at least 3 unique pieces then 3
    // unique pieces wil form the leading group. Otherwise the two kings will
    // form the leading group.
    let first_len = if material.has_piece(Pawn) {
        0
    } else if material.unique_pieces() >= 3 {
        3
    } else if material.unique_pieces() == 2 {
        2
    } else {
        usize::from(material.min_like_man())
    };

    if first_len > 0 {
        result.push(first_len);
    }

    // The remaining identical pieces are grouped together.
    result.extend(pieces.iter()
                  .skip(first_len)
                  .group_by(|p| *p)
                  .into_iter().map(|(_, g)| g.count()));

    result
}

/// Description of the encoding used for a piece configuration.
#[derive(Debug, Clone)]
pub struct GroupData {
    pieces:   Pieces,
    lens:     ArrayVec<usize, MAX_PIECES>,
    factors:  ArrayVec<u64, { MAX_PIECES + 1 }>,
}

impl GroupData {
    // pub fn new<S: Syzygy>(pieces: Pieces, order: [u8; 2], file: usize) -> ProbeResult<GroupData> {
    pub fn new(pieces: Pieces, order: [u8; 2], file: usize) -> ProbeResult<GroupData> {
        assert!(pieces.len() >= 2);

        let material = Material::from_iter(pieces.clone());

        // Compute group lengths.
        let lens = group_pieces(&pieces);

        // Compute a factor for each group.
        // let pp = material.white.has_pawns() && material.black.has_pawns();
        let pp = material.has_piece_side(Pawn, White) && material.has_piece_side(Pawn, Black);

        let mut factors = ArrayVec::from([0; MAX_PIECES + 1]);
        factors.truncate(lens.len() + 1);
        let mut free_squares = 64 - lens[0] - if pp { lens[1] } else { 0 };
        let mut next = if pp { 2 } else { 1 };
        let mut idx = 1;
        let mut k = 0;

        while next < lens.len() || k == order[0] || k == order[1] {
            if k == order[0] {
                // Leading pawns or pieces.
                factors[0] = idx;

                if material.has_piece(Pawn) {
                    idx *= CONSTS.lead_pawns_size[lens[0]][file];
                } else if material.unique_pieces() >= 3 {
                    idx *= 31_332;
                } else if material.unique_pieces() == 2 {
                    // idx *= if S::CONNECTED_KINGS { 518 } else { 462 };
                    // unimplemented!()
                    idx *= 462;
                } else if material.min_like_man() == 2 {
                    idx *= 278;
                } else {
                    idx *= CONSTS.mult_factor[usize::from(material.min_like_man()) - 1];
                }
            } else if k == order[1] {
                // Remaining pawns.
                factors[1] = idx;
                idx *= binomial(48 - lens[0], lens[1]) as u64;
            } else {
                // Remaining pieces.
                factors[next] = idx;
                idx *= binomial(free_squares, lens[next]) as u64;
                free_squares -= lens[next];
                next += 1;
            }
            k += 1;
        }

        factors[lens.len()] = idx;

        Ok(GroupData {
            pieces,
            lens,
            factors,
        })
    }
}

/// Indexes into table of remapped DTZ values.
#[derive(Debug)]
pub enum DtzMap {
    /// Normal 8-bit DTZ map.
    Normal {
        map_ptr: u64,
        by_wdl: [u16; 4]
    },
    /// Wide 16-bit DTZ map for very long endgames.
    Wide {
        map_ptr: u64,
        by_wdl: [u16; 4]
    },
}

impl DtzMap {
    fn read<F: ReadAt>(&self, raf: &F, wdl: DecisiveWdl, res: u16) -> ProbeResult<u16> {
        let wdl = match wdl {
            DecisiveWdl::Win => 0,
            DecisiveWdl::Loss => 1,
            DecisiveWdl::CursedWin => 2,
            DecisiveWdl::BlessedLoss => 3,
        };

        Ok(match *self {
            DtzMap::Normal { map_ptr, by_wdl } => {
                let offset = map_ptr + u64::from(by_wdl[wdl]) + u64::from(res);
                u16::from(raf.read_u8_at(offset)?)
            }
            DtzMap::Wide { map_ptr, by_wdl } => {
                let offset = map_ptr + 2 * (u64::from(by_wdl[wdl]) + u64::from(res));
                raf.read_u16_at::<LE>(offset)?
            }
        })
    }
}

impl PairsData {
    pub fn print_debug(&self) {

        eprintln!("PairsData Debug: ");

        let gd = &self.groups;
        for (side,pc) in gd.pieces.iter() {
            eprintln!("(side,pc) = {:?}", (side,pc));
        }
        eprint_self!(gd.lens);
        eprint_self!(gd.factors);

        // eprint_self!(self.flags);
        // eprint_self!(self.groups);
        // eprint_self!(self.block_size);
        // eprint_self!(self.span);
        // eprint_self!(self.blocks_num);
        // eprint_self!(self.btree);
        // eprint_self!(self.min_symlen);
        // eprint_self!(self.lowest_sym);
        // eprint_self!(self.base);
        // eprint_self!(self.symlen);
        // eprint_self!(self.sparse_index);
        // eprint_self!(self.sparse_index_size);
        // eprint_self!(self.block_lengths);
        // eprint_self!(self.block_length_size);
        // eprint_self!(self.data);
        // eprint_self!(self.dtz_map);

        eprintln!();

    }
}

/// Description of encoding and compression.
#[derive(Debug)]
pub struct PairsData {
    /// Encoding flags.
    pub flags: Flag,
    /// Piece configuration encoding info.
    pub groups: GroupData,

    /// Block size in bytes.
    pub block_size: u32,
    /// About every span values there is a sparse index entry.
    pub span: u32,
    /// Number of blocks in the table.
    pub blocks_num: u32,

    /// Offset of the symbol table.
    pub btree: u64,
    /// Minimum length in bits of the Huffman symbols.
    pub min_symlen: u8,
    /// Offset of the lowest symbols for each length.
    pub lowest_sym: u64,
    /// 64-bit padded lowest symbols for each length.
    pub base: Vec<u64>,
    /// Number of values represented by a given Huffman symbol.
    pub symlen: Vec<u8>,

    /// Offset of the sparse index.
    pub sparse_index: u64,
    /// Size of the sparse index.
    pub sparse_index_size: u32,

    /// Offset of the block length table.
    pub block_lengths: u64,
    /// Size of the block length table, padded to be bigger than `blocks_num`.
    pub block_length_size: u32,

    /// Start of compressed data.
    pub data: u64,

    /// DTZ mapping.
    pub dtz_map: Option<DtzMap>,
}

impl PairsData {
    // pub fn parse<S: Syzygy, T: TableTag, F: ReadAt>(
    //     raf: &F, mut ptr: u64, groups: GroupData) -> ProbeResult<(PairsData, u64)> {
    pub fn parse<T: TableTag, F: ReadAt>(
        raf:        &F,
        mut ptr:    u64,
        groups:     GroupData,
    ) -> ProbeResult<(PairsData, u64)> {
        let flags = Flag::from_bits_truncate(raf.read_u8_at(ptr)?);

        if flags.contains(Flag::SINGLE_VALUE) {
            let single_value = if T::METRIC == Metric::Wdl {
                raf.read_u8_at(ptr + 1)?
            // } else if S::CAPTURES_COMPULSORY {
            //     1 // http://www.talkchess.com/forum/viewtopic.php?p=698093#698093
            } else {
                0
            };

            return Ok((PairsData {
                flags,
                min_symlen: single_value,
                groups,
                base: Vec::new(),
                block_lengths: 0,
                block_length_size: 0,
                block_size: 0,
                blocks_num: 0,
                btree: 0,
                data: 0,
                lowest_sym: 0,
                span: 0,
                sparse_index: 0,
                sparse_index_size: 0,
                symlen: Vec::new(),
                dtz_map: None,
            }, ptr + 2));
        }

        // Read header.
        let mut header = [0; 10];
        raf.read_exact_at(ptr, &mut header)?;

        let tb_size = groups.factors[groups.lens.len()];
        let block_size = u!(1u32.checked_shl(u32::from(header[1])));
        ensure!(block_size <= MAX_BLOCK_SIZE as u32);
        let span = u!(1u32.checked_shl(u32::from(header[2])));
        let sparse_index_size = ((tb_size + u64::from(span) - 1) / u64::from(span)) as u32;
        let padding = header[3];
        let blocks_num = LE::read_u32(&header[4..]);
        let block_length_size = u!(blocks_num.checked_add(u32::from(padding)));

        let max_symlen = header[8];
        ensure!(max_symlen <= 32);
        let min_symlen = header[9];
        ensure!(min_symlen <= 32);

        ensure!(max_symlen >= min_symlen);
        let h = usize::from(max_symlen - min_symlen + 1);

        let lowest_sym = ptr + 10;

        // Initialize base.
        let mut base = vec![0u64; h];
        for i in (0..h - 1).rev() {
            let ptr = lowest_sym + i as u64 * 2;

            base[i] = u!(u!(base[i + 1]
                .checked_add(u64::from(raf.read_u16_at::<LE>(ptr)?)))
                .checked_sub(u64::from(raf.read_u16_at::<LE>(ptr + 2)?))) / 2;

            ensure!(base[i] * 2 >= base[i + 1]);
        }

        for (i, base) in base.iter_mut().enumerate() {
            *base = u!(base.checked_shl(64 - (u32::from(min_symlen) + i as u32)));
        }

        // Initialize symlen.
        ptr += 10 + h as u64 * 2;
        let sym = raf.read_u16_at::<LE>(ptr)?;
        ptr += 2;
        let btree = ptr;
        let mut symlen = vec![0; usize::from(sym)];
        let mut visited = vec![false; symlen.len()];
        for s in 0..sym {
           read_symlen(raf, btree, &mut symlen, &mut visited, s, 16)?;
        }
        ptr += symlen.len() as u64 * 3 + (symlen.len() as u64 & 1);

        // Result.
        Ok((PairsData {
            flags,
            groups,

            block_size,
            span,
            blocks_num,

            btree,
            min_symlen,
            lowest_sym,
            base,
            symlen,

            sparse_index: 0, // to be initialized later
            sparse_index_size,

            block_lengths: 0, // to be initialized later
            block_length_size,

            data: 0, // to be initialized later

            dtz_map: None, // to be initialized later
        }, ptr))
    }
}

/// Build the symlen table.
fn read_symlen<F: ReadAt>(
    raf: &F, btree: u64, symlen: &mut Vec<u8>, visited: &mut [bool], sym: u16, depth: u8
) -> ProbeResult<()> {
    if *u!(visited.get(usize::from(sym))) {
        return Ok(());
    }

    let ptr = btree + 3 * u64::from(sym);
    let (left, right) = read_lr(&raf, ptr)?;

    if right == 0xfff {
        symlen[usize::from(sym)] = 0;
    } else {
        // Guard against stack overflow.
        let depth = u!(depth.checked_sub(1));

        read_symlen(raf, btree, symlen, visited, left, depth)?;
        read_symlen(raf, btree, symlen, visited, right, depth)?;

        symlen[usize::from(sym)] = u!(u!(symlen[usize::from(left)]
                                         .checked_add(symlen[usize::from(right)]))
                                      .checked_add(1))
    }

    visited[usize::from(sym)] = true;
    Ok(())
}

/// Descripton of encoding and compression for both sides of a table.
#[derive(Debug)]
pub struct FileData {
    pub sides: ArrayVec<PairsData, 2>,
}

/// A Syzygy table.
#[derive(Debug)]
// struct Table<T: TableTag, P: Position + Syzygy, F: ReadAt> {
pub struct Table<T: TableTag, F: ReadAt> {
    pub is_wdl: PhantomData<T>,
    // syzygy: PhantomData<P>,

    pub raf:                   F,
    pub num_unique_pieces:     u8,
    pub min_like_man:          u8,
    pub files:                 ArrayVec<FileData, 4>,
}

// impl<T: TableTag, S: Position + Syzygy, F: ReadAt> Table<T, S, F> {
impl<T: TableTag, F: ReadAt + std::fmt::Debug> Table<T, F> {

    /// Open a table, parse the header, the headers of the subtables and
    /// prepare meta data required for decompression.
    ///
    /// # Panics
    ///
    /// Panics if the `material` configuration is not supported by Syzygy
    /// tablebases (more than 7 pieces or side without pieces).
    pub fn new(raf: F, material: &Material) -> ProbeResult<Table<T, F>> {
        let material = material.clone();
        assert!(material.count() as usize <= MAX_PIECES);
        assert!(material.count_side(White) >= 1);
        assert!(material.count_side(Black) >= 1);

        // Check magic.
        let magic = match T::METRIC {
            Metric::Wdl => TBW.magic,
            Metric::Dtz => TBZ.magic,
        };

        let magic_header = read_magic_header(&raf)?;
        if magic != magic_header && material.has_pawns()
        {
            return Err(ProbeError::Magic { magic: magic_header });
        }

        // Read layout flags.
        let layout = Layout::from_bits_truncate(raf.read_u8_at(4)?);
        let has_pawns = layout.contains(Layout::HAS_PAWNS);
        let split = layout.contains(Layout::SPLIT);

        // Check consistency of layout and material key.
        ensure!(has_pawns == material.has_pawns());
        ensure!(split != material.is_symmetric());

        // Read group data.
        let pp = material.has_piece_side(Pawn,White) && material.has_piece_side(Pawn,Black);
        let num_files = if has_pawns { 4 } else { 1 };
        let num_sides = if T::METRIC == Metric::Wdl && !material.is_symmetric() { 2 } else { 1 };

        let mut ptr = 5;

        // eprintln!("pp = {:?}", pp);
        // eprintln!("num_files = {:?}", num_files);
        // eprintln!("num_sides = {:?}", num_sides);

        let files = (0..num_files).map(|file| {
            let order = [
                [raf.read_u8_at(ptr)? & 0xf, if pp { raf.read_u8_at(ptr + 1)? & 0xf } else { 0xf }],
                [raf.read_u8_at(ptr)? >> 4, if pp { raf.read_u8_at(ptr + 1)? >> 4 } else { 0xf }],
            ];

            ptr += 1 + if pp { 1 } else { 0 };

            let sides = [Color::White, Color::Black].iter().take(num_sides).map(|side| {
                let pieces = parse_pieces(&raf, ptr, material.count() as usize, *side)?;
                let key = Material::from_iter(pieces.clone());

                // if !(key == material || key.into_flipped() == material) {
                //     panic!("nop 0");
                // }

                ensure!(key == material || key.into_flipped() == material);
                // println!("wat -2c");
                let gd = GroupData::new(pieces, order[side.fold(0, 1)], file);
                // eprintln!("gd = {:?}", gd);
                gd
                // unimplemented!()
            }).collect::<ProbeResult<ArrayVec<_, 4>>>()?;
            // println!("wat -3");

            // let mut sides: ArrayVec<GroupData, 2> = ArrayVec::default();
            // for side in [White,Black].iter().take(num_sides) {
            //     println!("wat -1");
            //     let pieces = parse_pieces(&raf, ptr, material.count() as usize, *side)?;
            //     for (c,p) in pieces.iter() {
            //         eprintln!("{:?}", (c,p));
            //     }
            //     let key = Material::from_iter(pieces.clone());
            // }

            ptr += material.count() as u64;

            Ok(sides)
        }).collect::<ProbeResult<ArrayVec<_, 4>>>()?;
        // println!("wat 3");

        ptr += ptr & 1;

        // Ensure reference pawn goes first.
        ensure!((files[0][0].pieces[0].1 == Pawn) == has_pawns);

        // Ensure material is consistent with first file.
        for file in files.iter() {
            for side in file.iter() {
                let key = Material::from_iter(side.pieces.clone());
                ensure!(key == Material::from_iter(files[0][0].pieces.clone()));
            }
        }

        // Setup pairs.
        let mut files = files.into_iter().map(|file| {
            let sides = file.into_iter().map(|side| {
                let (mut pairs, next_ptr) = PairsData::parse::<T, _>(&raf, ptr, side)?;

                // if T::METRIC == Metric::Dtz && S::CAPTURES_COMPULSORY
                //     && pairs.flags.contains(Flag::SINGLE_VALUE) {
                //     pairs.min_symlen = 1;
                // }

                ptr = next_ptr;

                Ok(pairs)
            }).collect::<ProbeResult<ArrayVec<_, 2>>>()?;

            Ok(FileData { sides })
        }).collect::<ProbeResult<ArrayVec<_, 4>>>()?;

        // Setup DTZ map.
        if T::METRIC == Metric::Dtz {
            let map_ptr = ptr;

            for file in files.iter_mut() {
                if file.sides[0].flags.contains(Flag::MAPPED) {
                    let mut by_wdl = [0; 4];
                    if file.sides[0].flags.contains(Flag::WIDE_DTZ) {
                        for idx in &mut by_wdl {
                            *idx = ((ptr - map_ptr + 2) / 2) as u16;
                            ptr += u64::from(raf.read_u16_at::<LE>(ptr)?) * 2 + 2;
                        }
                        file.sides[0].dtz_map = Some(DtzMap::Wide { map_ptr, by_wdl });
                    } else {
                        for idx in &mut by_wdl {
                            *idx = (ptr - map_ptr + 1) as u16;
                            ptr += u64::from(raf.read_u8_at(ptr)?) + 1;
                        }
                        file.sides[0].dtz_map = Some(DtzMap::Normal { map_ptr, by_wdl });
                    }
                }
            }

            ptr += ptr & 1;
        }

        // Setup sparse index.
        for file in files.iter_mut() {
            for side in file.sides.iter_mut() {
                side.sparse_index = ptr;
                ptr += u64::from(side.sparse_index_size) * 6;
            }
        }

        for file in files.iter_mut() {
            for side in file.sides.iter_mut() {
                side.block_lengths = ptr;
                ptr += u64::from(side.block_length_size) * 2;
            }
        }

        for file in files.iter_mut() {
            for side in file.sides.iter_mut() {
                ptr = (ptr + 0x3f) & !0x3f; // 64 byte alignment
                side.data = ptr;
                ptr = u!(ptr.checked_add(u64::from(side.blocks_num) * u64::from(side.block_size)));
            }
        }

        // Result.
        Ok(Table {
            is_wdl: PhantomData,
            // syzygy: PhantomData,
            raf,
            num_unique_pieces: material.unique_pieces(),
            min_like_man: material.min_like_man(),
            files,
        })
    }

    /// Retrieves the value stored for `idx` by decompressing Huffman coded
    /// symbols stored in the corresponding block of the table.
    fn decompress_pairs(&self, g: &Game, d: &PairsData, idx: u64) -> ProbeResult<u16> {
        // Special case: The table stores only a single value.
        if d.flags.contains(Flag::SINGLE_VALUE) {
            return Ok(u16::from(d.min_symlen));
        }

        // println!("wat 0");
        // Use the sparse index to jump very close to the correct block.
        let main_idx = idx / u64::from(d.span);
        ensure!(main_idx <= u64::from(u32::max_value()));

        // eprintln!("idx = {:?}", idx);
        // eprintln!("main_idx = {:?}", main_idx);
        // eprintln!("d.spars_index = {:?}", d.sparse_index);

        // println!("wat 1");
        let mut block = self.raf.read_u32_at::<LE>(d.sparse_index + 6 * main_idx)?;
        // println!("wat 2");
        let offset = i64::from(self.raf.read_u16_at::<LE>(d.sparse_index + 6 * main_idx + 4)?);
        // println!("wat 3");

        let mut lit_idx = idx as i64 % i64::from(d.span) - i64::from(d.span) / 2;
        lit_idx += offset;

        // Now move forwards/backwards to find the correct block.
        while lit_idx < 0 {
            block = u!(block.checked_sub(1));
            lit_idx += i64::from(self.raf.read_u16_at::<LE>(d.block_lengths + u64::from(block) * 2)?) + 1;
        }
        // println!("wat 4");
        loop {
            let block_length = i64::from(self.raf.read_u16_at::<LE>(d.block_lengths + u64::from(block) * 2)?) + 1;
            if lit_idx >= block_length {
                lit_idx -= block_length;
                block = u!(block.checked_add(1));
            } else {
                break;
            }
        }
        // println!("wat 5");

        // Read block (and 4 bytes to prevent out of bounds read) into memory.
        let mut block_buffer = [0; MAX_BLOCK_SIZE + 4];
        let block_buffer = &mut block_buffer[..(d.block_size as usize + 4)];

        // eprintln!("block = {:?}", block);
        // eprintln!("d.block_size = {:?}", d.block_size);

        self.raf.read_exact_at(u!(d.data.checked_add(u64::from(block) * u64::from(d.block_size))), block_buffer)?;

        // eprintln!("block_buffer = {:?}", block_buffer);

        // println!("wat 6");

        let mut cursor = io::Cursor::new(block_buffer);

        // Find sym, the Huffman symbol that encodes the value for idx.
        let mut buf = cursor.read_u64::<BE>()?;
        let mut buf_size = 64;
        // println!("wat 7");

        let mut sym;

        // let mut k = 0;
        loop {
            // eprintln!("wot k = {}", k);
            // k += 1;

            let mut len = 0;

            while buf < *u!(d.base.get(len)) {
                len += 1;
            }

            sym = ((buf - d.base[len]) >> (64 - len - usize::from(d.min_symlen))) as u16;
            // println!("wat 8");
            sym += self.raf.read_u16_at::<LE>(d.lowest_sym + 2 * len as u64)?;
            // println!("wat 9");

            if lit_idx < i64::from(*u!(d.symlen.get(usize::from(sym)))) + 1 {
                break;
            }

            lit_idx -= i64::from(*u!(d.symlen.get(usize::from(sym)))) + 1;
            len += usize::from(d.min_symlen);
            buf <<= len;
            buf_size -= len;

            // Refill the buffer.
            if buf_size <= 32 {
                buf_size += 32;

                // buf |= u64::from(cursor.read_u32::<BE>()?) << (64 - buf_size);

                let c = match cursor.read_u32::<BE>() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("nope 0: {:?}\n{:?}", e, g);
                        panic!()
                    },
                };

                buf |= u64::from(c) << (64 - buf_size);
            }
        }

        // println!("wat 10");
        // Decompress Huffman symbol.
        while *u!(d.symlen.get(usize::from(sym))) != 0 {
            let (left, right) = read_lr(&self.raf, d.btree + 3 * u64::from(sym))?;

            if lit_idx < i64::from(*u!(d.symlen.get(usize::from(left)))) + 1 {
                sym = left;
            } else {
                lit_idx -= i64::from(*u!(d.symlen.get(usize::from(left)))) + 1;
                sym = right;
            }
        }
        // println!("wat 11");

        let w = d.btree + 3 * u64::from(sym);
        match T::METRIC {
            Metric::Wdl => {
                let c = match self.raf.read_u8_at(w) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("nope 1: {:?}\n{:?}", e, g);
                        panic!()
                    }
                };
                Ok(u16::from(c))
            },
            Metric::Dtz => {
                let c = match self.raf.read_u16_at::<LE>(w) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("nope 2: {:?}\n{:?}", e, g);
                        panic!()
                    }
                };
                Ok(c & 0xfff)
            },
        }
    }

    /// Given a position, determine the unique (modulo symmetries) index into
    /// the corresponding subtable.
    fn encode(&self, g: &Game) -> ProbeResult<Option<(&PairsData, u64)>> {
        let key = g.state.material;
        let material = Material::from_iter(self.files[0].sides[0].groups.pieces.clone());
        assert!(key == material || key == material.clone().into_flipped());

        // eprintln!("key = {:?}", key);
        // eprintln!("material = {:?}", material);

        let symmetric_btm = material.is_symmetric() && g.state.side_to_move == Black;
        let black_stronger = key != material;
        let flip = symmetric_btm || black_stronger;
        // let bside = pos.turn().is_black() ^ flip;
        let bside = (g.state.side_to_move == Black) ^ flip;

        let mut squares: ArrayVec<Coord, MAX_PIECES> = ArrayVec::new();
        let mut used = BitBoard(0);

        // For pawns there are subtables for each file (a, b, c, d) the
        // leading pawn can be placed on.
        let file = &self.files[if material.has_pawns() {
            let reference_pawn = self.files[0].sides[0].groups.pieces[0];
            assert_eq!(reference_pawn.1, Pawn);
            let color = reference_pawn.0 ^ flip;

            // let lead_pawns = pos.board().pawns() & pos.board().by_color(color);
            let lead_pawns = g.get(Pawn, color);
            used.extend_mut(lead_pawns);
            squares.extend(lead_pawns.into_iter()
                           .map(|sq| if flip { Coord::from(sq).flip_vertical() } else { Coord::from(sq) }));

            // Ensure squares[0] is the maximum with regard to map_pawns.
            for i in 1..squares.len() {
                if CONSTS.map_pawns[usize::from(squares[0])] < CONSTS.map_pawns[usize::from(squares[i])] {
                    squares.swap(0, i);
                }
            }
            if squares[0].file() >= 4 {
                squares[0].flip_horizontal().file() as usize
            } else {
                squares[0].file() as usize
            }
        } else {
            0
        }];

        // WDL tables have subtables for each side to move.
        let side = &file.sides[if bside { file.sides.len() - 1 } else { 0 }];

        // DTZ tables store only one side to move. It is possible that we have
        // to check the other side (by doing a 1-ply search).
        if T::METRIC == Metric::Dtz
            && side.flags.contains(Flag::STM) != bside && (!material.is_symmetric() || material.has_pawns()) {
            return Ok(None);
        }

        // The subtable has been determined.
        //
        // So far squares has been initialized with the leading pawns.
        // Also add the other pieces.
        let lead_pawns_count = squares.len();

        for piece in side.groups.pieces.iter().skip(lead_pawns_count) {
            let color = piece.0 ^ flip;

            // let square = u!((pos.board().by_piece(piece.role.of(color)) & !used).first());
            let square = u!((g.get(piece.1, color) & !used).bitscan_checked());

            // squares.push(if flip { square.flip_vertical() } else { square });
            squares.push(if flip { Coord::from(square).flip_vertical() } else { Coord::from(square) });
            used.set_one_mut(Coord::from(square));
        }

        assert!(squares.len() >= 2);

        // Now we can compute the index according to the piece positions.
        if squares[0].file() >= 4 {
            for square in &mut squares {
                *square = square.flip_horizontal();
            }
        }

        // eprintln!("squares = {:?}", squares);

        let mut idx = if material.has_pawns() {
            // panic!("nope");
            let mut idx = CONSTS.lead_pawn_idx[lead_pawns_count][usize::from(squares[0])];

            // eprintln!("idx = {:?}", idx);

            squares[1..lead_pawns_count].sort_unstable_by_key(|sq| CONSTS.map_pawns[usize::from(*sq)]);

            // eprintln!("lead_pawns_count = {:?}", lead_pawns_count);

            for (i, &square) in squares.iter().enumerate().take(lead_pawns_count).skip(1) {
                // eprintln!("(i,square) = {:?}", (i,square));
                idx += binomial(CONSTS.map_pawns[usize::from(square)], i as u64);
            }

            // eprintln!("idx = {:?}", idx);

            idx
        } else {
            if squares[0].rank() >= 4 {
                // println!("wat 1");
                for square in &mut squares {
                    *square = square.flip_vertical();
                }
            }

            for i in 0..side.groups.lens[0] {
                // println!("wat 2");
                // if squares[i].file().flip_diagonal() == squares[i].rank() {
                if squares[i].flip_diagonal().rank() == squares[i].rank() {
                    // println!("wat 2 a");
                    continue;
                }

                // if squares[i].rank().flip_diagonal() > squares[i].file() {
                if squares[i].flip_diagonal().file() > squares[i].file() {
                    // println!("wat 2 b");
                    for square in &mut squares[i..] {
                        // println!("wat 2 c");
                        *square = square.flip_diagonal();
                    }
                }

                break;
            }
            // eprintln!("squares = {:?}", squares);

            // eprint_self!(self.num_unique_pieces);

            if self.num_unique_pieces > 2 {
                // println!("branch -1");
                let adjust1 = if squares[1] > squares[0] { 1 } else { 0 };
                let adjust2 = if squares[2] > squares[0] { 1 } else { 0 } +
                              if squares[2] > squares[1] { 1 } else { 0 };

                if offdiag(squares[0]) {
                    // println!("branch 0");
                    TRIANGLE[usize::from(squares[0])] * 63 * 62 +
                    (sq_to_u64(squares[1]) - adjust1) * 62 +
                    (sq_to_u64(squares[2]) - adjust2)
                } else if offdiag(squares[1]) {
                    // println!("branch 1");
                    6 * 63 * 62 +
                    squares[0].rank() as u64 * 28 * 62 +
                    LOWER[usize::from(squares[1])] * 62 +
                    sq_to_u64(squares[2]) - adjust2
                } else if offdiag(squares[2]) {
                    // println!("branch 2");
                    6 * 63 * 62 + 4 * 28 * 62 +
                    squares[0].rank() as u64 * 7 * 28 +
                    (squares[1].rank() as u64 - adjust1) * 28 +
                    LOWER[usize::from(squares[2])]
                } else {
                    // println!("branch 3");
                    6 * 63 * 62 + 4 * 28 * 62 + 4 * 7 * 28 +
                    squares[0].rank() as u64 * 7 * 6 +
                    (squares[1].rank() as u64 - adjust1) * 6 +
                    (squares[2].rank() as u64 - adjust2)
                }
            } else if self.num_unique_pieces == 2 {
                KK_IDX[TRIANGLE[usize::from(squares[0])] as usize][usize::from(squares[1])]
            } else if self.min_like_man == 2 {
                if TRIANGLE[usize::from(squares[0])] > TRIANGLE[usize::from(squares[1])] {
                    squares.swap(0, 1);
                }

                if squares[0].file() >= 4 {
                    for square in &mut squares {
                        *square = square.flip_horizontal();
                    }
                }

                if squares[0].rank() >= 4 {
                    for square in &mut squares {
                        *square = square.flip_vertical();
                    }
                }

                // if squares[0].rank().flip_diagonal() > squares[0].file() ||
                if squares[0].flip_diagonal().file() > squares[0].file() ||
                   // (!offdiag(squares[0]) && squares[1].rank().flip_diagonal() > squares[1].file()) {
                   (!offdiag(squares[0]) && squares[1].flip_diagonal().file() > squares[1].file()) {
                    for square in &mut squares {
                        *square = square.flip_diagonal();
                    }
                }

                if TEST45.is_one_at(squares[1])
                    && TRIANGLE[usize::from(squares[0])] == TRIANGLE[usize::from(squares[1])] {
                    squares.swap(0, 1);

                    for square in &mut squares {
                        *square = square.flip_vertical().flip_diagonal();
                    }
                }

                PP_IDX[TRIANGLE[usize::from(squares[0])] as usize][usize::from(squares[1])]
            } else {
                for i in 1..side.groups.lens[0] {
                    if TRIANGLE[usize::from(squares[0])] > TRIANGLE[usize::from(squares[i])] {
                        squares.swap(0, i);
                    }
                }

                if squares[0].file() >= 4 {
                    for square in &mut squares {
                        *square = square.flip_horizontal();
                    }
                }

                if squares[0].rank() >= 4 {
                    for square in &mut squares {
                        *square = square.flip_vertical();
                    }
                }

                // if squares[0].rank().flip_diagonal() > squares[0].file() {
                if squares[0].flip_diagonal().file() > squares[0].file() {
                    for square in &mut squares {
                        *square = square.flip_diagonal();
                    }
                }

                for i in 1..side.groups.lens[0] {
                    for j in (i + 1)..side.groups.lens[0] {
                        if MULT_TWIST[usize::from(squares[i])] > MULT_TWIST[usize::from(squares[j])] {
                            squares.swap(i, j);
                        }
                    }
                }

                let mut idx = CONSTS.mult_idx[side.groups.lens[0] - 1][TRIANGLE[usize::from(squares[0])] as usize];
                // eprintln!("idx -1 = {:?}", idx);
                for i in 1..side.groups.lens[0] {
                    idx += binomial(MULT_TWIST[usize::from(squares[i])], i as u64);
                }

                idx
            }
        };

        // eprintln!("idx 0 = {:?}", idx);

        idx *= side.groups.factors[0];

        // eprintln!("idx 0 = {:?}", idx);

        // Encode remaining pawns.
        // let mut remaining_pawns = material.white.has_pawns() && material.black.has_pawns();
        let mut remaining_pawns = material.has_piece_side(Pawn, White) && material.has_piece_side(Pawn, Black);
        let mut next = 1;
        let mut group_sq = side.groups.lens[0];
        for lens in side.groups.lens.iter().cloned().skip(1) {
            // eprintln!("\nlens = {:?}", lens);

            let (prev_squares, group_squares) = squares.split_at_mut(group_sq);
            let group_squares = &mut group_squares[..lens];
            // group_squares.sort_unstable();
            group_squares.sort_unstable_by_key(|x| {
                x.flip_diagonal()
            });

            let mut n = 0;

            // eprintln!("group_squares = {:?}", group_squares);

            for (i, &group_square) in group_squares.iter().enumerate().take(lens) {
                // eprintln!("(i,group_square) = {:?}", (i,group_square));
                // eprintln!("prev_squares = {:?}", prev_squares);
                // let adjust = prev_squares[..group_sq].iter().filter(|sq| group_square > **sq).count() as u64;
                // let adjust = prev_squares[..group_sq].iter().filter(|sq| group_square <= **sq).count() as u64;
                let adjust = prev_squares[..group_sq].iter()
                    .filter(|sq| u8::from(group_square) > u8::from(**sq)).count() as u64;
                // eprintln!("adjust = {:?}", adjust);
                let kk = binomial(
                    sq_to_u64(group_square) - adjust - if remaining_pawns { 8 } else { 0 }, i as u64 + 1);
                // eprintln!("kk = {:?}", kk);
                n += kk;
                // eprintln!("n {} = {:?}", i, n);
            }
            // eprintln!("n = {:?}", n);
            // eprintln!("next = {:?}", next);
            // eprintln!("side.groups.factors[next] = {:?}", side.groups.factors[next]);

            remaining_pawns = false;
            idx += n * side.groups.factors[next];
            group_sq += side.groups.lens[next];
            next += 1;
            // eprintln!("idx 1 = {:?}", idx);
        }

        Ok(Some((side, idx)))
    }

    pub fn probe_wdl(&self, g: &Game) -> ProbeResult<Wdl> {
        assert_eq!(T::METRIC, Metric::Wdl);

        let (side, idx) = self.encode(g)?.expect("wdl tables are two sided");

        // eprintln!("(side,idx) = {:?}", (side,idx));
        // eprintln!("probe_wdl idx = {:?}", idx);

        // side.print_debug();

        // let decompressed = self.decompress_pairs(g, side, idx)?;
        let decompressed = match self.decompress_pairs(g, side, idx) {
            Ok(d) => d,
            Err(e) => {
                // let k: RandomAccessFile = self.raf;
                eprintln!("self.raf = {:?}", self.raf);
                eprintln!("nope probe_wdl = {:?}\n{:?}", e, g);
                // eprintln!("nope probe_wdl = {:?}", e);
                panic!();
            }
        };

        Ok(match decompressed {
            0 => Wdl::Loss,
            1 => Wdl::BlessedLoss,
            2 => Wdl::Draw,
            3 => Wdl::CursedWin,
            4 => Wdl::Win,
            _ => throw!(),
        })
    }

    pub fn probe_dtz(&self, g: &Game, wdl: DecisiveWdl) -> ProbeResult<Option<Dtz>> {
        assert_eq!(T::METRIC, Metric::Dtz);

        let (side, idx) = match self.encode(g)? {
            Some(found) => found,
            None        => return Ok(None), // check other side
        };

        let res = self.decompress_pairs(g, side, idx)?;

        let res = i32::from(match side.dtz_map {
            None          => res,
            Some(ref map) => map.read(&self.raf, wdl, res)?,
        });

        let stores_plies = match wdl {
            DecisiveWdl::Win  => side.flags.contains(Flag::WIN_PLIES),
            DecisiveWdl::Loss => side.flags.contains(Flag::LOSS_PLIES),
            DecisiveWdl::CursedWin | DecisiveWdl::BlessedLoss => false,
        };

        Ok(Some(Dtz(if stores_plies { res } else { 2 * res })))
    }

}

fn sq_to_u64(c0: Coord) -> u64 {
    let k: u8 = c0.into();
    k as u64
}

pub fn open_table_file<P: AsRef<Path>>(path: P) -> ProbeResult<RandomAccessFile> {
    let file = std::fs::File::open(path)?;
    ensure!(file.metadata()?.len() % 64 == 16);
    Ok(RandomAccessFile::try_new(file)?)
}

/// A WDL Table.
#[derive(Debug)]
pub struct WdlTable<F: ReadAt> {
    pub table: Table<WdlTag, F>,
}

impl<F: ReadAt + std::fmt::Debug> WdlTable<F> {
    pub fn new(raf: F, material: &Material) -> ProbeResult<WdlTable<F>> {
        Table::new(raf, material).map(|table| WdlTable { table })
    }

    pub fn probe_wdl(&self, g: &Game) -> ProbeResult<Wdl> {
        self.table.probe_wdl(g)
    }
}

impl WdlTable<RandomAccessFile> {
    pub fn open<P: AsRef<Path>>(path: P, material: &Material)
                                           -> ProbeResult<WdlTable<RandomAccessFile>> {
        WdlTable::new(open_table_file(path)?, material)
    }
}

/// A DTZ Table.
#[derive(Debug)]
pub struct DtzTable<F: ReadAt> {
    table: Table<DtzTag, F>,
}

impl<F: ReadAt + std::fmt::Debug> DtzTable<F> {
    pub fn new(raf: F, material: &Material) -> ProbeResult<DtzTable<F>> {
        Table::new(raf, material).map(|table| DtzTable { table })
    }

    pub fn probe_dtz(&self, g: &Game, wdl: DecisiveWdl) -> ProbeResult<Option<Dtz>> {
        self.table.probe_dtz(g, wdl)
    }
}

impl DtzTable<RandomAccessFile> {
    pub fn open<P: AsRef<Path>>(path: P, material: &Material) -> ProbeResult<DtzTable<RandomAccessFile>> {
        DtzTable::new(open_table_file(path)?, material)
    }
}

