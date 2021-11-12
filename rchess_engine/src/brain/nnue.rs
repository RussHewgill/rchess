
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
use crate::brain::types::*;
use crate::brain::types::nnue::*;

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

    pub fn update_move(&mut self, g: &Game, mv: Move) {

        let king_sq_own: Coord   = g.get(King, g.state.side_to_move).bitscan().into();
        let king_sq_other: Coord = g.get(King, !g.state.side_to_move).bitscan().into();

        match mv.piece() {
            Some(King) => {
                unimplemented!()
            },
            Some(pc) => {

                // let idx = Self::index(king_sq, pc, c0, side)

                unimplemented!()
            },
            None => {
            },
        }

    }

    pub fn run_fresh(&mut self, g: &Game) -> f32 {

        // XXX: 
        // self.init_inputs(g);

        // let mut last = &self.inputs_own;

        let z0_own = self.weights_in_own.dot(&self.inputs_own);
        let z0_other = self.weights_in_other.dot(&self.inputs_other);

        let act1 = nd::concatenate![nd::Axis(0), z0_own, z0_other];

        let z2 = self.weights_l2.dot(&act1);
        let act2 = z2.map(Self::relu);

        let z3 = self.weights_l3.dot(&act2);
        let act3 = z3.map(Self::relu);

        let z_out = self.weights_out.dot(&act3);
        let act_out = z_out.map(Self::relu);

        let s0 = z0_own.shape();
        let s1 = act1.shape();
        let s2 = z2.shape();
        let s3 = z3.shape();
        let s4 = z_out.shape();

        // eprintln!("s0 = {:?}", s0);
        // eprintln!("s1 = {:?}", s1);
        // eprintln!("s2 = {:?}", s2);
        // eprintln!("s3 = {:?}", s3);
        // eprintln!("s4 = {:?}", s4);

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

        unimplemented!()
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

