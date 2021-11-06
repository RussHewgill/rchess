
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;



#[derive(Debug,Clone)]
pub struct NNUE {
    size_inputs:      usize,
    size_hidden:      usize,
    n_hidden:         usize,
    size_output:      usize,
}

impl NNUE {

    pub fn index(king_sq: Coord, pc: Piece, c0: Coord) -> usize {
        unimplemented!()
    }

}


