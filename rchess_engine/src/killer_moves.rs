
use std::collections::HashMap;

use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;




#[derive(Debug,Default,Clone)]
pub struct KillerMoves {
    primary:     HashMap<Depth, Move>,
    secondary:   HashMap<Depth, Move>,
    // pub arr:         [[[u8; 64]; 64]; 100],
}

impl KillerMoves {
    pub fn clear(&mut self) {
        self.primary.clear();
        self.secondary.clear();
    }

    pub fn insert(&mut self, depth: Depth, mv: Move) {
        unimplemented!()
    }

    pub fn get(&self, depth: Depth) -> (Option<Move>,Option<Move>) {
        unimplemented!()
    }

}


