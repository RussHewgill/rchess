
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
use nd::{Array2};

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

    fn update_insert_piece<T: Into<u8> + Copy>(&mut self, king_sq: u8, pc: Piece, c0: T, friendly: bool) {
        let c0 = c0.into();
        let idx0 = Self::index(king_sq, pc, c0, friendly);
        let idx1 = Self::index(king_sq, pc, c0, !friendly);
        self.inputs_own[(idx0,0)]   = 1;
        self.inputs_other[(idx1,0)] = 1;

        let mut c: nd::ArrayViewMut1<i8> = self.activations1_own.slice_mut(nd::s![.., 0]);
        let d: nd::ArrayView1<i16>       = self.weights_in_own.slice(nd::s![.., idx0]);
        c += &d.map(|x| *x as i8);

        let mut c: nd::ArrayViewMut1<i8> = self.activations1_other.slice_mut(nd::s![.., 0]);
        let d: nd::ArrayView1<i16>       = self.weights_in_other.slice(nd::s![.., idx1]);
        c += &d.map(|x| *x as i8);

        println!("Setting own king at {:?} with f({}) {:?} at {:?} to 1",
                 Coord::from(king_sq), friendly, pc, Coord::from(c0));

    }

    fn update_delete_piece<T: Into<u8> + Copy>(&mut self, king_sq: u8, pc: Piece, c0: T, friendly: bool) {
        let c0 = c0.into();
        let idx0 = Self::index(king_sq, pc, c0, friendly);
        let idx1 = Self::index(king_sq, pc, c0, !friendly);
        self.inputs_own[(idx0,0)]   = 0;
        self.inputs_other[(idx1,0)] = 0;

        let mut c: nd::ArrayViewMut1<i8> = self.activations1_own.slice_mut(nd::s![.., 0]);
        let d: nd::ArrayView1<i16>       = self.weights_in_own.slice(nd::s![.., idx0]);
        c -= &d.map(|x| *x as i8);

        let mut c: nd::ArrayViewMut1<i8> = self.activations1_other.slice_mut(nd::s![.., 0]);
        let d: nd::ArrayView1<i16>       = self.weights_in_other.slice(nd::s![.., idx1]);
        c -= &d.map(|x| *x as i8);

        println!("Setting own king at {:?} with f({}) {:?} at {:?} to 0",
                 Coord::from(king_sq), friendly, pc, Coord::from(c0));

    }

    fn update_move_piece<T: Into<u8> + Copy>(
        &mut self, king_sq: u8, pc: Piece, from: T, to: T, friendly: bool) {
        self.update_delete_piece(king_sq.into(), pc, from.into(), friendly);
        self.update_insert_piece(king_sq.into(), pc, to.into(), friendly);
    }

    /// Called AFTER game has had move applied
    pub fn update_move(&mut self, g: &Game, side: Color) {
        let mv = match g.last_move {
            None => {
                self.run_fresh(&g, side);
                return;
            },
            Some(mv) => mv,
        };

        // XXX: reversed, because g already had move applied
        let king_sq_own   = g.get(King, !g.state.side_to_move).bitscan();
        let king_sq_other = g.get(King, g.state.side_to_move).bitscan();

        if mv.piece() == Some(King) {
            unimplemented!()
        }

        match mv {
            Move::Quiet { from, to, pc } => {
                // XXX: friendly = false?
                self.update_move_piece(king_sq_own, pc, from, to, true);
            },
            Move::PawnDouble { from, to } => {
                self.update_move_piece(king_sq_own, Pawn, from, to, true);
            }
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

    pub fn run_fresh(&mut self, g: &Game, side: Color) -> i8 {

        self.init_inputs(g, side);

        let z0_own: Array2<i16>   = self.weights_in_own.dot(&self.inputs_own);
        let z0_other: Array2<i16> = self.weights_in_other.dot(&self.inputs_other);

        let z0_own   = z0_own.map(|x| *x as i8);
        let z0_other = z0_other.map(|x| *x as i8);

        self.activations1_own   = z0_own.clone();
        self.activations1_other = z0_other.clone();

        let act1 = nd::concatenate![nd::Axis(0), z0_own, z0_other];

        // eprintln!("z0_own.shape() = {:?}", z0_own.shape());
        // eprintln!("act1.shape() = {:?}", act1.shape());

        let z2 = self.weights_l2.dot(&act1);
        let act2 = z2.map(Self::relu);
        // let act2 = z2.map(|x| sigmoid(*x));

        let z3 = self.weights_l3.dot(&act2);
        let act3 = z3.map(Self::relu);
        // let act3 = z3.map(|x| sigmoid(*x));

        let z_out = self.weights_out.dot(&act3);
        let act_out = z_out.map(Self::relu);
        // let act_out = z_out.map(|x| sigmoid(*x));

        // let s0 = z0_own.shape();
        // let s1 = act1.shape();
        // let s2 = z2.shape();
        // let s3 = z3.shape();
        // let s4 = z_out.shape();

        act_out[(0,0)]
        // unimplemented!()
    }

    pub fn relu(x: &i8) -> i8 {
        *x.max(&0)
    }

}

impl NNUE {

    pub fn init_inputs(&mut self, g: &Game, side0: Color) {
        const COLORS: [Color; 2] = [White,Black];
        const PCS: [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];

        self.inputs_own.fill(0);
        self.inputs_other.fill(0);

        for side in COLORS {
            let mut indices = vec![];

            // let king_sq: Coord   = g.get(King, side).bitscan().into();
            let king_sq = g.get(King, side).bitscan();

            // let friendly = side == g.state.side_to_move;
            let friendly = side == side0;

            for pc in PCS {
                g.get(pc, side).into_iter().for_each(|sq| {

                    // if side0 == White && side == side0 {
                    //     println!("Setting {:?} king at {:?} with f({}) {:?} at {:?}",
                    //              side, Coord::from(king_sq), friendly, pc, Coord::from(sq));
                    // }

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
                if friendly {
                    self.inputs_own[(i,0)] = 1;
                } else {
                    self.inputs_other[(i,0)] = 1;
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

