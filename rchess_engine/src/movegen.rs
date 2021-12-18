
use crate::types::*;
use crate::tables::*;

use arrayvec::ArrayVec;

#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub enum MoveGenType {
    Captures,
    Quiets,
    Pseudo,
    AllLegal,
}

#[derive(Debug,Clone)]
pub struct MoveGen {
}

impl Iterator for MoveGen {
    type Item = Move;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

/// Generate
impl MoveGen {
}

/// Pawns
impl MoveGen {
    pub fn gen_pawns(&self, ts: &Tables, gen: MoveGenType) -> Vec<Move> {
        unimplemented!()
    }
}

/// Knights
impl MoveGen {
    pub fn gen_knights(&self, ts: &Tables, gen: MoveGenType) -> Vec<Move> {
        unimplemented!()
    }
}






