
use crate::evaluate::Score;
use crate::types::*;
use crate::sf_compat::accumulator::*;
use crate::sf_compat::NNIndex;

use super::{HALF_DIMS, NNUE4};
use super::accumulator::NNAccum;

use std::io::{self, Read,BufReader, BufWriter};
use std::fs::File;
use std::path::Path;

use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};

// #[derive(Debug,PartialEq,Clone)]
#[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
pub struct NNFeatureTrans {
    pub biases:         Vec<i16>, // 1024

    // pub weights:        [i16; Self::DIMS_IN * HALF_DIMS], // stack overflows
    pub weights:        Vec<i16>, // 1024 * INPUT = 23068672
    pub psqt_weights:   Vec<i32>, // INPUT * PSQT_BUCKETS = 180224

    pub accum:          NNAccum,

}

/// Consts, Init
impl NNFeatureTrans {
    // const HALF_DIMS: usize = 1024;

    const DIMS_IN: usize = 64 * 11 * 64 / 2;
    const DIMS_OUT: usize = HALF_DIMS * 2;

    const PSQT_BUCKETS: usize = 8;
    const LAYER_STACKS: usize = 8;

    pub const HASH: u32 = 0x7f234cb8 ^ Self::DIMS_OUT as u32;

    pub fn new() -> Self {
        Self {
            // nn,
            biases:         vec![0; HALF_DIMS],
            weights:        vec![0; HALF_DIMS * Self::DIMS_IN],
            // weights:        [0; HALF_DIMS * Self::DIMS_IN],
            psqt_weights:   vec![0; Self::DIMS_IN * Self::PSQT_BUCKETS],

            accum:          NNAccum::new(),
        }
    }

    pub fn read_parameters(&mut self, mut rdr: &mut BufReader<File>) -> io::Result<()> {
        // println!("wat NNFeatureTrans");

        let hash = rdr.read_u32::<LittleEndian>()?;
        assert_eq!(hash, Self::HASH);

        for mut x in self.biases.iter_mut() {
            *x = rdr.read_i16::<LittleEndian>()?;
        }

        for mut x in self.weights.iter_mut() {
            *x = rdr.read_i16::<LittleEndian>()?;
        }

        for mut x in self.psqt_weights.iter_mut() {
            *x = rdr.read_i32::<LittleEndian>()?;
        }

        // eprintln!("FT Read");
        // eprintln!("HALF_DIMS = {:?}", HALF_DIMS);
        // eprintln!("Self::DIMS_IN = {:?}", Self::DIMS_IN);
        // eprintln!("Self::PSQT_BUCKETS = {:?}", Self::PSQT_BUCKETS);

        Ok(())
    }

    pub fn write_parameters(&self, mut w: &mut BufWriter<File>) -> io::Result<()> {

        w.write_u32::<LittleEndian>(Self::HASH)?;

        for x in self.biases.iter() {
            w.write_i16::<LittleEndian>(*x)?;
        }
        for x in self.weights.iter() {
            w.write_i16::<LittleEndian>(*x)?;
        }
        for x in self.psqt_weights.iter() {
            w.write_i32::<LittleEndian>(*x)?;
        }
        Ok(())
    }

}

/// Transform
impl NNFeatureTrans {

    // pub fn transform(&mut self, g: &Game, output: &mut [u8], bucket: usize, refresh: bool) -> Score {
    pub fn transform(&mut self, g: &Game, output: &mut [u8], bucket: usize) -> Score {

        let output = &mut output[..HALF_DIMS*2];

        // eprintln!("FT transform");

        // // self.update_accum(g, White, refresh);
        // // self.update_accum(g, Black, refresh);
        // self.update_accum(g, White);
        // self.update_accum(g, Black);

        let persps: [Color; 2] = [g.state.side_to_move, !g.state.side_to_move];
        // let persps: [Color; 2] = [!g.state.side_to_move, g.state.side_to_move];

        let accum      = &mut self.accum.accum;
        let psqt_accum = &mut self.accum.psqt;

        let psqt = (psqt_accum[persps[0]][bucket] - psqt_accum[persps[1]][bucket]) / 2;

        // let mut x = 0;

        for p in 0..2 {
            let offset = HALF_DIMS * p;
            for k in 0..HALF_DIMS {
                let mut sum = accum[persps[p]][k];
                // x ^= sum.clamp(0, 127) as u8;
                output[offset + k] = sum.clamp(0, 127) as u8;
            }
        }

        // eprintln!("x = {:?}", x);

        psqt
    }

}

/// Directly Apply Moves
#[cfg(feature = "nope")]
impl NNFeatureTrans {

    // #[cfg(feature = "nope")]
    pub fn make_move_add(
        // &mut self, persp: Color, king_sq: Coord, pc: Piece, side: Color, sq: Coord) -> NNDelta {
        &mut self, persp: Color, king_sq: Coord, pc: Piece, side: Color, sq: Coord) -> NNIndex {
        // eprintln!("adding ({:?},{:?}) {:?} {:?} at {:?}", persp, king_sq, side, pc, sq);
        let d_add = super::NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
        // eprintln!("d_add = {:?}", d_add);
        self.accum_add(persp, d_add, true);
        d_add
    }

    // #[cfg(feature = "nope")]
    pub fn make_move_rem(
        // &mut self, persp: Color, king_sq: Coord, pc: Piece, side: Color, sq: Coord) -> NNDelta {
        &mut self, persp: Color, king_sq: Coord, pc: Piece, side: Color, sq: Coord) -> NNIndex {
        let d_rem = super::NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
        self.accum_rem(persp, d_rem, true);
        d_rem
    }

    // #[cfg(feature = "nope")]
    pub fn make_move_move(
        &mut self, persp: Color, king_sq: Coord, pc: Piece, side: Color,
        // from: Coord, to: Coord) -> [NNDelta; 2] {
        from: Coord, to: Coord) -> [NNIndex; 2] {
        let x = self.make_move_rem(persp, king_sq, pc, side, from);
        let y = self.make_move_add(persp, king_sq, pc, side, to);
        [x, y]
    }

    #[inline(always)]
    pub fn make_move(&mut self, g: &Game, mv: Move) {
        if mv.piece() == Some(King) {
            self.accum.push_copy();
            self.reset_accum(g);
        } else {
            self.accum.push_copy();
            self.reset_accum(g);
            // self._make_move(g, White, mv);
            // self._make_move(g, Black, mv);
            // self._make_move(g, !g.state.side_to_move, mv);
            // a.extend(b.into_iter());
            // self.accum.stack_delta.push(a);
        }
    }

    /// Noticable speed up
    #[inline(always)]
    #[cfg(feature = "nope")]
    pub fn make_move(&mut self, g: &Game, mv: Move) {
        if mv.piece() == Some(King) {
            self.accum.push_copy();
            self.reset_accum(g);
        } else {
            let mut a = self._make_move(g, White, mv);
            let b = self._make_move(g, Black, mv);
            // self._make_move(g, !g.state.side_to_move, mv);
            a.extend(b.into_iter());
            self.accum.stack_delta.push(a);
        }
    }

    #[inline(always)]
    // #[cfg(feature = "nope")]
    pub fn _make_move(&mut self, g: &Game, persp: Color, mv: Move) -> NNDeltas {

        // self.update_accum(g, White);
        // self.update_accum(g, Black);
        self.update_accum(g, persp);

        let mut out = ArrayVec::new();

        assert!(mv.piece() != Some(King));

        let king_sq = g.get(King,persp).bitscan();
        let side = !g.state.side_to_move;
        // let side = g.state.side_to_move;
        match mv {
            Move::Quiet { from, to, pc } => {
                let a = self.make_move_move(persp, king_sq, pc, side, from, to);
                out.push(a[0]);
                out.push(a[1]);
            },
            Move::PawnDouble { from, to } => {
                let a = self.make_move_move(persp, king_sq, Pawn, side, from, to);
                out.push(a[0]);
                out.push(a[1]);
            },
            Move::Capture { from, to, pc, victim } => {
                let a = self.make_move_move(persp, king_sq, pc, side, from, to);
                let b = self.make_move_rem(persp, king_sq, victim, !side, to);
                out.push(a[0]);
                out.push(a[1]);
                out.push(b);
            },
            Move::EnPassant { from, to, capture } => {
                let a = self.make_move_move(persp, king_sq, Pawn, side, from, to);
                let b = self.make_move_rem(persp, king_sq, Pawn, !side, capture);
                out.push(a[0]);
                out.push(a[1]);
                out.push(b);
            },
            Move::Castle { from, to, rook_from, rook_to } => {
                // let a = self.make_move_move(persp, king_sq, King, side, from, to);
                // let b = self.make_move_move(persp, king_sq, Rook, side, rook_from, rook_to);
                // out.push(a[0]);
                // out.push(a[1]);
                // out.push(b[0]);
                // out.push(b[1]);
                unimplemented!()
            },
            Move::Promotion { from, to, new_piece } => {
                let a = self.make_move_rem(persp, king_sq, Pawn, side, from);
                let b = self.make_move_add(persp, king_sq, new_piece, side, to);
                out.push(a);
                out.push(b);
            },
            Move::PromotionCapture { from, to, new_piece, victim } => {
                let a = self.make_move_rem(persp, king_sq, Pawn, side, from);
                let b = self.make_move_add(persp, king_sq, new_piece, side, to);
                let c = self.make_move_rem(persp, king_sq, victim, !side, to);
                out.push(a);
                out.push(b);
                out.push(c);
            },
            Move::NullMove => {},
        }
        NNDeltas::Deltas(out)
    }

}

/// Directly Apply Moves
impl NNFeatureTrans {

    pub fn make_move_rem(&mut self, ksqs: [Coord; 2], pc: Piece, side: Color, sq: Coord) -> NNDelta {
        let [i_w,i_b] = NNUE4::make_index_2(ksqs, pc, side, sq);
        self.accum_rem(i_w, i_b)
    }

    pub fn make_move_add(&mut self, ksqs: [Coord; 2], pc: Piece, side: Color, sq: Coord) -> NNDelta {
        let [i_w,i_b] = NNUE4::make_index_2(ksqs, pc, side, sq);
        self.accum_add(i_w, i_b)
    }

    pub fn make_move_move(
        &mut self, ksqs: [Coord; 2], pc: Piece, side: Color, from: Coord, to: Coord) -> [NNDelta; 2] {
        let a = self.make_move_rem(ksqs, pc, side, from);
        let b = self.make_move_add(ksqs, pc, side, to);
        [a,b]
    }

    #[inline(always)]
    pub fn make_move(&mut self, g: &Game, mv: Move) {
        if mv.piece() == Some(King) {
            self.accum.push_copy();
            self.reset_accum(g);
        } else {
            // self.accum.push_copy();
            // self.reset_accum(g);
            let ds = self._make_move(g, mv);
            self.accum.stack_delta.push(ds);
        }
    }

    pub fn _make_move(&mut self, g: &Game, mv: Move) -> NNDeltas {

        // self.update_accum(g, White);
        // self.update_accum(g, Black);

        let mut out = ArrayVec::new();

        assert!(mv.piece() != Some(King));

        // let side = g.state.side_to_move;
        let side = !g.state.side_to_move; // XXX: should be after make_move g -> g2

        // let king_sq = g.get(King,persp).bitscan();
        let ksqs = [g.get(King,White).bitscan(),g.get(King,Black).bitscan()];

        match mv {
            Move::Quiet { from, to, pc } => {
                let a = self.make_move_move(ksqs, pc, side, from, to);
                out.push(a[0]);
                out.push(a[1]);
            },
            Move::PawnDouble { from, to } => {
                let a = self.make_move_move(ksqs, Pawn, side, from, to);
                out.push(a[0]);
                out.push(a[1]);
            },
            Move::Capture { from, to, pc, victim } => {
                let a = self.make_move_move(ksqs, pc, side, from, to);
                let b = self.make_move_rem(ksqs, victim, !side, to);
                out.push(a[0]);
                out.push(a[1]);
                out.push(b);
            },
            Move::EnPassant { from, to, capture } => {
                let a = self.make_move_move(ksqs, Pawn, side, from, to);
                let b = self.make_move_rem(ksqs, Pawn, !side, capture);
                out.push(a[0]);
                out.push(a[1]);
                out.push(b);
            },
            Move::Castle { from, to, rook_from, rook_to } => {
                // let a = self.make_move_move(ksqs, King, side, from, to);
                // let b = self.make_move_move(ksqs, Rook, side, rook_from, rook_to);
                // out.push(a[0]);
                // out.push(a[1]);
                // out.push(b[0]);
                // out.push(b[1]);
                unimplemented!()
            },
            Move::Promotion { from, to, new_piece } => {
                let a = self.make_move_rem(ksqs, Pawn, side, from);
                let b = self.make_move_add(ksqs, new_piece, side, to);
                out.push(a);
                out.push(b);
            },
            Move::PromotionCapture { from, to, new_piece, victim } => {
                let a = self.make_move_rem(ksqs, Pawn, side, from);
                let b = self.make_move_add(ksqs, new_piece, side, to);
                let c = self.make_move_rem(ksqs, victim, !side, to);
                out.push(a);
                out.push(b);
                out.push(c);
            },
            Move::NullMove => {},
        }

        NNDeltas::Deltas(out)
    }

}

/// Update Accum
impl NNFeatureTrans {

    #[inline(always)]
    #[cfg(feature = "nope")] // XXX:
    pub fn accum_pop(&mut self) {}

    #[inline(always)]
    #[cfg(feature = "nope")] // XXX:
    pub fn accum_pop(&mut self) {
        self.accum.pop_prev();
    }

    #[inline(always)]
    // #[cfg(feature = "nope")]
    pub fn accum_pop(&mut self) {
        match self.accum.stack_delta.pop() {
            Some(NNDeltas::Deltas(ds)) => {
                for d in ds.into_iter() {
                    match d {
                        NNDelta::Add(i_w,i_b) => {
                            self.accum_add(i_w, i_b);
                        },
                        NNDelta::Remove(i_w,i_b) => {
                            self.accum_rem(i_w, i_b);
                        },
                    }
                }
            },
            Some(NNDeltas::Copy) => {
                self.accum.pop_prev();
            },
            None => {},
        }
    }

    #[inline(always)]
    #[cfg(feature = "nope")] // XXX:
    pub fn accum_pop(&mut self) {
        if let Some(xs) = self.accum.stack_delta.pop() {
            for x in xs {
                match x {
                    NNDelta::Copy        => {
                        self.accum.pop_prev();
                    },
                    NNDelta::Add(d_add,side)    => {
                        self.accum_add(side, d_add, false);
                        // self.accum_add(White, d_add, false);
                        // self.accum_add(Black, d_add, false);
                    },
                    NNDelta::Remove(d_rem,side) => {
                        self.accum_rem(side, d_rem, false);
                        // self.accum_rem(White, d_rem, false);
                        // self.accum_rem(Black, d_rem, false);
                    },
                }
            }
        }
    }

    fn _accum_pop(&mut self, d: NNDelta) {
        match d {
            NNDelta::Add(i_w,i_b) => {
                // self.accum_add(side, d_add, false);
                unimplemented!()
            },
            NNDelta::Remove(i_w,i_b) => {
                unimplemented!()
            },
        }
    }

    #[inline(always)]
    pub fn accum_add(&mut self, i_w: NNIndex, i_b: NNIndex) -> NNDelta {
        self._accum_add(White, i_w);
        self._accum_add(Black, i_b);
        NNDelta::Remove(i_w,i_b)
    }

    #[inline(always)]
    pub fn accum_rem(&mut self, i_w: NNIndex, i_b: NNIndex) -> NNDelta {
        self._accum_rem(White, i_w);
        self._accum_rem(Black, i_b);
        NNDelta::Add(i_w,i_b)
    }

    #[inline(always)]
    pub fn _accum_add(&mut self, persp: Color, idx: NNIndex) {
        let idx = idx.0;
        let offset = HALF_DIMS * idx;

        let mut accum = &mut self.accum.accum[persp][..HALF_DIMS];
        let mut weights = &mut self.weights[offset..offset + HALF_DIMS];

        for j in 0..HALF_DIMS {
            accum[j] += weights[j];
        }
        for k in 0..Self::PSQT_BUCKETS {
            self.accum.psqt[persp][k] += self.psqt_weights[idx * Self::PSQT_BUCKETS + k];
            // if let Some(x) = self.psqt_weights.get(d_add * Self::PSQT_BUCKETS + k) {
            //     self.accum.psqt[persp][k] += *x;
            // }
        }
    }

    #[inline(always)]
    pub fn _accum_rem(&mut self, persp: Color, idx: NNIndex) {
        let idx = idx.0;
        let offset = HALF_DIMS * idx;

        let mut accum = &mut self.accum.accum[persp][..HALF_DIMS];
        let mut weights = &mut self.weights[offset..offset + HALF_DIMS];

        for j in 0..HALF_DIMS {
            // self.accum.accum[persp][j] -= self.weights[offset + j];
            accum[j] -= weights[j];
        }
        for k in 0..Self::PSQT_BUCKETS {
            self.accum.psqt[persp][k] -= self.psqt_weights[idx * Self::PSQT_BUCKETS + k];
            // if let Some(x) = self.psqt_weights.get(d_rem * Self::PSQT_BUCKETS + k) {
            //     self.accum.psqt[persp][k] -= *x;
            // }
        }
    }

    #[inline(always)]
    #[cfg(feature = "nope")] // XXX:
    pub fn accum_add(&mut self, persp: Color, d_add: usize, push: bool) -> NNDelta {
        let offset = HALF_DIMS * d_add;

        let mut accum = &mut self.accum.accum[persp][..HALF_DIMS];
        let mut weights = &mut self.weights[offset..offset + HALF_DIMS];

        for j in 0..HALF_DIMS {
            accum[j] += weights[j];
        }
        for k in 0..Self::PSQT_BUCKETS {
            self.accum.psqt[persp][k] += self.psqt_weights[d_add * Self::PSQT_BUCKETS + k];
            // if let Some(x) = self.psqt_weights.get(d_add * Self::PSQT_BUCKETS + k) {
                // self.accum.psqt[persp][k] += *x;
            // }
        }
        // if push {
        //     self.accum.stack_delta.push(NNDelta::Remove(d_add));
        // }
        // NNDelta::Remove(d_add, None)
        NNDelta::Remove(d_add)
    }

    #[inline(always)]
    #[cfg(feature = "nope")] // XXX:
    pub fn accum_rem(&mut self, persp: Color, d_rem: usize, push: bool) -> NNDelta {
        let offset = HALF_DIMS * d_rem;

        let mut accum = &mut self.accum.accum[persp][..HALF_DIMS];
        let mut weights = &mut self.weights[offset..offset + HALF_DIMS];

        for j in 0..HALF_DIMS {
            // self.accum.accum[persp][j] -= self.weights[offset + j];
            accum[j] -= weights[j];
        }
        for k in 0..Self::PSQT_BUCKETS {
            self.accum.psqt[persp][k] -= self.psqt_weights[d_rem * Self::PSQT_BUCKETS + k];
            // if let Some(x) = self.psqt_weights.get(d_rem * Self::PSQT_BUCKETS + k) {
                // self.accum.psqt[persp][k] -= *x;
            // }
        }
        // TODO: for mut p in self.accum.psqt.iter_mut()
        // if push {
        //     self.accum.stack_delta.push(NNDelta::Add(d_rem));
        // }
        // NNDelta::Add(d_rem, None)
        NNDelta::Add(d_rem)
    }

    // pub fn apply_deltas(&mut self, persp: Color) {
    //     let rems = self.accum.deltas_rem.clone();
    //     let adds = self.accum.deltas_add.clone();
    //     self.accum.deltas_rem.clear();
    //     self.accum.deltas_add.clear();
    //     for d_rem in rems.into_iter() {
    //         self.accum_rem(persp, d_rem, true);
    //     }
    //     for d_add in adds.into_iter() {
    //         self.accum_add(persp, d_add, true);
    //     }
    // }

    // #[inline(always)]
    pub fn reset_accum(&mut self, g: &Game) {
        // debug!("resetting accum");
        self._update_accum(g, White);
        self._update_accum(g, Black);
        // self.accum.needs_refresh = [false; 2];
    }

    // // #[inline(always)]
    // pub fn update_accum(&mut self, g: &Game, persp: Color) {
    //     if self.accum.needs_refresh[persp] {
    //         self.accum.needs_refresh[persp] = false;
    //         self._update_accum(g, persp);
    //     }
    // }

    // #[inline(always)]
    pub fn _update_accum(&mut self, g: &Game, persp: Color) {
        assert!(self.biases.len() == self.accum.accum[persp].len());
        self.accum.accum[persp].copy_from_slice(&self.biases);

        let mut active = ArrayVec::default();
        NNAccum::append_active(g, persp, &mut active);

        self.accum.psqt[persp].fill(0);

        for idx in active.into_iter() {
            let offset = HALF_DIMS * idx.0;
            for j in 0..HALF_DIMS {
                self.accum.accum[persp][j] += self.weights[offset + j];
            }
            for k in 0..Self::PSQT_BUCKETS {
                self.accum.psqt[persp][k] += self.psqt_weights[idx.0 * Self::PSQT_BUCKETS + k];
            }
        }
    }

}
