
use std::path::Path;

use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
use crate::brain::types::*;
use crate::brain::types::nnue::*;
use crate::brain::matrix::*;

use nd::ArrayViewMut2;
use ndarray as nd;
use nd::{Array2};

use num_traits::PrimInt;
use sprs::{CsMat,CsMatViewMut};

/// Increment
impl NNUE {
    fn _increment_act(&mut self, idx: usize, add: bool, own: bool) {
        let mut c: nd::ArrayViewMut1<i16> = if own {
            self.activations_own.slice_mut(nd::s![.., 0])
        } else {
            self.activations_other.slice_mut(nd::s![.., 0])
        };
        let dw: nd::ArrayView1<i8> = if own {
            self.weights_1_own.slice(nd::s![.., idx])
        } else {
            self.weights_1_other.slice(nd::s![.., idx])
        };

        if add {
            trace!("increment: adding {} idx {:?}", if own { "own" } else { "other" }, idx);
            c += &dw.map(|x| *x as i16);
        } else {
            trace!("increment: removing {} idx {:?}", if own { "own" } else { "other" }, idx);
            c -= &dw.map(|x| *x as i16);
        }
    }

    fn increment_act_own(&mut self, idx: usize, add: bool) {
        self._increment_act(idx, add, true)
    }

    fn increment_act_other(&mut self, idx: usize, add: bool) {
        self._increment_act(idx, add, false)
    }

}

/// Apply Move
impl NNUE {

    pub fn update_insert_piece(
        &mut self, king_sq_own: Coord, king_sq_other: Coord, pc: Piece, sq: Coord, side: Color,
    ) {
        // trace!("inserting (k {:?}) {:?} {:?} at {:?}",
        //        Coord::from(king_sq_own), side, pc, Coord::from(sq));
        let idx0 = self.index_halfkp(king_sq_own, pc, side, sq.into());
        let idx1 = self.index_halfkp(king_sq_other, pc, side, sq.into());
        self.increment_act_own(idx0, true);
        self.increment_act_other(idx1, true);
    }

    pub fn update_delete_piece(
        &mut self, king_sq_own: Coord, king_sq_other: Coord, pc: Piece, sq: Coord, side: Color,
    ) {
        // trace!("removing (k {:?}) {:?} {:?} at {:?}",
        //        Coord::from(king_sq_own), side, pc, Coord::from(sq));
        let idx0 = self.index_halfkp(king_sq_own, pc, side, sq.into());
        let idx1 = self.index_halfkp(king_sq_other, pc, side, sq.into());
        self.increment_act_own(idx0, false);
        self.increment_act_other(idx1, false);
    }

    pub fn update_move_piece(
        &mut self, king_sq_own: Coord, king_sq_other: Coord, pc: Piece, from: Coord, to: Coord, side: Color,
    ) {
        self.update_delete_piece(king_sq_own, king_sq_other, pc, from, side);
        self.update_insert_piece(king_sq_own, king_sq_other, pc, to, side);
    }

    /// Called AFTER game has had move applied
    pub fn update_move(&mut self, g: &Game, run: bool) -> Option<i32> {
        let mv = match g.last_move {
            None => {
                debug!("No previous move, running fresh");
                return if run {
                    Some(self.run_fresh(&g))
                } else { None }
                // unimplemented!()
            },
            Some(mv) => mv,
        };

        // if self.dirty {
        //     debug!("dirty, running fresh");
        //     return self.run_fresh(&g);
        //     // unimplemented!()
        // }

        if mv.piece() == Some(King) {
            trace!("king move, running fresh");
            return if run {
                Some(self.run_fresh(&g))
            } else { None }
        }

        self._update_move(&g, mv);

        if run {
            Some(self.run_partial())
        } else {
            None
        }

    }

    pub fn _update_move(&mut self, g: &Game, mv: Move) {
        let s = if g.state.side_to_move != self.side { self.side } else { !self.side };
        let king_sq_own   = g.get(King, s).bitscan();
        let king_sq_other = g.get(King, !s).bitscan();

        let side = !g.state.side_to_move;
        match mv {
            Move::Quiet { from, to, pc } => {
                self.update_move_piece(king_sq_own,king_sq_other,pc,from,to,side);
            },
            Move::PawnDouble { from, to } => {
                self.update_move_piece(king_sq_own,king_sq_other, Pawn, from, to,side);
            },
            Move::Capture { from, to, pc, victim } => {
                self.update_move_piece(king_sq_own,king_sq_other, pc, from, to,side);
                self.update_delete_piece(king_sq_own,king_sq_other, pc, to, side);
            },
            Move::EnPassant { from, to, capture } => {
                self.update_move_piece(king_sq_own,king_sq_other, Pawn, from, to,side);
                self.update_delete_piece(king_sq_own,king_sq_other, Pawn, capture, side);
            },
            // Move::Castle { from, to, rook_from, rook_to } => {
            Move::Castle { .. } => {
                // return self.run_fresh(&g);
                unimplemented!()
            },
            Move::Promotion { from, to, new_piece } => {
                self.update_delete_piece(king_sq_own,king_sq_other, Pawn, from, side);
                self.update_insert_piece(king_sq_own,king_sq_other, new_piece, to, side);
            },
            Move::PromotionCapture { from, to, new_piece, victim } => {
                self.update_delete_piece(king_sq_own,king_sq_other, Pawn, from, side);
                self.update_insert_piece(king_sq_own,king_sq_other, new_piece, to, side);
                // self.update_delete_piece(king_sq_own,king_sq_other, victim, to, side);
                self.update_delete_piece(king_sq_own,king_sq_other, victim, to, !side); // XXX: ??
            },
            Move::NullMove => {},
        }
    }

}

pub type LayerArrays = ((Array2<i8>,Array2<i8>,Array2<i8>),(Array2<i16>,Array2<i32>,Array2<i32>,Array2<i32>));

/// Run
impl NNUE {

    pub fn _run_partial(&self) -> (i32,LayerArrays) {
        let mut z1: Array2<i16> = nd::concatenate![
            nd::Axis(0), self.activations_own.clone(), self.activations_other.clone()
        ];
        // let bs = nd::concatenate![nd::Axis(0), self.biases_1_own, self.biases_1_other];
        // z1 += &bs;
        // let act1 = z1.map(Self::relu); // 512,1
        // let act1 = z1.map(|x| (*x).clamp(0, 127) as i8); // 512,1
        let act1 = z1.map(Self::act_fn); // 512,1

        let mut z2 = self.weights_2.map(|x| *x as i32).dot(&act1.map(|x| *x as i32)); // 32,1
        z2 += &self.biases_2;
        // let act2 = z2.map(|x| (*x / 64).clamp(0, 127) as i8);
        let act2 = z2.map(|x| Self::act_fn(&(x / 64)));

        let mut z3 = self.weights_3.map(|x| *x as i32).dot(&act2.map(|x| *x as i32)); // 32,1
        z3 += &self.biases_3;
        // let act3 = z3.map(|x| (*x / 64).clamp(0, 127) as i8);
        let act3 = z3.map(|x| Self::act_fn(&(x / 64)));

        let mut z_out = self.weights_4.map(|x| *x as i32).dot(&act3.map(|x| *x as i32)); // 
        z_out += &self.biases_4;
        // let act_out = z_out.map(|x| (*x / 64).clamp(0, 127) as i8);

        // const SQRT_I32MAX: i32 = 46340;

        let pred = z_out[(0,0)];
        let pred = pred / NNUE_SCALE_OUTPUT;
        // let pred = pred.clamp(-SQRT_I32MAX, SQRT_I32MAX);

        (pred, ((act1,act2,act3), (z1, z2, z3, z_out)))
    }

    pub fn run_partial(&self) -> i32 {
        let (out,_) = self._run_partial();
        out
    }

    pub fn run_fresh(&mut self, g: &Game) -> i32 {
        self.init_inputs(g);
        self.run_partial()
    }
}

/// Utils
impl NNUE {

    pub fn cost_fn(pred: i32, correct: i32) -> i8 {
        unimplemented!()
    }

    pub fn cost_delta(z: i32, pred: i32, correct: i32) -> i8 {
        unimplemented!()
    }

    pub fn act_fn<T: PrimInt, V: PrimInt>(x: &T) -> V {
        Self::relu(x)
    }

    pub fn act_d<T: PrimInt, V: PrimInt>(x: &T) -> V {
    // pub fn act_d<T: PrimInt, V: PrimInt>(x: &T, e: V) -> V {
        Self::relu_d(x)
    }

    fn relu<T: num_traits::PrimInt, V: num_traits::PrimInt>(x: &T) -> V {
        V::from((*x).clamp(T::from(0).unwrap(), T::from(127).unwrap())).unwrap()
    }

    // fn relu_d<T: num_traits::PrimInt, V: num_traits::PrimInt>(x: &T, e: V) -> V {
    fn relu_d<T: num_traits::PrimInt, V: num_traits::PrimInt>(x: &T) -> V {
        if *x < T::zero() {
            V::zero()
        } else if *x > T::zero() {
            V::from(1).unwrap()
            // e
        } else {
            V::zero()
        }
    }

}

/// Init
impl NNUE {

    // pub fn init_inputs(&mut self, g: &Game) {
    //     self._init_inputs(g, None)
    // }

    // TODO: En Passant
    // TODO: Castling
    /// Reset inputs and activations, and refill from board
    pub fn init_inputs(
        &mut self,
        g: &Game,
        // inputs: Option<(ArrayViewMut2<i8>,ArrayViewMut2<i8>)>
        // inputs: Option<(CsMatViewMut<i8>,CsMatViewMut<i8>)>
    ) {
        const PCS: [Piece; 6] = [Pawn,Knight,Bishop,Rook,Queen,King];

        self.inputs_own.map_inplace(|_| 0);
        self.inputs_other.map_inplace(|_| 0);

        // self.activations_own.fill(0);
        // self.activations_other.fill(0);
        self.activations_own   = self.biases_1_own.clone();
        self.activations_other = self.biases_1_other.clone();

        let king_sq_own   = g.get(King, self.side).bitscan();
        let king_sq_other = g.get(King, !self.side).bitscan();

        // let (mut is0,mut is1) = (vec![],vec![]);
        // let ix = inputs.is_some();

        for pc in PCS {
            let side = self.side;
            if pc != King {
                g.get(pc, side).into_iter().for_each(|sq| {
                    trace!("Setting own king at {:?} with {:?} at {:?}",
                        Coord::from(king_sq_own), pc, Coord::from(sq));
                    let idx0 = self.index_halfkp(king_sq_own, pc, side, sq);
                    let idx1 = self.index_halfkp(king_sq_other, pc, side, sq);

                    self.increment_act_own(idx0, true);
                    self.increment_act_other(idx1, true);

                    self.inputs_own.insert(idx0,0, 1);
                    self.inputs_other.insert(idx1,0, 1);

                });
            }
            let side = !self.side;
            g.get(pc, side).into_iter().for_each(|sq| {
                trace!("Setting other king at {:?} with {:?} at {:?}",
                       Coord::from(king_sq_other), pc, Coord::from(sq));
                let idx0 = self.index_halfkp(king_sq_own, pc, side, sq);
                let idx1 = self.index_halfkp(king_sq_other, pc, side, sq);

                self.increment_act_own(idx0, true);
                self.increment_act_other(idx1, true);

                self.inputs_own.insert(idx0,0, 1);
                self.inputs_other.insert(idx1,0, 1);
            });

        }

        // TODO: En Passant

        // TODO: Castling

        // if let Some((mut in_own,mut in_other)) = inputs {
        //     // in_own.map_inplace(|_| 0);
        //     // in_other.map_inplace(|_| 0);
        //     in_own.fill(0);
        //     in_other.fill(0);
        //     for idx0 in is0.into_iter() {
        //         in_own[(idx0,0)]   = 1;
        //         // in_own.set(idx0,0, 1);
        //     }
        //     for idx1 in is1.into_iter() {
        //         in_other[(idx1,0)] = 1;
        //         // in_other.set(idx1,0, 1);
        //     }
        // }

    }

}

/// Indexing
impl NNUE {
    pub fn index_halfkp(&self, king_sq: Coord, pc: Piece, side: Color, sq: Coord) -> usize {
        let cc = if self.side == side { 1 } else { 0 };
        let pi = pc.index() * 2 + cc;
        sq.inner() as usize + (pi + king_sq.inner() as usize * 11) * 64
    }
}

/// Save, load
impl NNUE {

    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        use std::io::Write;
        let b: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut file = std::fs::File::create(path)?;
        file.write_all(&b)
    }

    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        use std::io::Write;
        let mut b = std::fs::read(path)?;
        let out: Self = bincode::deserialize(&b).unwrap();
        Ok(out)
    }

}
