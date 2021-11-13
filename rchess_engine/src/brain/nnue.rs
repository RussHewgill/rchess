
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
use crate::brain::types::*;
use crate::brain::types::nnue::*;
use crate::brain::sigmoid;

// use ndarray::prelude::*;
// use ndarray_rand::RandomExt;
// use ndarray_rand::rand_distr::Uniform;

// use ndarray::prelude::*;
use ndarray as nd;

use nalgebra::{SMatrix,SVector,Matrix,Vector,DVector,DMatrix,Dynamic,Const};
use nalgebra as na;

use rand::{Rng,SeedableRng};
use rand::prelude::{StdRng,Distribution};
use rand::distributions::Uniform;

impl NNUE {

    pub fn swap_sides(&mut self) {
        // std::mem::swap(&mut self.weights_in_own, &mut self.weights_in_other);
        // std::mem::swap(&mut self.weights_l2_own, &mut self.weights_l2_other);
        unimplemented!()
    }

    fn update_insert_piece(&mut self, king_sq: u8, pc: Piece, c0: u8, friendly: bool) {
        let idx0 = Self::index(king_sq, pc, c0, friendly);
        let idx1 = Self::index(king_sq, pc, c0, !friendly);
        self.inputs_own[(idx0,0)]   = 1.0;
        self.inputs_other[(idx1,0)] = 1.0;
    }

    fn update_delete_piece(&mut self, king_sq: u8, pc: Piece, c0: u8, friendly: bool) {
        let idx0 = Self::index(king_sq, pc, c0, friendly);
        let idx1 = Self::index(king_sq, pc, c0, !friendly);
        self.inputs_own[(idx0,0)]   = 0.0;
        self.inputs_other[(idx1,0)] = 0.0;
    }

    fn update_move_piece(&mut self, king_sq: u8, pc: Piece, c0: u8, c1: u8, friendly: bool) {
        self.update_delete_piece(king_sq, pc, c0, friendly);
        self.update_insert_piece(king_sq, pc, c1, friendly);
    }

    /// Called AFTER game has had move applied?
    pub fn update_move(&mut self, g: &Game, mv: Move) {

        let king_sq_own: Coord   = g.get(King, g.state.side_to_move).bitscan().into();
        let king_sq_other: Coord = g.get(King, !g.state.side_to_move).bitscan().into();

        if mv.piece() == Some(King) {
            unimplemented!()
        }

        match mv {
            _ => unimplemented!()
        }

        // match mv.piece() {
        //     Some(King) => {
        //         unimplemented!()
        //     },
        //     Some(pc) => {
        //         // let idx = Self::index(king_sq, pc, c0, side);
        //     },
        //     None => {
        //     },
        // }

    }

    pub fn run_fresh(&mut self, g: &Game) -> f32 {

        // XXX: 
        // self.init_inputs(g);

        // let mut last = &self.inputs_own;

        let z0_own = self.weights_in_own.dot(&self.inputs_own);
        let z0_other = self.weights_in_other.dot(&self.inputs_other);

        self.activations1_own = z0_own.clone();
        self.activations1_other = z0_other.clone();

        let act1 = nd::concatenate![nd::Axis(0), z0_own, z0_other];

        let z2 = self.weights_l2.dot(&act1);
        // let act2 = z2.map(Self::relu);
        let act2 = z2.map(|x| sigmoid(*x));

        let z3 = self.weights_l3.dot(&act2);
        // let act3 = z3.map(Self::relu);
        let act3 = z3.map(|x| sigmoid(*x));

        let z_out = self.weights_out.dot(&act3);
        // let act_out = z_out.map(Self::relu);
        let act_out = z_out.map(|x| sigmoid(*x));

        let s0 = z0_own.shape();
        let s1 = act1.shape();
        let s2 = z2.shape();
        let s3 = z3.shape();
        let s4 = z_out.shape();

        act_out[(0,0)]
        // unimplemented!()
    }

    fn relu(x: &f32) -> f32 {
        x.max(0.0)
    }

}

impl NNUE {

    pub fn init_inputs(&mut self, g: &Game) {
        const COLORS: [Color; 2] = [White,Black];
        const PCS: [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];

        self.inputs_own.fill(0.0);
        self.inputs_other.fill(0.0);

        for side in COLORS {
            let mut indices = vec![];

            // let king_sq: Coord   = g.get(King, side).bitscan().into();
            let king_sq = g.get(King, side).bitscan();

            let friendly = side == g.state.side_to_move;

            for pc in PCS {
                g.get(pc, side).into_iter().for_each(|sq| {
                    let idx = Self::index(king_sq, pc, sq, friendly);
                    indices.push(idx);
                });
            }

            if let Some(c_ep) = g.state.en_passant {
                if let Some(ep) = Self::index_en_passant(c_ep) {
                    indices.push(ep);
                }
            }

            let cs = Self::index_castling(&g.state.castling);
            for i in cs.into_iter() {
                indices.push(i);
            }

            for i in indices.into_iter() {
                if side == g.state.side_to_move {
                    self.inputs_own[(i,0)] = 1.0;
                } else {
                    self.inputs_other[(i,0)] = 1.0;
                }
            }
        }

    }

    fn index_en_passant(c0: Coord) -> Option<usize> {
        const K: usize = 63 * 64 * 10;
        if c0.1 == 0 || c0.1 == 1 || c0.1 == 6 || c0.1 == 7 {
            return None;
        }
        let c0 = BitBoard::index_square(c0) as usize - 16;
        Some(K + c0)
    }

    // XXX: 8 ?
    fn index_castling(c: &Castling) -> Vec<usize> {
        const K: usize = 63 * 64 + 10 + 32;

        eprintln!("TODO: castling");
        // unimplemented!()
        vec![]
    }

    // pub fn index(king_sq: Coord, pc: Piece, c0: Coord, friendly: bool) -> usize {
    pub fn index(king_sq: u8, pc: Piece, c0: u8, friendly: bool) -> usize {
        // let king_sq: u64 = BitBoard::index_square(king_sq) as u64;
        // let c0: u64      = BitBoard::index_square(c0) as u64;

        let mut out = king_sq * 63 * 5 * 2;

        let pc1 = if friendly {
            pc.index()
        } else {
            pc.index() + 5
        };

        let c1 = c0 as usize * 10 + pc1;
        (out as usize) + c1
    }

    // pub fn index(king_sq: Coord, pc: Piece, c0: Coord, side: Color) -> usize {
    //     assert!(pc != King);
    //     let king_sq: u64 = BitBoard::index_square(king_sq) as u64;
    //     let c0: u64      = BitBoard::index_square(c0) as u64;
    //     let mut out = king_sq * (64 * 5 * 2);
    //     let pc1 = if side == White {
    //         pc.index()
    //     } else {
    //         pc.index() + 5
    //     };
    //     let c1 = c0 as usize * 10 + pc1;
    //     (out as usize) + c1
    // }

}

