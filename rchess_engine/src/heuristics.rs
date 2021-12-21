
pub mod killer_moves;
pub mod butterfly;

pub use self::killer_moves::*;
pub use self::butterfly::*;

use crate::types::*;

#[derive(Debug,Clone)]
pub struct KillerMoves {
    primary:       [Option<Move>; 100],
    secondary:     [Option<Move>; 100],
    counter:       [[[[u8; 2]; 64]; 64]; 100],
}

#[derive(Debug,Clone)]
pub struct ButterflyHistory {
    buf:        [[[i16; 64]; 64]; 2]
}

