
// pub mod killer_moves;
pub mod butterfly;
pub mod counter_moves;
pub mod capture_history;
pub mod continuation_history;

// pub use self::killer_moves::*;
pub use self::butterfly::*;
pub use self::counter_moves::*;
pub use self::capture_history::*;
pub use self::continuation_history::*;

use crate::types::*;
use crate::tuning::*;

use arrayvec::ArrayVec;

// pub type Score = Score;
// pub type Score = i16;

pub type ButterflyBoard = [[[Score; 64]; 64]; 2];
type HistoryPieceTo = [[Score; 64]; 6];

// #[derive(Debug,Clone)]
// pub struct KillerMoves {
//     primary:       [Option<Move>; 100],
//     secondary:     [Option<Move>; 100],
//     // tertiary:      [Option<Move>; 100],
//     // counter:       [[[[u8; 2]; 64]; 64]; 100],
// }

#[derive(Debug,Clone)]
pub struct ButterflyHistory {
    buf:        ButterflyBoard,
    // buf:           [[[(Score,Score); 64]; 64]; 2]
}

#[derive(Debug,Clone)]
/// [Piece][To][Piece][To]
/// XXX: include side?
pub struct ContinuationHistory {
    // buf:        [[HistoryPieceTo; 64]; 6],

    buf:           [[HistoryPieceTo; 64]; 6],
    buf_in_check:  [[HistoryPieceTo; 64]; 6],

    // buf:        [[[[Score; 64]; 6]; 64]; 6],
    // buf:           [[[(Score,Score); 64]; 64]; 2]
}

/// [Piece][To][CapturedPiece]
#[derive(Debug,Clone)]
pub struct CaptureHistory {
    buf:        [[[Score; 5]; 64]; 6],
}

// #[derive(Debug,Clone)]
// pub struct CounterMoveHistory {
//     buf:        [[[Score; 5]; 64]; 6],
// }

#[derive(Debug,Clone)]
/// [Side][Piece][To]
pub struct CounterMoves {
    buf:        [[[Option<Move>; 64]; 6]; 2],
    // buf:        [[Option<Move>; 64]; 6],
}

/// Bounded to i16 range when HISTORY_MAX is 512
pub fn update_stat_bonus(bonus: Score, mut current: &mut Score) {

    assert!(bonus.abs() <= 512);

    *current += 32 * bonus - *current * bonus.abs() / 512;
    // *current = *current + bonus;

    assert!((*current).abs() <= i16::MAX as Score);

    // *current += bonus - *current * bonus.abs() / 512;

    // *current = (*current).clamp(-HISTORY_MAX, HISTORY_MAX);
}
