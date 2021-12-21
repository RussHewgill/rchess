
use crate::types::*;
use crate::tables::*;
use crate::heuristics::CounterMoves;

impl Default for CounterMoves {
    fn default() -> Self {
        Self {
            buf:      [[[0; 64]; 64]; 2],
        }
    }
}


