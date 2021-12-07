
use crate::types::*;


#[derive(Debug,PartialEq,Clone,Copy)]
pub struct NNAccum {
    pub accum:      [[i16; 1024]; 2], // TransformedFeatureDimensions = 1024
    pub psqt:       [[i32; 8]; 2],    // PSQTBuckets = 8
    pub computed:   [bool; 2],
}

/// New
impl NNAccum {
    pub fn new() -> Self {
        Self {
            accum:     [[0; 1024]; 2],
            psqt:      [[0; 8]; 2],
            computed:  [false; 2],
        }
    }
}

impl NNAccum {

    pub fn append_active(g: &Game, persp: Color, mut active: &mut Vec<usize>) {
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


