
use std::collections::HashMap;

use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;




#[derive(Debug,Clone)]
pub struct KillerMoves {
    primary:       [Option<Move>; 100],
    secondary:     [Option<Move>; 100],
    counter:       [[[[u8; 2]; 64]; 64]; 100],
}

impl Default for KillerMoves {
    fn default() -> Self {
        Self {
            primary:    [None; 100],
            secondary:  [None; 100],
            counter:    [[[[0; 2]; 64]; 64]; 100],
        }
    }
}

impl KillerMoves {

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn get(&self, side: Color, ply: Depth) -> (Option<Move>,Option<Move>) {
        (self.primary[ply as usize],self.secondary[ply as usize])
        // (self.primary.get(ply as usize).unwrap(),self.secondary.get(ply as usize).unwrap())
    }

    // fn increment(&mut self, side: Color, ply: Depth, mv: &Move) {
    //     let from = u8::from(mv.sq_from());
    //     let to   = u8::from(mv.sq_to());
    //     self.counter[ply as usize][from as usize][to as usize][side] += 1;
    // }

    pub fn store(&mut self, side: Color, ply: Depth, mv: Move) {
        // self.increment(side, ply, &mv);
        if let Some(prev) = self.primary[ply as usize] {
            if prev != mv {
                self.secondary[ply as usize] = Some(prev);
                self.primary[ply as usize] = Some(mv);
            }
        } else {
            self.primary[ply as usize] = Some(mv);
        }
    }

    // fn get(&self, side: Color, ply: Depth, mv: &Move) -> u8 {
    //     let from = u8::from(mv.sq_from());
    //     let to   = u8::from(mv.sq_to());
    //     self.arr[ply as usize][from as usize][to as usize][side]
    // }

}

// impl KillerMoves {
//     pub fn reset(&mut self) {
//         self.arr = [[[[0; 2]; 64]; 64]; 100];
//     }
//     pub fn increment(&mut self, side: Color, ply: Depth, mv: &Move) {
//         let from = u8::from(mv.sq_from());
//         let to   = u8::from(mv.sq_to());
//         self.arr[ply as usize][from as usize][to as usize][side] += 1;
//     }
//     pub fn get(&self, side: Color, ply: Depth, mv: &Move) -> u8 {
//         let from = u8::from(mv.sq_from());
//         let to   = u8::from(mv.sq_to());
//         self.arr[ply as usize][from as usize][to as usize][side]
//     }
// }

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


