
use crate::evaluate::Score;
use crate::types::*;
use crate::sf_compat::accumulator::*;

use super::HALF_DIMS;
use super::accumulator::NNAccum;

use std::io::{self, Read,BufReader, BufWriter};
use std::fs::File;
use std::path::Path;

use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};

// #[derive(Debug,PartialEq,Clone)]
#[derive(Debug,Eq,PartialEq,PartialOrd,Ord,Clone)]
pub struct NNFeatureTrans {
    pub biases:         Vec<i16>, // 1024
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

    pub fn transform(&mut self, g: &Game, output: &mut [u8], bucket: usize, refresh: bool) -> Score {

        // eprintln!("FT transform");

        self.update_accum(g, White, refresh);
        self.update_accum(g, Black, refresh);

        let persps: [Color; 2] = [g.state.side_to_move, !g.state.side_to_move];
        // let persps: [Color; 2] = [!g.state.side_to_move, g.state.side_to_move];

        let accum      = &mut self.accum.accum;
        let psqt_accum = &mut self.accum.psqt;

        let psqt = (psqt_accum[persps[0]][bucket] - psqt_accum[persps[1]][bucket]) / 2;

        let mut x = 0;

        for p in 0..2 {
            let offset = HALF_DIMS * p;
            for j in 0..HALF_DIMS {
                let mut sum = accum[persps[p]][j];
                x ^= sum.clamp(0, 127) as u8;
                output[offset + j] = sum.clamp(0, 127) as u8;
            }
        }

        // eprintln!("x = {:?}", x);

        psqt
    }

}

/// Directly Apply Moves
impl NNFeatureTrans {

    pub fn make_move_add(
        &mut self, persp: Color, king_sq: Coord, pc: Piece, side: Color, sq: Coord) {
        let d_add = super::NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
        self.accum_add(persp, d_add, true);
    }

    pub fn make_move_rem(
        &mut self, persp: Color, king_sq: Coord, pc: Piece, side: Color, sq: Coord) {
        let d_rem = super::NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
        self.accum_rem(persp, d_rem, true);
    }

    pub fn make_move_move(
        &mut self, persp: Color, king_sq: Coord, pc: Piece, side: Color, from: Coord, to: Coord) {
        self.make_move_rem(persp, king_sq, pc, side, from);
        self.make_move_add(persp, king_sq, pc, side, to);
    }

    // XXX: No op
    #[inline(always)]
    pub fn make_move(&mut self, g: &Game, mv: Move) {
        // self._make_move(g, White, mv);
        // self._make_move(g, Black, mv);
    }

    #[inline(always)]
    pub fn _make_move(&mut self, g: &Game, persp: Color, mv: Move) {
        let king_sq = g.get(King,persp).bitscan();
        let side = g.state.side_to_move;
        match mv {
            Move::Quiet { from, to, pc } => {
                self.make_move_move(persp, king_sq, pc, side, from, to);
            },
            Move::PawnDouble { from, to } => {
                self.make_move_move(persp, king_sq, Pawn, side, from, to);
            },
            Move::Capture { from, to, pc, victim } => {
                self.make_move_move(persp, king_sq, pc, side, from, to);
                self.make_move_rem(persp, king_sq, victim, !side, to);
            },
            Move::EnPassant { from, to, capture } => {
                self.make_move_move(persp, king_sq, Pawn, side, from, to);
                self.make_move_rem(persp, king_sq, Pawn, !side, capture);
            },
            Move::Castle { from, to, rook_from, rook_to } => {
                self.make_move_move(persp, king_sq, King, side, from, to);
                self.make_move_move(persp, king_sq, Rook, side, rook_from, rook_to);
            },
            Move::Promotion { from, to, new_piece } => {
                self.make_move_rem(persp, king_sq, Pawn, side, from);
                self.make_move_add(persp, king_sq, new_piece, side, to);
            },
            Move::PromotionCapture { from, to, new_piece, victim } => {
                self.make_move_rem(persp, king_sq, Pawn, side, from);
                self.make_move_add(persp, king_sq, new_piece, side, to);
                self.make_move_rem(persp, king_sq, victim, !side, to);
            },
            Move::NullMove => {},
        }
    }

}

/// Update Accum
impl NNFeatureTrans {

    #[inline(always)]
    pub fn accum_pop(&mut self) {}

    #[inline(always)]
    pub fn _accum_pop(&mut self) {
        if let Some(delta) = self.accum.stack.pop() {
            match delta {
                NNDelta::Add(d_add)    => {
                    self.accum_add(White, d_add, false);
                    self.accum_add(Black, d_add, false);
                },
                NNDelta::Remove(d_rem) => {
                    self.accum_rem(White, d_rem, false);
                    self.accum_rem(Black, d_rem, false);
                },
            }
        }
    }

    pub fn accum_add(&mut self, persp: Color, d_add: usize, push: bool) {
        let offset = HALF_DIMS * d_add;
        for j in 0..HALF_DIMS {
            self.accum.accum[persp][j] += self.weights[offset + j];
        }
        for k in 0..Self::PSQT_BUCKETS {
            self.accum.psqt[persp][k] += self.psqt_weights[d_add * Self::PSQT_BUCKETS + k];
        }
        if push {
            self.accum.stack.push(NNDelta::Remove(d_add));
        }
    }

    pub fn accum_rem(&mut self, persp: Color, d_rem: usize, push: bool) {
        let offset = HALF_DIMS * d_rem;
        for j in 0..HALF_DIMS {
            self.accum.accum[persp][j] -= self.weights[offset + j];
        }
        for k in 0..Self::PSQT_BUCKETS {
            self.accum.psqt[persp][k] -= self.psqt_weights[d_rem * Self::PSQT_BUCKETS + k];
        }
        // TODO: for mut p in self.accum.psqt.iter_mut()
        if push {
            self.accum.stack.push(NNDelta::Add(d_rem));
        }
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

    #[inline(always)]
    pub fn update_accum(&mut self, g: &Game, persp: Color, refresh: bool) {

        // if self.accum.needs_refresh[persp] {
        if true {
            // full refresh
            // if !refresh {
            //     self.accum.needs_refresh[persp] = false;
            // }

            assert!(self.biases.len() == self.accum.accum[persp].len());
            self.accum.accum[persp].copy_from_slice(&self.biases);

            let mut active = ArrayVec::default();
            NNAccum::append_active(g, persp, &mut active);

            self.accum.psqt[persp].fill(0);

            for idx in active.into_iter() {
                let offset = HALF_DIMS * idx;

                for j in 0..HALF_DIMS {
                    self.accum.accum[persp][j] += self.weights[offset + j];
                }

                for k in 0..Self::PSQT_BUCKETS {
                    self.accum.psqt[persp][k] += self.psqt_weights[idx * Self::PSQT_BUCKETS + k];
                }

            }
        } else {
            // incremental
            // self.apply_deltas(persp);
        }

    }
}






