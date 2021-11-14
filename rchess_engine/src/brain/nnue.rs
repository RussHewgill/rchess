
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

/// Increment Activations
impl NNUE {
    fn increment_act_own(&mut self, idx: usize, add: bool) {
        let mut c: nd::ArrayViewMut1<i8> = self.activations_own.slice_mut(nd::s![.., 0]);
        let d: nd::ArrayView1<i16>       = self.weights_1.slice(nd::s![.., idx]);
        if add {
            c += &d.map(|x| *x as i8);
        } else {
            c -= &d.map(|x| *x as i8);
        }
    }

    fn increment_act_other(&mut self, idx: usize, add: bool) {
        let mut c: nd::ArrayViewMut1<i8> = self.activations_other.slice_mut(nd::s![.., 0]);
        let d: nd::ArrayView1<i16>       = self.weights_1.slice(nd::s![.., idx]);
        if add {
            c += &d.map(|x| *x as i8);
        } else {
            c -= &d.map(|x| *x as i8);
        }
    }
}

/// Updates
impl NNUE {

    pub fn update_insert_piece<T: Into<u8> + Copy>(
        // &mut self, king_sq_own: u8, king_sq_other: u8, pc: Piece, c0: T, friendly: bool) {
        &mut self, king_sq_own: u8, king_sq_other: u8, pc: Piece, c0: T, trace: bool) {
        let c0 = c0.into();
        let (idx0,idx1) = Self::index2(king_sq_own, king_sq_other, pc, c0);

        self.inputs_own[(idx0,0)]   = 1;
        self.increment_act_own(idx0, true);

        self.inputs_other[(idx1,0)] = 1;
        self.increment_act_other(idx1, true);

        if trace {
            trace!("idx0, idx1 = {:?}, {:?}", idx0, idx1);
            trace!("Updating own king at   {:?} with {:?} at {:?} to 1",
                Coord::from(king_sq_own),
                // if friendly { "friendly" } else { "enemy" }, pc, Coord::from(c0));
                pc, Coord::from(c0));
            trace!("Updating other king at {:?} with {:?} at {:?} to 1",
                Coord::from(king_sq_other),
                pc, Coord::from(c0));
        }

    }

    pub fn update_delete_piece<T: Into<u8> + Copy>(
        // &mut self, king_sq_own: u8, king_sq_other: u8, pc: Piece, c0: T, friendly: bool) {
        &mut self, king_sq_own: u8, king_sq_other: u8, pc: Piece, c0: T, trace: bool) {
        let c0 = c0.into();
        let (idx0,idx1) = Self::index2(king_sq_own, king_sq_other, pc, c0);

        self.inputs_own[(idx0,0)]   = 0;
        self.increment_act_own(idx0, false);

        self.inputs_other[(idx1,0)] = 0;
        self.increment_act_other(idx1, false);

        if trace {
            trace!("idx0, idx1 = {:?}, {:?}", idx0, idx1);
            trace!("Updating own king at   {:?} with {:?} at {:?} to 0",
                Coord::from(king_sq_own),
                // if friendly { "friendly" } else { "enemy" }, pc, Coord::from(c0));
                pc, Coord::from(c0));
            trace!("Updating other king at {:?} with {:?} at {:?} to 0",
                Coord::from(king_sq_other),
                // if !friendly { "friendly" } else { "enemy" }, pc, Coord::from(c0));
                pc, Coord::from(c0));
        }

    }

    pub fn update_move_piece<T: Into<u8> + Copy>(
        // &mut self, king_sq_own: u8, king_sq_other: u8, pc: Piece, from: T, to: T, friendly: bool) {
        &mut self, king_sq_own: u8, king_sq_other: u8, pc: Piece, from: T, to: T) {
        // self.update_delete_piece(king_sq_own, king_sq_other, pc, from.into(), friendly);
        // self.update_insert_piece(king_sq_own, king_sq_other, pc, to.into(), friendly);
        self.update_delete_piece(king_sq_own, king_sq_other, pc, from.into(), true);
        self.update_insert_piece(king_sq_own, king_sq_other, pc, to.into(), true);
    }

    fn update_en_passant<T: Into<Coord> + Copy>(&mut self, c0: Option<T>) {
        if let Some(ep) = self.en_passant {
            let idx = Self::index_en_passant(ep).unwrap();
            self.increment_act_own(idx, false);
        }
        if let Some(ep) = c0 {
            let idx = Self::index_en_passant(ep.into()).unwrap();
            self.increment_act_own(idx, true);
        }
    }

    /// Called AFTER game has had move applied
    pub fn update_move(&mut self, g: &Game) -> i8 {
        let mv = match g.last_move {
            None => {
                debug!("No previous move, running fresh");
                return self.run_fresh(&g);
                // unimplemented!()
            },
            Some(mv) => mv,
        };

        if self.dirty {
            debug!("dirty, running fresh");
            return self.run_fresh(&g);
            // unimplemented!()
        }

        if mv.piece() == Some(King) {
            trace!("king move, running fresh");
            return self.run_fresh(&g);
            // unimplemented!()
        }

        self._update_move(&g, mv);

        self.run_partial()

    }

    pub fn _update_move(&mut self, g: &Game, mv: Move) {

        // // XXX: reversed, because g already had move applied
        // let king_sq_own   = g.get(King, !g.state.side_to_move).bitscan();
        // let king_sq_other = g.get(King, g.state.side_to_move).bitscan();

        let king_sq_own   = g.get(King, self.side).bitscan();
        let king_sq_other = g.get(King, !self.side).bitscan();

        match mv {
            Move::Quiet { from, to, pc } => {
                self.update_move_piece(king_sq_own,king_sq_other, pc, from, to);
            },
            Move::PawnDouble { from, to } => {
                self.update_move_piece(king_sq_own,king_sq_other, Pawn, from, to);
            },
            Move::Capture { from, to, pc, victim } => {
                self.update_move_piece(king_sq_own,king_sq_other, pc, from, to);
                self.update_delete_piece(king_sq_own,king_sq_other, pc, to, true);
            },
            Move::EnPassant { from, to, capture } => {
                self.update_move_piece(king_sq_own,king_sq_other, Pawn, from, to);
                self.update_delete_piece(king_sq_own,king_sq_other, Pawn, capture, true);
            },
            Move::Castle { from, to, rook_from, rook_to } => {
                // return self.run_fresh(&g);
                unimplemented!()
            },
            Move::Promotion { from, to, new_piece } => {
                self.update_delete_piece(king_sq_own,king_sq_other, Pawn, from, true);
                self.update_insert_piece(king_sq_own,king_sq_other, new_piece, to, true);
            },
            Move::PromotionCapture { from, to, new_piece, victim } => {
                self.update_delete_piece(king_sq_own,king_sq_other, Pawn, from, true);
                self.update_insert_piece(king_sq_own,king_sq_other, new_piece, to, true);
                self.update_delete_piece(king_sq_own,king_sq_other, victim, to, true);
            },
            Move::NullMove => {},
        }
    }

}

/// Run
impl NNUE {

    pub fn run_partial(&self) -> i8 {

        let act1: Array2<i8> = nd::concatenate![
            nd::Axis(0), self.activations_own.clone(), self.activations_other.clone()
        ];

        let z2 = self.weights_2.dot(&act1);
        let act2 = z2.map(Self::relu);

        let z3 = self.weights_3.dot(&act2);
        let act3 = z3.map(Self::relu);

        let z_out = self.weights_4.dot(&act3);
        let act_out = z_out.map(Self::relu);

        act_out[(0,0)]
    }

    pub fn run_fresh(&mut self, g: &Game) -> i8 {

        self.init_inputs(g);

        let z0_own: Array2<i16>   = self.weights_1.dot(&self.inputs_own);
        let z0_other: Array2<i16> = self.weights_1.dot(&self.inputs_other);

        let z0_own   = z0_own.map(|x| *x as i8);
        let z0_other = z0_other.map(|x| *x as i8);

        self.activations_own   = z0_own;
        self.activations_other = z0_other;
        // self.activations_own   = z0_own.clone();
        // self.activations_other = z0_other.clone();

        self.run_partial()

        // let act1 = nd::concatenate![nd::Axis(0), z0_own, z0_other];

        // let z2 = self.weights_2.dot(&act1);
        // let act2 = z2.map(Self::relu);
        // // let act2 = z2.map(|x| sigmoid(*x));

        // let z3 = self.weights_3.dot(&act2);
        // let act3 = z3.map(Self::relu);
        // // let act3 = z3.map(|x| sigmoid(*x));

        // let z_out = self.weights_4.dot(&act3);
        // let act_out = z_out.map(Self::relu);
        // // let act_out = z_out.map(|x| sigmoid(*x));

        // act_out[(0,0)]
    }

    pub fn relu(x: &i8) -> i8 {
        *x.max(&0)
    }

}

/// Init, Indexing
impl NNUE {

    /// Reset inputs and activations, and refill from board
    pub fn init_inputs(&mut self, g: &Game) {
        const COLORS: [Color; 2] = [White,Black];
        const PCS: [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];

        self.inputs_own.fill(0);
        self.inputs_other.fill(0);

        self.activations_own.fill(0);
        self.activations_other.fill(0);

        self.dirty = false;

        let king_sq_own   = g.get(King, self.side).bitscan();
        let king_sq_other = g.get(King, !self.side).bitscan();

        for pc in PCS {
            g.get(pc, self.side).into_iter().for_each(|sq| {

                trace!("Setting own king at {:?} with {:?} at {:?}",
                       Coord::from(king_sq_own), pc, Coord::from(sq));

                self.update_insert_piece(king_sq_own, king_sq_other, pc, sq, false);
            });
            g.get(pc, !self.side).into_iter().for_each(|sq| {

                trace!("Setting other king at {:?} with {:?} at {:?}",
                       Coord::from(king_sq_other), pc, Coord::from(sq));

                self.update_insert_piece(king_sq_other, king_sq_own, pc, sq, false);
            });
        }

    }

    /// Reset inputs and activations, and refill from board
    pub fn init_inputs2(&mut self, g: &Game) {
        const COLORS: [Color; 2] = [White,Black];
        const PCS: [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];

        self.inputs_own.fill(0);
        self.inputs_other.fill(0);

        self.activations_own.fill(0);
        self.activations_other.fill(0);

        self.dirty = false;

        for side in COLORS {
            let mut indices_f = vec![];
            // let mut indices_e = vec![];

            // let king_sq: Coord   = g.get(King, side).bitscan().into();
            let king_sq_own   = g.get(King, side).bitscan();
            let king_sq_other = g.get(King, !side).bitscan();

            // let friendly = side == g.state.side_to_move;
            let friendly = side == self.side;

            for pc in PCS {

            }

            // for pc in PCS {
            //     g.get(pc, !side).into_iter().for_each(|sq| {
            //         trace!("Setting other {:?} king at {:?} with f({}) {:?} at {:?}",
            //                side, Coord::from(king_sq_own), friendly, pc, Coord::from(sq));
            //         let idx = Self::index(king_sq_other, pc, sq, friendly);
            //         indices_e.push(idx);
            //     });
            //     g.get(pc, side).into_iter().for_each(|sq| {
            //         // if side0 == White && side == side0 {}
            //         trace!("Setting own {:?} king at {:?} with f({}) {:?} at {:?}",
            //                  side, Coord::from(king_sq_own), friendly, pc, Coord::from(sq));
            //         let idx = Self::index(king_sq_own, pc, sq, friendly);
            //         indices_f.push(idx);
            //     });
            // }

            if let Some(c_ep) = g.state.en_passant {
                if let Some(ep) = Self::index_en_passant(c_ep) {
                    indices_f.push(ep);
                }
            }

            // TODO: 
            let cs = Self::index_castling(&g.state.castling);
            for i in cs.into_iter() {
                indices_f.push(i);
            }

            for i in indices_f.into_iter() {
                if friendly {
                    self.inputs_own[(i,0)] = 1;
                } else {
                    self.inputs_other[(i,0)] = 1;
                }
            }

            // for i in indices_e.into_iter() {
            //     if !friendly {
            //         self.inputs_own[(i,0)] = 1;
            //     } else {
            //         self.inputs_other[(i,0)] = 1;
            //     }
            // }

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

        // eprintln!("TODO: castling");
        // unimplemented!()
        vec![]
    }

    pub fn index2(king_sq_own: u8, king_sq_other: u8, pc: Piece, c0: u8) -> (usize,usize) {
        let i0 = Self::index(king_sq_own, pc, c0, true);
        let i1 = Self::index(king_sq_other, pc, c0, false);
        (i0,i1)
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

    // let mut xs: HashSet<usize> = HashSet::default();
    // for f in [true,false] {
    //     for king_sq in 0..64u8 {
    //         for c0 in 0..63u8 {
    //             for pc in Piece::iter_nonking_pieces() {
    //                 let idx = NNUE::index(king_sq, pc, c0, f);
    //                 if xs.contains(&idx) {
    //                     panic!()
    //                 }
    //                 xs.insert(idx);
    //             }
    //         }
    //     }
    // }
    // let k = xs.len();
    // eprintln!("k = {:?}", k);
    // let m0 = xs.iter().min();
    // let m1 = xs.iter().max();
    // eprintln!("max,min = ({:?},{:?})", m0,m1);

    // // pub fn index(king_sq: Coord, pc: Piece, c0: Coord, friendly: bool) -> usize {
    // pub fn index(king_sq: u8, pc: Piece, c0: u8, friendly: bool) -> usize {
    //     // let king_sq: u64 = BitBoard::index_square(king_sq) as u64;
    //     // let c0: u64      = BitBoard::index_square(c0) as u64;
    //     let mut out = king_sq * 63 * 5 * 2;
    //     let pc1 = if friendly {
    //         pc.index()
    //     } else {
    //         pc.index() + 5
    //     };
    //     let c1 = c0 as usize * 10 + pc1;
    //     (out as usize) + c1
    // }

}

