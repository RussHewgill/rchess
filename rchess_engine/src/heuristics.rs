
pub mod killer_moves;
pub mod butterfly;
pub mod counter_moves;

pub use self::killer_moves::*;
pub use self::butterfly::*;
pub use self::counter_moves::*;

use crate::types::*;

// TODO: tune
pub fn depth_stat_bonus(ply: Depth) -> Score {
    let ply = ply as Score;
    (ply * ply).min(250)
}

pub fn lmr_reduction(d: Depth, ms: u8) -> Depth {
    if ms < 4 {
        return 1;
    }
    (d / 3).min(1)
    // let mut r =
    // unimplemented!()
}

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

#[derive(Debug,Clone)]
pub struct CounterMoves {
    buf:        [[[Option<Move>; 64]; 64]; 2],
}

