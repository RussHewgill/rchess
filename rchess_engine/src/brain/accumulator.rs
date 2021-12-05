
use crate::types::*;
use crate::tables::*;
use crate::brain::types::nnue::{NNUE_INPUT,NNUE_L2,NNUE_L3,NNUE_OUTPUT};
use crate::brain::types::*;

pub use self::gradient::*;

use arrayvec::ArrayVec;
use itertools::Itertools;
use na::DVector;
use nd::ArrayView2;
use ndarray as nd;
use nd::{Array2};
use nalgebra as na;

use sprs::CsMat;

use rand::prelude::StdRng;
use serde::{Serialize,Deserialize};
use serde_big_array::BigArray;

// use num_

mod old2 {

    #[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
    pub struct NNUE3 {
        // #[serde(skip,default = "NNUE3::def_inputs")]
        // pub inputs:       CsMat<i8>,
        // pub inputs:       Array2<f32>, // 769,1
        pub inputs:       na::DVector<f32>,
        pub nn:           DNetwork<f32, 769, 1>,
    }

    /// Construction
    impl NNUE3 {
        pub const INPUT_SIZE: usize = 769;

        pub fn new(mm: (f32,f32), mut rng: &mut StdRng) -> Self {
            Self {
                // inputs:    Self::def_inputs(),
                // inputs:    Array2::zeros((Self::INPUT_SIZE,1)),
                inputs:    na::DVector::<f32>::zeros(Self::INPUT_SIZE),
                nn:        DNetwork::_new_rng(vec![769,128,1], mm, &mut rng),
            }
        }

        // fn def_inputs() -> CsMat<i8> {
        //     sprs::CsMat::zero((Self::INPUT_SIZE,1)).into_csc()
        // }

    }

    /// Run
    impl NNUE3 {

        pub fn run_fresh(&mut self, g: &Game) -> f32 {
            self.init_inputs(g);
            self.run_partial()
        }

        pub fn run_partial(&self) -> f32 {
            let out = self.nn.run(&self.inputs);
            out[(0,0)]
        }
    }

    /// Backprop
    impl NNUE3 {
        pub fn backprop(&mut self, g: Option<&Game>, correct: f32, eta: f32) {
            if let Some(g) = g { self.init_inputs(g); }

            let ins: Vec<(DVector<f32>,DVector<f32>)> = vec![
                (self.inputs.clone(), DVector::from_vec(vec![correct]))
            ];
            self.nn.backprop_mut_matrix(&ins, eta);
        }
    }

    /// Increment
    impl NNUE3 {
    }

    /// Inputs
    impl NNUE3 {

        pub fn init_inputs(&mut self, g: &Game) {
            self.inputs.fill(0.0);
            for side in [White,Black] {
                for pc in Piece::iter_pieces() {
                    g.get(pc, side).into_iter().for_each(|sq| {
                        let idx = Self::make_index(side, pc, sq);
                        // self.inputs[(idx,1)] = 1.0;
                        self.inputs[idx] = 1.0;
                    });
                }
            }
            if g.state.side_to_move == White {
                self.inputs[Self::INPUT_SIZE - 1] = 1.0;
            }
        }

        pub fn make_index(side: Color, pc: Piece, c0: u8) -> usize {
            let pc = pc.index();
            let pc = if side == White { pc } else { pc + 6 };
            pc as usize * 64 + c0 as usize
        }
    }

}

mod old {
    use super::*;

    #[derive(Debug,PartialEq,Clone,Copy)]
    pub enum AccDelta {
        Move   { pc: Piece, from: u8, to: u8 },
        Add    { pc: Piece, sq: u8 },
        Delete { pc: Piece, sq: u8 },
    }

    impl AccDelta {
        pub fn piece(&self) -> Piece {
            match self {
                Self::Move { pc, .. }   => *pc,
                Self::Add { pc, .. }    => *pc,
                Self::Delete { pc, .. } => *pc,
            }
        }
    }

    #[derive(Debug,PartialEq,Serialize,Deserialize,Clone)]
    pub struct Accum<const MAXPLY: usize> {
        pub side:           Color,

        pub accurate:       bool,

        #[serde(skip)]
        pub deltas:         ArrayVec<AccDelta, MAXPLY>,

        #[serde(skip)]
        pub values_own:     Array2<i16>,
        #[serde(skip)]
        pub values_other:   Array2<i16>,

    }

    impl<const MAXPLY: usize> Accum<MAXPLY> {
        pub fn new(side: Color) -> Self {
            Self {
                side,
                accurate:     true,
                deltas:       ArrayVec::default(),
                // values_own:   [0; NNUE_L2],
                // values_other: [0; NNUE_L2],
                values_own:   Array2::zeros((NNUE_L2,1)),
                values_other: Array2::zeros((NNUE_L2,1)),
            }
        }
    }

    impl<const MAXPLY: usize> Accum<MAXPLY> {

        pub fn push(&mut self, g: &Game) {
            unimplemented!()
        }

        pub fn move_piece(&mut self, pc: Piece, from: u8, to: u8) {
            self.accurate = false;
            self.deltas.push(AccDelta::Move { pc, from, to });
        }

        pub fn add_piece(&mut self, pc: Piece, sq: u8) {
            self.accurate = false;
            self.deltas.push(AccDelta::Add { pc, sq });
        }

        pub fn delete_piece(&mut self, pc: Piece, sq: u8) {
            self.accurate = false;
            self.deltas.push(AccDelta::Delete { pc, sq });
        }

        pub fn apply_deltas(&mut self, ws: &ArrayView2<i8>, bs: &ArrayView2<i32>, g: &Game) {

            let ds = self.deltas.clone();
            self.deltas.clear();

            let mut adds = vec![];
            let mut rms  = vec![];

            for d in ds.into_iter() {
                if d.piece() == King {
                    self.init_inputs(g);
                    continue;
                }
                match d {
                    AccDelta::Move   { pc, from, to } => {
                        rms.push((pc,from));
                        adds.push((pc,to));
                    },
                    AccDelta::Add    { pc, sq } => {
                        adds.push((pc,sq));
                    },
                    AccDelta::Delete { pc, sq } => {
                        rms.push((pc,sq));
                    },
                }
            }
            self.accurate = true;

            for (pc,sq) in adds.into_iter() {
            }

        }

    }

    impl<const MAXPLY: usize> Accum<MAXPLY> {
        // fn increment_act_own(&mut self, idx: usize, add: bool) {
        // }
    }

    impl<const MAXPLY: usize> Accum<MAXPLY> {

        // TODO: En Passant
        // TODO: Castling
        pub fn init_inputs(&mut self, g: &Game) {

            self.values_own.fill(0);
            self.values_other.fill(0);
            self.accurate     = true;

            // XXX: ?
            self.deltas.clear();

            let c_own   = g.get_color(self.side);
            let c_other = g.get_color(!self.side);
            let kings   = g.get_piece(King);

            let pcs = (c_own | c_other) & !kings;

            let king_sq_own   = g.get(King, self.side).bitscan();
            let king_sq_other = g.get(King, !self.side).bitscan();

            const PCS: [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];

            for pc in PCS {
                let side = !self.side;
                g.get(pc, side).into_iter().for_each(|sq| {
                    trace!("Setting own king at {:?} with {:?} at {:?}",
                        Coord::from(king_sq_own), pc, Coord::from(sq));
                    // self.update_insert_piece(king_sq_own, king_sq_other, pc, sq, false);

                    // let idx = self.index_halfka(king_sq_own, pc, self.side, sq);
                    let idx0 = self.index_halfkp(king_sq_own, pc, side, sq);
                    let idx1 = self.index_halfkp(king_sq_other, pc, side, sq);

                    // self.values_own[]

                });
                let side = !self.side;
                g.get(pc, side).into_iter().for_each(|sq| {
                    trace!("Setting other king at {:?} with {:?} at {:?}",
                        Coord::from(king_sq_other), pc, Coord::from(sq));
                    // self.update_insert_piece(king_sq_other, king_sq_own, pc, sq, false);

                    // let idx = self.index_halfka(king_sq_own, pc, !self.side, sq);

                    let idx0 = self.index_halfkp(king_sq_own, pc, side, sq);
                    let idx1 = self.index_halfkp(king_sq_other, pc, side, sq);


                });
            }

            // TODO: En Passant

            // TODO: Castling

        }

    }

    impl<const MAXPLY: usize> Accum<MAXPLY> {

        pub fn sq64_to_sq32(sq: u8) -> usize {
            const MIRROR: [u8; 8] = [ 3, 2, 1, 0, 0, 1, 2, 3 ];
            ((sq as usize >> 1) & !0x3) + MIRROR[(sq & 0x7) as usize] as usize
        }

        pub fn index_halfka(&self, king_sq: u8, pc: Piece, side: Color, sq: u8) -> usize {

            // let rel_k  = BitBoard::relative_square(side, king_sq);
            // let rel_sq = BitBoard::relative_square(side, sq);

            // assert!(rel_k != sq);

            // let mksq = if FLANK_LEFT.is_one_at(rel_k.into()) {
            //     rel_k ^ 0x7 } else { rel_k };
            // let mpsq = if FLANK_LEFT.is_one_at(rel_k.into()) {
            //     rel_sq ^ 0x7 } else { rel_sq };

            let cc = if self.side == side { 1 } else { 0 };

            // // 640 * mksq as usize + (64 * (5 * cc + pc.index())) + mpsq as usize
            // // 640 * rel_k as usize + (64 * (5 * cc + pc.index())) + rel_sq as usize
            // 640 * king_sq as usize + (64 * (5 * cc + pc.index())) + sq as usize

            // (640 * king_sq as usize)
            //     + (64 * (5 * cc + pc.index())) + sq as usize
            unimplemented!()
        }

        pub fn index_halfkp(&self, king_sq: u8, pc: Piece, side: Color, sq: u8) -> usize {
            let cc = if self.side == side { 1 } else { 0 };
            let pi = pc.index() * 2 + cc;
            sq as usize + (pi + king_sq as usize * 11) * 64
        }

        /// Mirrored so king is always on (a..d) or (e..h) ??
        pub fn index_halfka_m(&self, king_sq: u8, pc: Piece, side: Color, sq: u8) -> usize {
            let rel_k  = BitBoard::relative_square(side, king_sq);
            let rel_sq = BitBoard::relative_square(side, sq);

            let mksq = if FLANK_LEFT.is_one_at(rel_k.into()) {
                rel_k ^ 0x7 } else { rel_k };
            let mpsq = if FLANK_LEFT.is_one_at(rel_k.into()) {
                rel_sq ^ 0x7 } else { rel_sq };

            let cc = if self.side == side { 1 } else { 0 };

            640 * Self::sq64_to_sq32(mksq) + (64 * (5 * cc + pc.index())) + mpsq as usize
        }

        fn index_en_passant_halfkp(c0: Coord) -> Option<usize> {
            const K: usize = 63 * 64 * 10;
            if c0.1 == 0 || c0.1 == 1 || c0.1 == 6 || c0.1 == 7 {
                return None;
            }
            let c0 = BitBoard::index_square(c0) as usize - 16;
            // Some(K + c0)
            unimplemented!()
        }

        // /// https://github.com/glinscott/nnue-pytorch/blob/master/docs/nnue.md
        // /// https://github.com/AndyGrant/EtherealDev/blob/openbench_nnue/src/nnue/accumulator.c
        // pub fn index_halfkp(king_sq: u8, pc: Piece, c0: u8, friendly: bool) -> usize {
        //     assert_ne!(pc, King);
        //     let f = if friendly { 1 } else { 0 };
        //     let pi = pc.index() * 2 + f;
        //     c0 as usize + (pi + king_sq as usize * 10) * 63
        //     // (640 * king_sq as usize)
        //     //     + (63 * (5 * f + pc.index())) + c0 as usize
        // }

    }

}

