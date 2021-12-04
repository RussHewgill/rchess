
use std::collections::HashMap;

use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;




#[derive(Debug,Clone)]
pub struct KillerMoves {
    // primary:     HashMap<Depth, Move>,
    // secondary:   HashMap<Depth, Move>,
    arr:         [[[[u8; 2]; 64]; 64]; 100],
}

impl Default for KillerMoves {
    fn default() -> Self {
        Self {
            arr:    [[[[0; 2]; 64]; 64]; 100],
        }
    }
}

impl KillerMoves {
    pub fn reset(&mut self) {
        self.arr = [[[[0; 2]; 64]; 64]; 100];
    }
    pub fn increment(&mut self, side: Color, ply: Depth, mv: &Move) {
        let from = u8::from(mv.sq_from());
        let to   = u8::from(mv.sq_to());
        self.arr[ply as usize][from as usize][to as usize][side] += 1;
    }
    pub fn get(&self, side: Color, ply: Depth, mv: &Move) -> u8 {
        let from = u8::from(mv.sq_from());
        let to   = u8::from(mv.sq_to());
        self.arr[ply as usize][from as usize][to as usize][side]
    }
}

// impl KillerMoves {
//     pub fn clear(&mut self) {
//         self.primary.clear();
//         self.secondary.clear();
//     }
//     pub fn insert(&mut self, depth: Depth, mv: &Move) {
//         unimplemented!()
//     }
//     pub fn get(&self, side: Color, ply: Depth, depth: Depth) -> (Option<Move>,Option<Move>) {
//         unimplemented!()
//     }
// }


