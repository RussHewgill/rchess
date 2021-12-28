
pub mod killer_moves;
pub mod butterfly;
pub mod counter_moves;
pub mod capture_history;

pub use self::killer_moves::*;
pub use self::butterfly::*;
pub use self::counter_moves::*;
pub use self::capture_history::*;

use crate::types::*;

use arrayvec::ArrayVec;

pub type ButterflyBoard = [[[Score; 64]; 64]; 2];

#[derive(Debug,Clone)]
pub struct KillerMoves {
    primary:       [Option<Move>; 100],
    secondary:     [Option<Move>; 100],
    // counter:       [[[[u8; 2]; 64]; 64]; 100],
}

#[derive(Debug,Clone)]
pub struct ButterflyHistory {
    buf:        ButterflyBoard,
}

/// [Piece][To][CapturedPiece]
#[derive(Debug,Clone)]
pub struct CaptureHistory {
    buf:        [[[i16; 5]; 64]; 6],
}

#[derive(Debug,Clone)]
pub struct CounterMoves {
    buf:        [[[Option<Move>; 64]; 6]; 2],
    // buf:        [[Option<Move>; 64]; 6],
}

