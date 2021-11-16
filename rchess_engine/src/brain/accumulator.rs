
use crate::types::*;
use crate::tables::*;

use arrayvec::ArrayVec;
use itertools::Itertools;
use ndarray as nd;
use nd::{Array2};

use serde::{Serialize,Deserialize};
use serde_big_array::BigArray;

#[derive(Debug,PartialEq,Clone,Copy)]
pub struct AccDelta {
    pub pc:    Piece,
    pub from:  u8,
    pub to:    u8,
}

#[derive(Debug,PartialEq,Serialize,Deserialize,Clone)]
pub struct Accum<const MAXPLY: usize> {
    pub side:         Color,

    pub accurate:     bool,
    pub changes:      usize,

    #[serde(skip)]
    pub deltas:       ArrayVec<AccDelta, MAXPLY>,

    #[serde(with = "BigArray")]
    pub values_own:   [i16; 192],
    #[serde(with = "BigArray")]
    pub values_other: [i16; 192],

    // pub inputs_own:   

}

impl<const MAXPLY: usize> Accum<MAXPLY> {

    pub fn new(side: Color) -> Self {
        Self {
            side,
            accurate:     true,
            changes:      0,
            deltas:       ArrayVec::default(),
            values_own:   [0; 192],
            values_other: [0; 192],
        }
    }

    pub fn push(&mut self, g: &Game) {
        unimplemented!()
    }

    pub fn move_piece(&mut self, pc: Piece, from: u8, to: u8) {
        unimplemented!()
    }

    pub fn add_piece(&mut self, pc: Piece, sq: u8) {
        unimplemented!()
    }

    pub fn delete_piece(&mut self, pc: Piece, sq: u8) {
        unimplemented!()
    }

}

impl<const MAXPLY: usize> Accum<MAXPLY> {

    pub fn init_inputs(&mut self, g: &Game) {

        self.values_own   = [0; 192];
        self.values_other = [0; 192];
        self.accurate     = true;

        let c_own   = g.get_color(self.side);
        let c_other = g.get_color(!self.side);
        let kings   = g.get_piece(King);

        let pcs = (c_own | c_other) & !kings;

        let king_sq_own   = g.get(King, self.side).bitscan();
        let king_sq_other = g.get(King, !self.side).bitscan();

        // pcs.into_iter().for_each(|sq| {
        //     // let idx0 = Self::index(king_sq_own, pc, c0, friendly)
        // });

        // const COLORS: [Color; 2] = [White,Black];
        const PCS: [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];

        for pc in PCS {
            g.get(pc, self.side).into_iter().for_each(|sq| {
                trace!("Setting own king at {:?} with {:?} at {:?}",
                       Coord::from(king_sq_own), pc, Coord::from(sq));
                // self.update_insert_piece(king_sq_own, king_sq_other, pc, sq, false);

                let idx = Self::index(king_sq_own, pc, sq, true);

                // let inputs = 

            });
            g.get(pc, !self.side).into_iter().for_each(|sq| {
                trace!("Setting other king at {:?} with {:?} at {:?}",
                       Coord::from(king_sq_other), pc, Coord::from(sq));
                // self.update_insert_piece(king_sq_other, king_sq_own, pc, sq, false);

                let idx = Self::index(king_sq_own, pc, sq, false);
            });
        }

    }

}

impl<const MAXPLY: usize> Accum<MAXPLY> {

    pub fn sq64_to_sq32(sq: u8) -> u8 {
        const MIRROR: [u8; 8] = [ 3, 2, 1, 0, 0, 1, 2, 3 ];
        ((sq >> 1) & !0x3) + MIRROR[(sq & 0x7) as usize]
    }

    // pub fn index_delta(king_sq: u8, pc: Piece, sq: u8, side: Color) -> usize {
    //     let rel_k  = BitBoard::relative_square(side, king_sq);
    //     let rel_sq = BitBoard::relative_square(side, sq);
    //     let mksq = if FLANK_LEFT.is_one_at(rel_k.into()) {
    //         rel_k ^ 0x7 } else { rel_k };
    //     let mpsq = if FLANK_LEFT.is_one_at(rel_k.into()) {
    //         rel_sq ^ 0x7 } else { rel_sq };
    //     // 640 * sq64_to_sq32(mksq) + (64 * (5 * (colour == pcolour) + ptype)) + mpsq
    //     unimplemented!()
    // }

    fn index_en_passant(c0: Coord) -> Option<usize> {
        const K: usize = 63 * 64 * 10;
        if c0.1 == 0 || c0.1 == 1 || c0.1 == 6 || c0.1 == 7 {
            return None;
        }
        let c0 = BitBoard::index_square(c0) as usize - 16;
        Some(K + c0)
    }

    /// https://github.com/glinscott/nnue-pytorch/blob/master/docs/nnue.md
    /// https://github.com/AndyGrant/EtherealDev/blob/openbench_nnue/src/nnue/accumulator.c
    pub fn index(king_sq: u8, pc: Piece, c0: u8, friendly: bool) -> usize {
        assert_ne!(pc, King);
        let f = if friendly { 1 } else { 0 };

        let pi = pc.index() * 2 + f;
        c0 as usize + (pi + king_sq as usize * 10) * 63

        // (640 * king_sq as usize)
        //     + (63 * (5 * f + pc.index())) + c0 as usize

    }

}

