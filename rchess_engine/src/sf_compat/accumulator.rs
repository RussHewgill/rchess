
use arrayvec::ArrayVec;

use crate::types::*;

#[derive(Debug,Eq,PartialEq,PartialOrd,Ord,Clone,Copy)]
pub enum NNDelta {
    Add(usize),
    Remove(usize),
}

// #[derive(Debug,PartialEq,Clone,Copy)]
#[derive(Debug,Eq,PartialEq,PartialOrd,Ord,Clone)]
pub struct NNAccum {
    pub accum:           [[i16; 1024]; 2], // TransformedFeatureDimensions = 1024
    pub psqt:            [[i32; 8]; 2],    // PSQTBuckets = 8

    // pub computed:   [bool; 2],
    // pub deltas:     ArrayVec<NNDelta, 9>, // 3 moves

    // pub deltas_add:      ArrayVec<usize, 6>, // 2 moves
    // pub deltas_rem:      ArrayVec<usize, 6>, // 2 moves
    // pub stack:           ArrayVec<NNDelta, 300>,
    pub stack:           Vec<NNDelta>,

    pub needs_refresh:   [bool; 2],
}

/// New
impl NNAccum {
    pub fn new() -> Self {
        Self {
            accum:            [[0; 1024]; 2],
            psqt:             [[0; 8]; 2],
            // deltas_add:       ArrayVec::default(),
            // deltas_rem:       ArrayVec::default(),
            // stack:            ArrayVec::default(),
            stack:            Vec::with_capacity(1024),
            needs_refresh:    [true; 2],
        }
    }
}

impl NNAccum {

    // pub fn push_delta_move(
    //     &mut self,
    //     persp:      Color,
    //     king_sq:    Coord,
    //     pc:         Piece,
    //     side:       Color,
    //     from:       Coord,
    //     to:         Coord,
    // ) {
    //     self.push_delta_rem(persp, king_sq, pc, side, from);
    //     self.push_delta_add(persp, king_sq, pc, side, to);
    // }

    // pub fn push_delta_add(
    //     &mut self, persp: Color, king_sq: Coord, pc: Piece, side: Color, sq: Coord) {
    //     let idx = super::NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
    //     self.deltas_add.push(idx);
    // }

    // pub fn push_delta_rem(
    //     &mut self, persp: Color, king_sq: Coord, pc: Piece, side: Color, sq: Coord) {
    //     let idx = super::NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
    //     self.deltas_rem.push(idx);
    // }

    // pub fn refresh(&mut self, g: &Game, persp: Color) {
    //     self.psqt[persp].fill(0);
    //     let mut active = ArrayVec::default();
    //     Self::append_active(g, persp, &mut active);
    // }

    pub fn append_active(g: &Game, persp: Color, mut active: &mut ArrayVec<usize, 32>) {
        let king_sq = g.get(King,persp).bitscan();

        for side in [White,Black] {
            for pc in Piece::iter_pieces() {
                // if side == persp && pc == King { continue; }
                g.get(pc,side).into_iter().for_each(|sq| {
                    let idx = super::NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
                    active.push(idx);
                });
            }
        }
    }

}


