
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
use crate::brain::types::*;
use crate::brain::types::nnue::*;
use crate::brain::accumulator::*;
use crate::brain::matrix::*;

use ndarray as nd;
use nd::{Array2};

use num_traits::PrimInt;

/// Increment
impl NNUE {
    fn _increment_act(&mut self, idx: usize, add: bool, own: bool) {
        let mut c: nd::ArrayViewMut1<i16> = if own {
            self.activations_own.slice_mut(nd::s![.., 0])
        } else {
            self.activations_other.slice_mut(nd::s![.., 0])
        };
        let dw: nd::ArrayView1<i8> = self.weights_1.slice(nd::s![.., idx]);
        // let db: nd::ArrayView1<i8> = self.biases_1.slice(nd::s![.., idx]);

        let own = if own { "own" } else { "other" };
        if add {
            trace!("increment: adding {} idx {:?}", own, idx);
            c += &dw.map(|x| *x as i16);
        } else {
            trace!("increment: removing {} idx {:?}", own, idx);
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

    // pub fn update_move(&mut self, g: &Game) -> i32 {
    //     unimplemented!()
    // }

    // pub fn _update_move(&mut self, g: &Game, mv: Move) {
    //     unimplemented!()
    // }

    pub fn update_insert_piece<T: Into<u8> + Copy>(
        &mut self, king_sq_own: u8, king_sq_other: u8, pc: Piece, sq: T, side: Color,
    ) {
        // trace!("inserting (k {:?}) {:?} {:?} at {:?}",
        //        Coord::from(king_sq_own), side, pc, Coord::from(sq));
        let idx0 = self.index_halfkp(king_sq_own, pc, side, sq.into());
        let idx1 = self.index_halfkp(king_sq_other, pc, side, sq.into());
        self.increment_act_own(idx0, true);
        self.increment_act_other(idx1, true);
    }

    pub fn update_delete_piece<T: Into<u8> + Copy>(
        &mut self, king_sq_own: u8, king_sq_other: u8, pc: Piece, sq: T, side: Color,
    ) {
        // trace!("removing (k {:?}) {:?} {:?} at {:?}",
        //        Coord::from(king_sq_own), side, pc, Coord::from(sq));
        let idx0 = self.index_halfkp(king_sq_own, pc, side, sq.into());
        let idx1 = self.index_halfkp(king_sq_other, pc, side, sq.into());
        self.increment_act_own(idx0, false);
        self.increment_act_other(idx1, false);
    }

    pub fn update_move_piece<T: Into<u8> + Copy>(
        &mut self, king_sq_own: u8, king_sq_other: u8, pc: Piece, from: T, to: T, side: Color,
    ) {
        self.update_delete_piece(king_sq_own, king_sq_other, pc, from, side);
        self.update_insert_piece(king_sq_own, king_sq_other, pc, to, side);
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

        // if self.dirty {
        //     debug!("dirty, running fresh");
        //     return self.run_fresh(&g);
        //     // unimplemented!()
        // }

        if mv.piece() == Some(King) {
            trace!("king move, running fresh");
            return self.run_fresh(&g);
        }

        self._update_move(&g, mv);

        self.run_partial()

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
            Move::Castle { from, to, rook_from, rook_to } => {
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
                self.update_delete_piece(king_sq_own,king_sq_other, victim, to, side);
            },
            Move::NullMove => {},
        }
    }

}

/// Run
impl NNUE {

    pub fn _run_partial(&self)
                        -> (i32,(Array2<i8>,Array2<i8>,Array2<i8>),(Array2<i16>,Array2<i32>,Array2<i32>)) {
        let mut z1: Array2<i16> = nd::concatenate![
            nd::Axis(0), self.activations_own.clone(), self.activations_other.clone()
        ];
        // let bs = nd::concatenate![nd::Axis(0), self.biases_1_own, self.biases_1_other];
        // z1 += &bs;
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
    }

    pub fn run_partial(&self) -> i32 {
        let (out,_,_) = self._run_partial();
        out
    }

    pub fn run_fresh(&mut self, g: &Game) -> i32 {
        self.init_inputs(g);
        self.run_partial()
    }
}

/// Backprop
impl NNUE {

    #[allow(unused_doc_comments)]
    pub fn backprop(&mut self, correct: i32, eta: i8) {

        let (pred, (act1,act2,act3), (z1, z2, z3)) = self._run_partial();

    }

}

/// Utils
impl NNUE {

    fn cost_fn(pred: i32, correct: i32) -> i8 {
        unimplemented!()
    }

    fn cost_delta(z: i32, pred: i32, correct: i32) -> i8 {
        unimplemented!()
    }

    fn act_fn<T: PrimInt, V: PrimInt>(x: &T) -> V {
        unimplemented!()
    }

    fn act_d<T: PrimInt, V: PrimInt>(x: &T) -> V {
        unimplemented!()
    }

}

/// Init
impl NNUE {

    // TODO: En Passant
    // TODO: Castling
    /// Reset inputs and activations, and refill from board
    pub fn init_inputs(&mut self, g: &Game) {

        // self.activations_own.fill(0);
        // self.activations_other.fill(0);
        self.activations_own   = self.biases_1.clone();
        self.activations_other = self.biases_1.clone();

        let king_sq_own   = g.get(King, self.side).bitscan();
        let king_sq_other = g.get(King, !self.side).bitscan();

        // const PCS: [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];
        const PCS: [Piece; 6] = [Pawn,Knight,Bishop,Rook,Queen,King];

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
            });
        }

        // TODO: En Passant

        // TODO: Castling

    }

}

/// Indexing
impl NNUE {
    pub fn index_halfkp(&self, king_sq: u8, pc: Piece, side: Color, sq: u8) -> usize {
        let cc = if self.side == side { 1 } else { 0 };
        let pi = pc.index() * 2 + cc;
        sq as usize + (pi + king_sq as usize * 11) * 64
    }
}

