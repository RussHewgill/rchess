
use crate::evaluate::TaperedScore;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;

pub use self::piece_square_tables::*;

use serde::{Serialize,Deserialize};
use derive_new::new;

pub static LMR_MIN_MOVES: Depth = 2;
pub static LMR_MIN_PLY: Depth = 3;
pub static LMR_MIN_DEPTH: Depth = 3;

pub static LMR_REDUCTION: Depth = 3;
pub static LMR_PLY_CONST: Depth = 6;

pub static QS_RECAPS_ONLY: Depth = 5;
// pub static QS_RECAPS_ONLY: Depth = 100;

pub static NULL_PRUNE_MIN_DEPTH: Depth = 2;

mod piece_square_tables {
}

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize)]
pub struct EvalParams {
    pub pawns:     EPPawns,
    pub pieces:    EPPieces,
    #[serde(skip)]
    pub psqt:      PcTables,
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize,new)]
pub struct EPPieces {
    pub rook_open_file:  [Score; 2],
    pub outpost:         EvOutpost,
}

impl Default for EPPieces {
    fn default() -> Self {
        Self {
            rook_open_file:   [10,20],
            outpost:          EvOutpost::default(),
        }
    }
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize,new)]
pub struct EvOutpost {
    pub outpost_knight:     Score,
    pub outpost_bishop:     Score,
    pub reachable_knight:   Score,
    pub reachable_bishop:   Score,
}

impl Default for EvOutpost {
    fn default() -> Self {
        Self {
            // Self::new(50,30,30,0)
            outpost_knight:     50,
            outpost_bishop:     30,
            reachable_knight:   30,
            reachable_bishop:   0,
        }
    }
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize,new)]
pub struct EPPawns {
    pub doubled_isolated:     Score,
    pub isolated:             Score,
    pub backward:             Score,
    pub doubled:              Score,
    // pub connected:            Score,
    // pub passed:               Score,
    pub blocked_r5:           Score,
    pub blocked_r6:           Score,
}

impl Default for EPPawns {
    fn default() -> Self {
        Self {
            doubled_isolated:     10,
            isolated:             5,
            backward:             10,
            doubled:              10,
            // connected:            Score,
            blocked_r5:           -10,
            blocked_r6:           -5,
        }
    }
}

/// Passed bonus = passed * ranks past 2nd
// #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvPawn {
    pub backward: TaperedScore,
    pub doubled:  TaperedScore,
    pub isolated: TaperedScore,
    pub passed:   TaperedScore,
}

// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvRook {
//     pub rank7: TaperedScore,
// }

// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvKnight {
// }

// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvBishop {
// }

// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvQueen {
// }

// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvKing {
// }

// impl EvPawn {
//     pub fn new() -> Self {
//         Self {
//             backward: TaperedScore::new(-10, -10),
//             doubled:  TaperedScore::new(-15, -15),
//             isolated: TaperedScore::new(-20, -20),
//             passed:   TaperedScore::new(5,   10),
//         }
//     }
// }

// impl EvRook {
//     pub fn new() -> Self {
//         Self {
//             rank7: TaperedScore::new(20,40),
//         }
//     }
// }

// impl EvKnight {
//     pub fn new() -> Self {
//         Self {
//         }
//     }
// }

// impl EvBishop {
//     pub fn new() -> Self {
//         Self {
//         }
//     }
// }

// impl EvQueen {
//     pub fn new() -> Self {
//         Self {
//         }
//     }
// }

// impl EvKing {
//     pub fn new() -> Self {
//         Self {
//         }
//     }
// }


