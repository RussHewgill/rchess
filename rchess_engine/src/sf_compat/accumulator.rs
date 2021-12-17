
use crate::types::*;
use super::{NNIndex, HALF_DIMS};

use arrayvec::ArrayVec;
use aligned::{Aligned,A32};

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub enum NNDelta {
    Add(NNIndex,NNIndex),
    Remove(NNIndex,NNIndex),
    // Add(usize,(Coord,Color,Piece,Color,Coord)),
    // Remove(usize,(Coord,Color,Piece,Color,Coord)),
    // Copy,
}

impl NNDelta {
    pub fn get(self) -> (NNIndex,NNIndex) {
        match self {
            Self::Add(a,b) => (a,b),
            Self::Remove(a,b) => (a,b),
        }
    }
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
pub enum NNDeltas {
    Deltas(ArrayVec<NNDelta,3>),
    Copy,
    // // CopyCastle(Color,(NNIndex,NNIndex),(NNIndex,NNIndex)),
    // CopyCastle(Color),
    // CopyKing(Color,(NNIndex,NNIndex)),
}

// #[derive(Debug,Eq,PartialEq,PartialOrd,Ord,Clone,Copy)]
// #[repr(align(64))]
// pub struct NNAccumData {
//     pub side:            Color,
//     // pub accum:           [i16; 1024], // TransformedFeatureDimensions = 1024
//     // pub psqt:            [i32; 8],    // PSQTBuckets = 8
//     pub accum:           [[i16; 1024]; 2], // TransformedFeatureDimensions = 1024
//     pub psqt:            [[i32; 8]; 2],    // PSQTBuckets = 8
// }

#[derive(Debug,Eq,PartialEq,PartialOrd,Ord,Clone)]
// #[repr(align(64))]
pub enum NNAccumData {
    // Half(Color, [i16; 1024], [i32; 8]),
    Full(Color, Aligned<A32,[[i16; 1024]; 2]>, Aligned<A32,[[i32; 8]; 2]>),
}

// #[derive(Debug,PartialEq,Clone,Copy)]
#[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
// #[repr(align(64))]
pub struct NNAccum {
    // pub accum:           [[i16; 1024]; 2], // TransformedFeatureDimensions = 1024
    // pub psqt:            [[i32; 8]; 2],    // PSQTBuckets = 8
    pub accum:           Aligned<A32,[[i16; 1024]; 2]>, // TransformedFeatureDimensions = 1024
    pub psqt:            Aligned<A32,[[i32; 8]; 2]>,    // PSQTBuckets = 8

    // pub computed:   [bool; 2],
    // pub deltas:     ArrayVec<NNDelta, 9>, // 3 moves

    // pub deltas_add:      ArrayVec<usize, 6>, // 2 moves
    // pub deltas_rem:      ArrayVec<usize, 6>, // 2 moves
    // pub stack:           ArrayVec<NNDelta, 300>,
    // pub stack:           Vec<NNDelta>,

    // pub stack_delta:        Vec<NNDelta>,
    // pub stack_delta:        Vec<ArrayVec<NNDelta,8>>,
    pub stack_delta:        Vec<NNDeltas>,
    pub stack_copies:       Vec<NNAccumData>,

    // pub needs_refresh:   [bool; 2],
}

/// New
impl NNAccum {
    pub fn new() -> Self {
        Self {
            accum:            Aligned([[0; 1024]; 2]),
            psqt:             Aligned([[0; 8]; 2]),
            // deltas_add:       ArrayVec::default(),
            // deltas_rem:       ArrayVec::default(),
            // stack:            ArrayVec::default(),

            stack_delta:      Vec::with_capacity(1024),
            stack_copies:     Vec::with_capacity(1024),

            // needs_refresh:    [true; 2],
        }
    }
}

/// Delta
impl NNAccum {

    pub fn make_copy(&self, side: Color) -> NNAccumData {
        NNAccumData::Full(
            side,
            // accum:  self.accum[side],
            // psqt:   self.psqt[side],
            self.accum.clone(),
            self.psqt.clone(),
        )
    }

    // pub fn make_copy_half(&self, side: Color) -> NNAccumData {
    //     NNAccumData::Half(
    //         side,
    //         self.accum[side],
    //         self.psqt[side],
    //     )
    // }

    pub fn push_copy_full(&mut self, side: Color) {
        let delta = self.make_copy(side);
        self.stack_delta.push(NNDeltas::Copy);
        self.stack_copies.push(delta);
    }

    // pub fn push_copy_half(&mut self, side: Color, xs: (NNIndex,NNIndex)) {
    //     let delta = self.make_copy_half(side);
    //     self.stack_delta.push(NNDeltas::CopyKing(side,xs));
    //     self.stack_copies.push(delta);
    // }

    // pub fn push_copy_king(&mut self, side: Color, xs: (NNIndex,NNIndex)) {
    //     let delta = self.make_copy(side);
    //     self.stack_delta.push(NNDeltas::CopyKing(side,xs));
    //     self.stack_copies.push(delta);
    // }

    // // pub fn push_copy_castle(&mut self, side: Color, (xs,ys): ((NNIndex,NNIndex),(NNIndex,NNIndex))) {
    // pub fn push_copy_castle(&mut self) {
    //     let delta_w = self.make_copy(White);
    //     let delta_b = self.make_copy(Black);
    //     // self.stack_delta.push(NNDeltas::CopyCastle(side,xs,ys));
    //     self.stack_delta.push(NNDeltas::CopyCastle(White));
    //     self.stack_delta.push(NNDeltas::CopyCastle(Black));
    //     self.stack_copies.push(delta_w);
    //     self.stack_copies.push(delta_b);
    // }

    pub fn pop_prev(&mut self) {
        if let Some(prev) = self.stack_copies.pop() {
            match prev {
                NNAccumData::Full(side, accum, psqt) => {
                    self.accum = accum;
                    self.psqt  = psqt;
                },
                // NNAccumData::Half(side, accum, psqt) => {
                //     self.accum[side].copy_from_slice(&accum);
                //     self.psqt[side].copy_from_slice(&psqt);
                // },
            }
        }
    }

}

/// Append Active
impl NNAccum {

    pub fn append_active(g: &Game, persp: Color, mut active: &mut ArrayVec<NNIndex, 32>) {
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


