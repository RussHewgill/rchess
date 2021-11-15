
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
use crate::brain::types::*;
use crate::brain::types::nnue::*;
use crate::brain::matrix::*;

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
        let d: nd::ArrayView1<i16>       = self.weights_1_own.slice(nd::s![.., idx]);
        if add {
            c += &d.map(|x| *x as i8);
        } else {
            c -= &d.map(|x| *x as i8);
        }
    }

    fn increment_act_other(&mut self, idx: usize, add: bool) {
        let mut c: nd::ArrayViewMut1<i8> = self.activations_other.slice_mut(nd::s![.., 0]);
        let d: nd::ArrayView1<i16>       = self.weights_1_other.slice(nd::s![.., idx]);
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
    pub fn update_move(&mut self, g: &Game) -> i32 {
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

        let s = if g.state.side_to_move != self.side { self.side } else { !self.side };

        let king_sq_own   = g.get(King, s).bitscan();
        let king_sq_other = g.get(King, !s).bitscan();

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

    // pub fn _run_partial(&self) -> i32 {
    pub fn _run_partial(&self)
                        -> (i32,(Array2<i8>,Array2<i8>,Array2<i8>),(Array2<i8>,Array2<i32>,Array2<i32>)) {
        println!("_run_partial");

        let mut z1: Array2<i8> = nd::concatenate![
            nd::Axis(0), self.activations_own.clone(), self.activations_other.clone()
        ];
        let bs = nd::concatenate![nd::Axis(0), self.biases_1_own, self.biases_1_other];
        z1 += &bs;
        // let act1 = z1.map(Self::relu); // 512,1
        let act1 = z1.map(|x| (*x).clamp(0, 127) as i8); // 512,1

        let mut z2 = self.weights_2.map(|x| *x as i32).dot(&act1.map(|x| *x as i32)); // 32,1
        z2 += &self.biases_2;
        let act2 = z2.map(|x| (*x / 64).clamp(0, 127) as i8);

        let mut z3 = self.weights_3.map(|x| *x as i32).dot(&act2.map(|x| *x as i32)); // 32,1
        z3 += &self.biases_3;
        let act3 = z3.map(|x| (*x / 64).clamp(0, 127) as i8);

        let mut z_out = self.weights_4.map(|x| *x as i32).dot(&act3.map(|x| *x as i32)); // 
        z_out += &self.biases_4;
        // let act_out = z_out.map(|x| (*x / 64).clamp(0, 127) as i8);

        let pred = z_out[(0,0)];

        (pred, (act1,act2,act3), (z1, z2, z3))
        // unimplemented!()
    }

    pub fn run_partial(&self) -> i32 {
        let (out,_,_) = self._run_partial();
        out
    }

    pub fn run_fresh(&mut self, g: &Game) -> i32 {

        self.init_inputs(g);

        let z0_own: Array2<i16>   = self.weights_1_own.dot(&self.inputs_own);
        let z0_other: Array2<i16> = self.weights_1_other.dot(&self.inputs_other);

        let z0_own   = z0_own.map(|x| *x as i8);
        let z0_other = z0_other.map(|x| *x as i8);

        self.activations_own   = z0_own;
        self.activations_other = z0_other;

        self.run_partial()
    }

    pub fn cost_fn_cross_ent(correct: i8, pred: i8) -> i8 {
        // np.sum(np.nan_to_num(-y*np.log(a)-(1-y)*np.log(1-a)))
        let correct = ((correct as f32) + 127.) / 255.;
        let pred    = ((pred as f32) + 127.) / 255.;
        // let out = -correct * f32::ln(pred) - (1. - correct) * f32::ln(1.0 - correct);
        let x0 = -correct * pred.ln();
        let x1 = 1.0 - correct;
        let x2 = (1.0 - correct).ln();
        let out = x0 - x1 * x2;
        if out.is_nan() {
            panic!("NaN cost fn?");
        }
        (out * 255. - 127.) as i8
    }

    pub fn cost_fn(correct: i8, pred: i8) -> i8 {
        unimplemented!()
    }

    pub fn cost_delta(correct: i8, pred: i8) -> i8 {
        pred.checked_sub(correct).unwrap()
    }

    pub fn relu<T: num_traits::PrimInt, V: num_traits::PrimInt>(x: &T) -> V {
        V::from((*x).clamp(T::from(0).unwrap(), T::from(127).unwrap())).unwrap()
    }

    // pub fn relu(x: &i8) -> i8 {
    //     *x.max(&0)
    // }

    // pub fn relu_d(x: &i8) -> i8 {
    pub fn relu_d<T: num_traits::PrimInt, V: num_traits::PrimInt>(x: &T) -> V {
        if *x < T::zero() {
            V::zero()
        } else if *x > T::zero() {
            V::from(1).unwrap()
        } else {
            V::zero()
        }
    }

    pub fn cp_to_wdl(x: i8) -> f32 {
        ((x as f32) + 127.) / 255.
    }

    pub fn wdl_to_cp(x: f32) -> i8 {
        (x * 255. - 127.) as i8
    }

}

/// Backprop
impl NNUE {

    pub fn run_fresh2(&mut self, g: &Game) -> f32 {

        self.init_inputs(g);

        let z0_own: Array2<i16>   = self.weights_1_own.dot(&self.inputs_own);
        let z0_other: Array2<i16> = self.weights_1_other.dot(&self.inputs_other);

        let z0_own   = z0_own.map(|x| *x as i8);
        let z0_other = z0_other.map(|x| *x as i8);

        self.activations_own   = z0_own;
        self.activations_other = z0_other;

        let mut z1: Array2<i8> = nd::concatenate![
            nd::Axis(0), self.activations_own.clone(), self.activations_other.clone()
        ];
        let bs = nd::concatenate![
            nd::Axis(0), self.biases_1_own, self.biases_1_other
        ];
        z1 += &bs;
        // z1 += &self.biases_1;
        let act1 = z1.map(Self::relu);

        let act1: nd::Array1<i8> = act1.slice(nd::s![.., 0]).to_owned();
        // let act1: DVector<f32> = act1.into_nalgebra().map(|x| (x as f32) / 127.0);
        let act1: DVector<f32> = act1.into_nalgebra().map(|x| (x as f32) / 127.0);

        // println!("act1 = {}", act1);

        let out = self.test_nn.run(&act1)[0];

        out
    }

    pub fn backprop2(&mut self, correct: i8, eta: i8) {

        let mut z1: Array2<i8> = nd::concatenate![
            nd::Axis(0), self.activations_own.clone(), self.activations_other.clone()
        ];
        let bs = nd::concatenate![
            nd::Axis(0), self.biases_1_own, self.biases_1_other
        ];
        z1 += &bs;
        // z1 += &self.biases_1;
        let act1 = z1.map(Self::relu);

        let act1: nd::Array1<i8> = act1.slice(nd::s![.., 0]).to_owned();
        let act1: DVector<f32> = act1.into_nalgebra().map(|x| (x as f32) / 127.0);

        let ins: Vec<(DVector<f32>, DVector<f32>)> = vec![
            (act1,na::dvector![(correct as f32) / 127.0]),
        ];

        self.test_nn.backprop_mut_matrix(&ins, 0.1);

        // let mut z2 = self.weights_2.dot(&act1);
        // z2 += &self.biases_2;
        // let act2 = z2.map(Self::relu);

        // let mut z3 = self.weights_3.dot(&act2);
        // z3 += &self.biases_3;
        // let act3 = z3.map(Self::relu);

        // let z_out = self.weights_4.dot(&act3);
        // let act_out = z_out.map(Self::relu);
        // let pred = act_out[(0,0)];

    }

    #[allow(unused_doc_comments)]
    pub fn backprop(&mut self, g: Option<&Game>, correct: i8, eta: i8) {

        if let Some(g) = g {
            self.init_inputs(&g);

            let z0_own: Array2<i16>   = self.weights_1_own.dot(&self.inputs_own);
            let z0_other: Array2<i16> = self.weights_1_other.dot(&self.inputs_other);

            let z0_own   = z0_own.map(|x| *x as i8);
            let z0_other = z0_other.map(|x| *x as i8);

            self.activations_own   = z0_own;
            self.activations_other = z0_other;

        }

        /// (i32,
        ///   (Array2<i8>,Array2<i8>,Array2<i8>),
        ///   (Array2<i8>,Array2<i32>,Array2<i32>))
        let (pred, (act1,act2,act3), (z1, z2, z3)) = self._run_partial();

        /// L4
        // // let delta = pred - correct;
        let delta = pred.checked_sub(correct as i32).unwrap();
        // let delta = Self::cost_delta(correct, pred);
        eprintln!("pred  = {:?}", pred);
        eprintln!("delta = {:?}", delta);
        // let delta: Array2<i8> = delta * z_out.map(Self::relu_d); // 1,1
        let delta: Array2<i8> = nd::array![[delta.clamp(-127,127) as i8]];

        let ws4_n = &delta.dot(&act3.t()); // 1,32
        let bs4_n = delta.clone();
        // eprintln!("ws4_n.shape() = {:?}", ws4_n.shape());

        /// L3
        let sp3 = z3.map(Self::relu_d); // 32,1

        let mut d3: Array2<i8> = self.weights_4.t().dot(&delta); // 32,1
        d3 *= &sp3;

        let ws3_n = d3.dot(&act2.t()); // 32,32
        let bs3_n = d3.clone();        // 1,1
        // eprintln!("ws3_n.shape() = {:?}", ws3_n.shape());

        /// L2
        let sp2 = z2.map(Self::relu_d); // 32,1

        let mut d2 = self.weights_3.t().dot(&d3); // 32,1
        d2 *= &sp2;

        let ws2_n = d2.dot(&act1.t()); // 32,512
        let bs2_n = d2.clone();        // 32,1
        // eprintln!("ws2_n.shape() = {:?}", ws2_n.shape());

        // /// L1

        let sp: Array2<i8> = z1.map(Self::relu_d); // 512,1
        let sp_own   = sp.slice(nd::s![..256, ..]);
        let sp_other = sp.slice(nd::s![256.., ..]);

        let d1: Array2<i8> = self.weights_2.t().dot(&d2); // 512,1
        let mut d1_own   = d1.slice(nd::s![..256, ..]).to_owned();
        let mut d1_other = d1.slice(nd::s![256.., ..]).to_owned();
        d1_own   *= &sp_own;
        d1_other *= &sp_own;

        let ws1_n_own: Array2<i8>   = d1_own.dot(&self.inputs_own.t().map(|x| *x as i8));
        let ws1_n_other: Array2<i8> = d1_other.dot(&self.inputs_other.t().map(|x| *x as i8));

        let bs1_n_own   = d1_own;
        let bs1_n_other = d1_other;

        // println!("ws 1");
        self.weights_1_own   += &ws1_n_own.map(|x| (*x as i16) / eta as i16);
        self.weights_1_other += &ws1_n_other.map(|x| (*x as i16) / eta as i16);
        self.biases_1_own    += &bs1_n_own;
        self.biases_1_other  += &bs1_n_other;

        // println!("ws 2");
        self.weights_2 -= &(ws2_n / eta);
        // self.weights_2 -= &ws2_n;
        self.biases_2  -= &bs2_n.map(|x| *x as i32);

        // println!("ws 3");
        self.weights_3 -= &(ws3_n / eta);
        // self.weights_3 -= &ws3_n;
        self.biases_3  -= &bs3_n.map(|x| *x as i32);

        // println!("ws 4");
        self.weights_4 -= &(ws4_n / eta);
        // self.weights_4 -= ws4_n;
        self.biases_4  -= &bs4_n.map(|x| *x as i32);

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

    pub fn _index<T: Into<Coord>>(king_sq: T, pc: Piece, c0: T, friendly: bool) -> usize {
        let king_sq: Coord = king_sq.into();
        let c0: Coord = c0.into();
        Self::index(king_sq.into(), pc, c0.into(), friendly)
    }

    pub fn index2(king_sq_own: u8, king_sq_other: u8, pc: Piece, c0: u8) -> (usize,usize) {
        let i0 = Self::index(king_sq_own, pc, c0, true);
        let i1 = Self::index(king_sq_other, pc, c0, false);
        (i0,i1)
    }

    pub fn rev_index(idx0: usize) -> Option<(Coord, Piece, Coord, bool)> {
        for f in [true,false] {
            for king_sq in 0..64u8 {
                for c0 in 0..63u8 {
                    for pc in Piece::iter_nonking_pieces() {
                        let idx = NNUE::index(king_sq, pc, c0, f);
                        if idx == idx0 {
                            // eprintln!("wot = {:?}, {:?}, {:?}", Coord::from(king_sq), pc, Coord::from(c0));
                            return Some((Coord::from(king_sq), pc, Coord::from(c0), f));
                        }
                    }
                }
            }
        }
        None
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

