
#[cfg(not(feature = "prev_accum"))]
pub use self::new::*;
#[cfg(feature = "prev_accum")]
pub use self::old::*;

#[cfg(not(feature = "prev_accum"))]
mod new {
    use crate::types::*;
    use crate::sf_compat::accumulator::new::*;
    use crate::sf_compat::{NNIndex,HALF_DIMS,NNUE4};

    use std::io::{self, Read,BufReader, BufWriter};
    use std::fs::File;
    use std::path::Path;

    use arrayvec::ArrayVec;
    use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};
    use aligned::{Aligned,A64,A32};


    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
    pub struct NNFeatureTrans {
        // pub biases:         Vec<i16>, // 1024
        pub biases:         Aligned<A64,Vec<i16>>, // 1024

        // pub weights:        [i16; Self::DIMS_IN * HALF_DIMS], // stack overflows
        // pub weights:        Vec<i16>, // 1024 * INPUT = 23068672
        // pub psqt_weights:   Vec<i32>, // INPUT * PSQT_BUCKETS = 180224

        pub weights:        Aligned<A64,Vec<i16>>, // 1024 * INPUT = 23068672
        pub psqt_weights:   Aligned<A64,Vec<i32>>, // INPUT * PSQT_BUCKETS = 180224

        pub accum_stack:    Vec<NNAccum>,

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
                biases:         Aligned(vec![0; HALF_DIMS]),
                weights:        Aligned(vec![0; HALF_DIMS * Self::DIMS_IN]),
                // weights:        [0; HALF_DIMS * Self::DIMS_IN],
                psqt_weights:   Aligned(vec![0; Self::DIMS_IN * Self::PSQT_BUCKETS]),

                accum_stack:    vec![],
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

    /// Apply deltas
    impl NNFeatureTrans {
        pub fn apply_deltas(&mut self, g: &Game, persp: Color) {

            /// find index of most recent computed accum
            let mut idx = self.accum_stack.len() - 1;
            loop {
                if let Some(acc) = self.accum_stack.get(idx) {
                    if acc.computed[persp] { break; }
                }
                if idx == 0 { /// this state should never occur
                    unreachable!()
                }
                idx -= 1;
            }

            // eprintln!("most recent computed idx = {:?}", idx);

            let weights = &self.weights;
            let psqt_weights = &self.psqt_weights;

            for accum_idx in idx+1..self.accum_stack.len() {
                // self.apply_delta(persp, k).unwrap();
                // eprintln!("accum_idx = {:?}", accum_idx);

                self.accum_stack[accum_idx].accum[persp] =
                    self.accum_stack[accum_idx - 1].accum[persp];
                self.accum_stack[accum_idx].psqt[persp]  =
                    self.accum_stack[accum_idx - 1].psqt[persp];

                let acc = self.accum_stack.get_mut(accum_idx).unwrap();

                for delta in acc.deltas.clone().iter() {
                    Self::apply_delta(weights, psqt_weights, acc, *delta, persp);
                }

            }

            // for (prev,acc) in self.accum_stack[idx].ite
            // for acc in self.accum_stack[idx].iter_mut().peekable() {}

        }

        #[cfg(feature = "nope")]
        fn apply_delta(&mut self, persp: Color, accum_idx: usize) -> Option<()> {
            // let prev = self.accum_stack.get(accum_idx - 1)?;
            // let acc  = self.accum_stack.get_mut(accum_idx)?;

            // acc.accum[persp] = prev.accum[persp].clone();
            // acc.psqt[persp]  = prev.psqt[persp].clone();

            self.accum_stack[accum_idx].accum[persp] =
                self.accum_stack[accum_idx - 1].accum[persp];
            self.accum_stack[accum_idx].psqt[persp]  =
                self.accum_stack[accum_idx - 1].psqt[persp];

            let weights = &self.weights;
            let psqt_weights = &self.psqt_weights;

            let acc = self.accum_stack.get_mut(accum_idx)?;

            for delta in acc.deltas.clone().iter() {
                Self::_apply_delta(weights, psqt_weights, acc, persp);
            }

            // acc.computed[persp] = true;

            // unimplemented!()
            Some(())
        }

        fn apply_delta(ws: &[i16], psqt_ws: &[i32], acc: &mut NNAccum, delta: NNDelta, persp: Color) {
            match delta {
                NNDelta::Add(w,b)    => {
                    let idx = if persp == White { w } else { b };
                    Self::_apply_delta::<true>(ws, psqt_ws, acc, persp, idx);
                },
                NNDelta::Remove(w,b) => {
                    let idx = if persp == White { w } else { b };
                    Self::_apply_delta::<false>(ws, psqt_ws, acc, persp, idx);
                },
            }
        }

        fn _apply_delta<const ADD: bool>(
            ws:               &[i16],
            psqt_ws:          &[i32],
            acc:              &mut NNAccum,
            persp:            Color,
            idx:              NNIndex,
        ) {
            let idx = idx.0;
            let offset = HALF_DIMS * idx;

            let mut accum = &mut acc.accum[persp][..HALF_DIMS];
            let weights = &ws[offset..offset + HALF_DIMS];

            assert!(accum.len() == HALF_DIMS);
            assert!(weights.len() == HALF_DIMS);

            if ADD {
                for j in 0..HALF_DIMS {
                    accum[j] += weights[j];
                }
                for k in 0..Self::PSQT_BUCKETS {
                    if let Some(x) = psqt_ws.get(idx * Self::PSQT_BUCKETS + k) {
                        acc.psqt[persp][k] += *x;
                    }
                }
            } else {
                for j in 0..HALF_DIMS {
                    accum[j] -= weights[j];
                }
                for k in 0..Self::PSQT_BUCKETS {
                    if let Some(x) = psqt_ws.get(idx * Self::PSQT_BUCKETS + k) {
                        acc.psqt[persp][k] -= *x;
                    }
                }
            }
        }

    }

    /// Accum add, sub, no simd
    impl NNFeatureTrans {

        #[cfg(feature = "nope")]
        pub fn _accum_add(&mut self, persp: Color, idx: NNIndex) {
            let idx = idx.0;
            let offset = HALF_DIMS * idx;

            let mut accum = &mut self.accum.accum[persp][..HALF_DIMS];
            let weights = &self.weights[offset..offset + HALF_DIMS];

            assert!(accum.len() == HALF_DIMS);
            assert!(weights.len() == HALF_DIMS);

            for j in 0..HALF_DIMS {
                accum[j] += weights[j];
            }
            for k in 0..Self::PSQT_BUCKETS {
                // self.accum.psqt[persp][k] += self.psqt_weights[idx * Self::PSQT_BUCKETS + k];
                if let Some(x) = self.psqt_weights.get(idx * Self::PSQT_BUCKETS + k) {
                    self.accum.psqt[persp][k] += *x;
                }
            }
        }

        #[cfg(feature = "nope")]
        pub fn _accum_rem(&mut self, persp: Color, idx: NNIndex) {
            let idx = idx.0;
            let offset = HALF_DIMS * idx;

            let mut accum = &mut self.accum.accum[persp][..HALF_DIMS];
            let weights = &self.weights[offset..offset + HALF_DIMS];

            assert!(accum.len() == HALF_DIMS);
            assert!(weights.len() == HALF_DIMS);

            // for (j,a) in accum.iter_mut().enumerate() {
            //     *a -= weights[j];
            // }

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

    }

    /// transform
    impl NNFeatureTrans {
        pub fn transform(&mut self, g: &Game, output: &mut [u8], bucket: usize) -> Score {

            self.apply_deltas(g, White);
            self.apply_deltas(g, Black);

            let output = &mut output[..HALF_DIMS*2];

            let persps: [Color; 2] = [g.state.side_to_move, !g.state.side_to_move];

            let acc: &mut NNAccum = self.accum_stack.last_mut().unwrap();

            let accum      = &mut acc.accum;
            let psqt_accum = &mut acc.psqt;

            // let accum:      &mut Aligned<A32,[[i16; 1024]; 2]> =
            //     if let Some(accum) = acc.accum.as_mut() { accum } else { unreachable!() };
            // let psqt_accum: &mut Aligned<A32,[[i32; 8]; 2]> =
            //     if let Some(psqt_accum) = acc.psqt.as_mut() { psqt_accum } else { unreachable!() };

            let psqt = (psqt_accum[persps[0]][bucket] - psqt_accum[persps[1]][bucket]) / 2;

            for p in 0..2 {
                let offset = HALF_DIMS * p;
                for k in 0..HALF_DIMS {
                    let mut sum = accum[persps[p]][k];
                    // x ^= sum.clamp(0, 127) as u8;
                    output[offset + k] = sum.clamp(0, 127) as u8;
                }
            }

            psqt

            // unimplemented!()
        }
    }

    /// reset_accum, init_fresh_accum
    impl NNFeatureTrans {

        pub fn reset_accum(&mut self, g: &Game) {
            self.accum_stack.clear();
            let acc = self.init_fresh_accum(g);
            self.accum_stack.push(acc);
        }

        pub fn init_fresh_accum(&self, g: &Game) -> NNAccum {
            let mut acc = NNAccum::default();
            self._reset_accum(g, White, &mut acc);
            self._reset_accum(g, Black, &mut acc);
            acc
        }

        /// only used to make a fresh accum, for first node and king moves
        pub fn _reset_accum(&self, g: &Game, persp: Color, accum: &mut NNAccum) {
            assert!(self.biases.len() == accum.accum[persp].len());
            accum.accum[persp].copy_from_slice(&self.biases);

            let mut active = ArrayVec::default();
            NNAccum::append_active(g, persp, &mut active);

            accum.psqt[persp].fill(0);

            for idx in active.into_iter() {
                let offset = HALF_DIMS * idx.0;
                for j in 0..HALF_DIMS {
                    accum.accum[persp][j] += self.weights[offset + j];
                }
                for k in 0..Self::PSQT_BUCKETS {
                    accum.psqt[persp][k] += self.psqt_weights[idx.0 * Self::PSQT_BUCKETS + k];
                }
            }

            accum.computed = [true; 2];
        }

    }

    /// pop
    impl NNFeatureTrans {
        pub fn accum_pop(&mut self) {
            self.accum_stack.pop();
        }
    }

    /// make_move
    impl NNFeatureTrans {

        pub fn make_move_move(
            &mut self, ksqs: [Coord; 2], pc: Piece, side: Color, from: Coord, to: Coord) -> [NNDelta; 2] {
            let a = self.make_move_rem(ksqs, pc, side, from);
            let b = self.make_move_add(ksqs, pc, side, to);
            [a,b]
        }

        pub fn make_move_add(&mut self, ksqs: [Coord; 2], pc: Piece, side: Color, sq: Coord) -> NNDelta {
            let [i_w,i_b] = NNUE4::make_index_2(ksqs, pc, side, sq);
            NNDelta::Add(i_w,i_b)
        }

        pub fn make_move_rem(&mut self, ksqs: [Coord; 2], pc: Piece, side: Color, sq: Coord) -> NNDelta {
            let [i_w,i_b] = NNUE4::make_index_2(ksqs, pc, side, sq);
            NNDelta::Remove(i_w,i_b)
        }

        /// note: g has already had move made
        pub fn make_move(&mut self, g: &Game, mv: Move) {

            assert_eq!(Some(mv), g.last_move);

            if mv.piece() == Some(King) {
                let acc = self.init_fresh_accum(g);
                self.accum_stack.push(acc);
            } else {

                // let mut acc = NNAccum::default();
                // // acc.computed = [false; 2];
                // self.accum_stack.push(acc);

                let deltas = self._make_move(g, mv);
                let prev = self.accum_stack.last().unwrap();
                let acc = NNAccum::new_from_prev(prev, deltas);
                self.accum_stack.push(acc);

            }
        }

        pub fn _make_move(&mut self, g: &Game, mv: Move) -> ArrayVec<NNDelta,3> {

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
                // Move::Capture { from, to, pc, victim } => {
                Move::Capture { from, to, pcs } => {
                    // let a = self.make_move_move(ksqs, pc, side, from, to);
                    // let b = self.make_move_rem(ksqs, victim, !side, to);
                    let a = self.make_move_move(ksqs, pcs.first(), side, from, to);
                    let b = self.make_move_rem(ksqs, pcs.second(), !side, to);
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
                // Move::Castle { from, to, rook_from, rook_to } => {
                Move::Castle { .. } => {
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
                // Move::PromotionCapture { from, to, new_piece, victim } => {
                Move::PromotionCapture { from, to, pcs } => {
                    let a = self.make_move_rem(ksqs, Pawn, side, from);
                    // let b = self.make_move_add(ksqs, new_piece, side, to);
                    // let c = self.make_move_rem(ksqs, victim, !side, to);
                    let b = self.make_move_add(ksqs, pcs.first(), side, to);
                    let c = self.make_move_rem(ksqs, pcs.second(), !side, to);
                    out.push(a);
                    out.push(b);
                    out.push(c);
                },
                Move::NullMove => {},
            }

            // NNDeltas::Deltas(out)
            out
        }

    }

}

#[cfg(feature = "prev_accum")]
mod old {
    use crate::types::*;
    use crate::sf_compat::accumulator::*;
    use crate::sf_compat::NNIndex;

    use crate::sf_compat::{HALF_DIMS, NNUE4};
    use crate::sf_compat::accumulator::NNAccum;

    use std::io::{self, Read,BufReader, BufWriter};
    use std::fs::File;
    use std::path::Path;

    use arrayvec::ArrayVec;
    use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};
    use aligned::{Aligned,A64,A32};

    // #[derive(Debug,PartialEq,Clone)]
    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
    pub struct NNFeatureTrans {
        // pub biases:         Vec<i16>, // 1024
        pub biases:         Aligned<A64,Vec<i16>>, // 1024

        // pub weights:        [i16; Self::DIMS_IN * HALF_DIMS], // stack overflows
        // pub weights:        Vec<i16>, // 1024 * INPUT = 23068672
        // pub psqt_weights:   Vec<i32>, // INPUT * PSQT_BUCKETS = 180224

        pub weights:        Aligned<A64,Vec<i16>>, // 1024 * INPUT = 23068672
        pub psqt_weights:   Aligned<A64,Vec<i32>>, // INPUT * PSQT_BUCKETS = 180224

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
                biases:         Aligned(vec![0; HALF_DIMS]),
                weights:        Aligned(vec![0; HALF_DIMS * Self::DIMS_IN]),
                // weights:        [0; HALF_DIMS * Self::DIMS_IN],
                psqt_weights:   Aligned(vec![0; Self::DIMS_IN * Self::PSQT_BUCKETS]),

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

            // self.reset_accum(g);

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
            // psqt.clamp(i16::MIN as i32,i16::MAX as i32) as i16
        }

    }

    /// SIMD
    #[cfg(target_feature = "avx2")]
    impl NNFeatureTrans {

        const NUM_REGS: usize = 16; // AVX2
        const NUM_REGS_PSQT: usize = 1; // AVX2

        /// AVX2 = 256
        const TILE_HEIGHT: usize = Self::NUM_REGS * std::mem::size_of::<safe_arch::m256i>() / 2;
        /// AVX2 = 8
        const TILE_HEIGHT_PSQT: usize = Self::NUM_REGS_PSQT * std::mem::size_of::<safe_arch::m256i>() / 4;

        pub fn _update_accum_simd(&mut self, g: &Game, persp: Color) {
            use safe_arch::*;
            use crate::simd_utils::safe_arch::*;

            assert!(self.biases.len() == self.accum.accum[persp].len());
            self.accum.accum[persp].copy_from_slice(&self.biases);

            let mut active = ArrayVec::default();
            NNAccum::append_active(g, persp, &mut active);

            let mut acc      = [m256i::default(); Self::NUM_REGS];
            let mut acc_psqt = [m256i::default(); Self::NUM_REGS_PSQT];

            for k in 0..HALF_DIMS / Self::TILE_HEIGHT {

                let biases_tile: &[m256i] = unsafe {
                    let bs = &self.biases[k * Self::TILE_HEIGHT..];
                    cast_slice_to_m256i(&bs)
                };

                for i in 0..Self::NUM_REGS {
                    acc[i] = biases_tile[i];
                }

                for idx in active.iter() {
                    let offset = HALF_DIMS * idx.0 + k * Self::TILE_HEIGHT;

                    let column = unsafe { cast_slice_to_m256i(&self.weights[offset..]) };

                    for i in 0..Self::NUM_REGS {
                        acc[i] = add_i16_m256i(acc[i], column[i]);
                    }
                }

                let acc_tile: &mut [m256i] = unsafe {
                    let xs = &mut self.accum.accum[persp][k * Self::TILE_HEIGHT..];
                    cast_slice_to_m256i_mut(xs)
                };

                for i in 0..Self::NUM_REGS {
                    // vec_store(&mut accTile[k], acc[k]);
                    store_m256i(&mut acc_tile[i], acc[i]);
                }

            }

            for k in 0..Self::PSQT_BUCKETS / Self::TILE_HEIGHT_PSQT {
                self.accum.psqt[persp].fill(0);

                for idx in active.iter() {
                    let offset = Self::PSQT_BUCKETS * idx.0 + k * Self::TILE_HEIGHT_PSQT;

                    let column_psqt = unsafe { cast_slice_to_m256i(&self.psqt_weights[offset..]) };

                    for i in 0..Self::NUM_REGS_PSQT {
                        acc_psqt[i] = add_i32_m256i(acc_psqt[i], column_psqt[i]);
                    }
                }

                let acc_tile_psqt: &mut [m256i] = unsafe {
                    let xs = &mut self.accum.psqt[persp][k * Self::TILE_HEIGHT_PSQT..];
                    cast_slice_to_m256i_mut(xs)
                };

                for i in 0..Self::NUM_REGS_PSQT {
                    store_m256i(&mut acc_tile_psqt[i], acc_psqt[i]);
                }

            }

        }

        pub fn _accum_inc_simd<const ADD: bool>(&mut self, persp: Color, idx: NNIndex) {
            use safe_arch::*;
            use crate::simd_utils::safe_arch::*;

            let mut acc      = [m256i::default(); Self::NUM_REGS];

            for k in 0..HALF_DIMS / Self::TILE_HEIGHT {
                let acc_tile: &mut [m256i] = unsafe {
                    let xs = &mut self.accum.accum[persp][k * Self::TILE_HEIGHT..];
                    cast_slice_to_m256i_mut(xs)
                };

                for i in 0..Self::NUM_REGS {
                    acc[i] = load_m256i(&acc_tile[i]);
                }

                let offset = HALF_DIMS * idx.0 + k * Self::TILE_HEIGHT;
                let column = unsafe { cast_slice_to_m256i(&self.weights[offset..]) };

                for i in 0..Self::NUM_REGS {
                    if ADD {
                        acc[i] = add_i16_m256i(acc[i], column[i]);
                    } else {
                        acc[i] = sub_i16_m256i(acc[i], column[i]);
                    }
                }

                for i in 0..Self::NUM_REGS {
                    store_m256i(&mut acc_tile[i], acc[i]);
                    // acc_tile[i] = acc[i];
                }
            }

            // drop(acc);
            let mut acc_psqt = [m256i::default(); Self::NUM_REGS_PSQT];

            for k in 0..Self::PSQT_BUCKETS / Self::TILE_HEIGHT_PSQT {
                let acc_tile_psqt: &mut [m256i] = unsafe {
                    let xs = &mut self.accum.psqt[persp][k * Self::TILE_HEIGHT_PSQT..];
                    cast_slice_to_m256i_mut(xs.as_mut())
                };
                for i in 0..Self::NUM_REGS_PSQT {
                    acc_psqt[i] = load_m256i(&acc_tile_psqt[i]);
                    // acc_psqt[i] = acc_tile_psqt[i];
                }
                let offset = Self::PSQT_BUCKETS * idx.0 + k * Self::TILE_HEIGHT_PSQT;
                let column_psqt = unsafe { cast_slice_to_m256i(&self.psqt_weights[offset..]) };
                for i in 0..Self::NUM_REGS_PSQT {
                    if ADD {
                        acc_psqt[i] = add_i32_m256i(acc_psqt[i], column_psqt[i]);
                    } else {
                        acc_psqt[i] = sub_i32_m256i(acc_psqt[i], column_psqt[i]);
                    }
                }
                for i in 0..Self::NUM_REGS_PSQT {
                    store_m256i(&mut acc_tile_psqt[i], acc_psqt[i]);
                    // acc_tile_psqt[i] = acc_psqt[i];
                }
            }

            // let idx = idx.0;
            // let offset = HALF_DIMS * idx;

            // let mut accum = &mut self.accum.accum[persp][..HALF_DIMS];
            // let weights = &self.weights[offset..offset + HALF_DIMS];

            // for j in 0..HALF_DIMS {
            //     if ADD {
            //         accum[j] += weights[j];
            //     } else {
            //         accum[j] -= weights[j];
            //     }
            // }

            // for k in 0..Self::PSQT_BUCKETS {
            //     if ADD {
            //         self.accum.psqt[persp][k] += self.psqt_weights[idx * Self::PSQT_BUCKETS + k];
            //     } else {
            //         self.accum.psqt[persp][k] -= self.psqt_weights[idx * Self::PSQT_BUCKETS + k];
            //     }
            // }

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

        #[cfg(feature = "nope")] // XXX: 
        pub fn make_move(&mut self, g: &Game, mv: Move) {
            if let Move::Castle { from, to, rook_from, rook_to } = mv {

                // let persp = g.state.side_to_move;
                // let ksq = g.get(King,persp).bitscan();
                // let from = NNUE4::make_index_half_ka_v2(ksq, persp, King, !persp, from);
                // let to   = NNUE4::make_index_half_ka_v2(ksq, persp, King, !persp, to);
                // let rook_from = NNUE4::make_index_half_ka_v2(ksq, persp, King, !persp, rook_from);
                // let rook_to   = NNUE4::make_index_half_ka_v2(ksq, persp, King, !persp, rook_to);

                // self.accum.push_copy_castle(!g.state.side_to_move,((from,to),(rook_from,rook_to)));

                self.accum.push_copy_full(!g.state.side_to_move);
                self.reset_accum(g);

            } else if mv.piece() == Some(King) {
                let persp = g.state.side_to_move;
                let ksq = g.get(King,persp).bitscan();

                let from = NNUE4::make_index_half_ka_v2(ksq, persp, King, !persp, mv.sq_from());
                let to   = NNUE4::make_index_half_ka_v2(ksq, persp, King, !persp, mv.sq_to());

                // self.accum.push_copy(!g.state.side_to_move);
                self.accum.push_copy_half(!g.state.side_to_move,(from,to));
                self.reset_accum(g);
            } else {
                let ds = self._make_move(g, mv);
                self.accum.stack_delta.push(NNDeltas::Deltas(ds));
            }
        }

        // #[cfg(feature = "nope")] // XXX: 
        pub fn make_move(&mut self, g: &Game, mv: Move) {
            if mv.piece() == Some(King) {
                self.accum.push_copy_full(!g.state.side_to_move);
                self.reset_accum(g);
            } else {
                let ds = self._make_move(g, mv);
                self.accum.stack_delta.push(NNDeltas::Deltas(ds));
            }
        }

        pub fn _make_move(&mut self, g: &Game, mv: Move) -> ArrayVec<NNDelta,3> {

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
                // Move::Capture { from, to, pc, victim } => {
                Move::Capture { from, to, pcs } => {
                    // let a = self.make_move_move(ksqs, pc, side, from, to);
                    // let b = self.make_move_rem(ksqs, victim, !side, to);
                    let a = self.make_move_move(ksqs, pcs.first(), side, from, to);
                    let b = self.make_move_rem(ksqs, pcs.second(), !side, to);
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
                // Move::Castle { from, to, rook_from, rook_to } => {
                Move::Castle { .. } => {
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
                // Move::PromotionCapture { from, to, new_piece, victim } => {
                Move::PromotionCapture { from, to, pcs } => {
                    let a = self.make_move_rem(ksqs, Pawn, side, from);
                    // let b = self.make_move_add(ksqs, new_piece, side, to);
                    // let c = self.make_move_rem(ksqs, victim, !side, to);
                    let b = self.make_move_add(ksqs, pcs.first(), side, to);
                    let c = self.make_move_rem(ksqs, pcs.second(), !side, to);
                    out.push(a);
                    out.push(b);
                    out.push(c);
                },
                Move::NullMove => {},
            }

            // NNDeltas::Deltas(out)
            out
        }

    }

    /// Update Accum
    impl NNFeatureTrans {

        pub fn accum_pop(&mut self) {
            match self.accum.stack_delta.pop() {
                Some(NNDeltas::Deltas(ds)) => {
                    for d in ds.into_iter() {
                        self._accum_pop(d);
                    }
                },

                // Some(NNDeltas::CopyCastle(persp,(from,to),(rook_from,rook_to))) => {
                //     self.accum.pop_prev();
                //     self._accum_add(!persp, from);
                //     self._accum_rem(!persp, to);
                //     self._accum_add(!persp, rook_from);
                //     self._accum_rem(!persp, rook_to);
                // },

                // Some(NNDeltas::CopyCastle(persp)) => {
                //     self.accum.pop_prev();
                //     self.accum.pop_prev();
                // }

                // Some(NNDeltas::CopyKing(persp,(from,to))) => {
                //     self.accum.pop_prev();
                //     self._accum_add(!persp, from);
                //     self._accum_rem(!persp, to);
                // },

                Some(NNDeltas::Copy) => {
                    self.accum.pop_prev();
                },

                None => {
                    panic!("empty stack pop?");
                },
            }
        }

        fn _accum_pop(&mut self, d: NNDelta) {
            match d {
                NNDelta::Add(i_w,i_b) => {
                    self.accum_add(i_w, i_b);
                },
                NNDelta::Remove(i_w,i_b) => {
                    self.accum_rem(i_w, i_b);
                },
            }
        }

        /// temp no simd
        pub fn accum_add(&mut self, i_w: NNIndex, i_b: NNIndex) -> NNDelta {

            // #[cfg(not(target_feature = "avx2"))]
            // self._accum_add(White, i_w);
            // #[cfg(not(target_feature = "avx2"))]
            // self._accum_add(Black, i_b);

            // #[cfg(target_feature = "avx2")]
            // self._accum_inc_simd::<true>(White, i_w);
            // #[cfg(target_feature = "avx2")]
            // self._accum_inc_simd::<true>(Black, i_b);

            self._accum_add(White, i_w);
            self._accum_add(Black, i_b);

            NNDelta::Remove(i_w,i_b)
        }

        /// temp no simd
        pub fn accum_rem(&mut self, i_w: NNIndex, i_b: NNIndex) -> NNDelta {
            // eprintln!("rem (i_w,i_b) = {:?}", (i_w,i_b));

            // #[cfg(not(target_feature = "avx2"))]
            // self._accum_rem(White, i_w);
            // #[cfg(not(target_feature = "avx2"))]
            // self._accum_rem(Black, i_b);

            // #[cfg(target_feature = "avx2")]
            // self._accum_inc_simd::<false>(White, i_w);
            // #[cfg(target_feature = "avx2")]
            // self._accum_inc_simd::<false>(Black, i_b);

            self._accum_rem(White, i_w);
            self._accum_rem(Black, i_b);

            NNDelta::Add(i_w,i_b)
        }

        pub fn _accum_add(&mut self, persp: Color, idx: NNIndex) {
            let idx = idx.0;
            let offset = HALF_DIMS * idx;

            let mut accum = &mut self.accum.accum[persp][..HALF_DIMS];
            let weights = &self.weights[offset..offset + HALF_DIMS];

            assert!(accum.len() == HALF_DIMS);
            assert!(weights.len() == HALF_DIMS);

            for j in 0..HALF_DIMS {
                accum[j] += weights[j];
            }
            for k in 0..Self::PSQT_BUCKETS {
                // self.accum.psqt[persp][k] += self.psqt_weights[idx * Self::PSQT_BUCKETS + k];
                if let Some(x) = self.psqt_weights.get(idx * Self::PSQT_BUCKETS + k) {
                    self.accum.psqt[persp][k] += *x;
                }
            }
        }

        // #[cfg(feature = "nope")]
        pub fn _accum_rem(&mut self, persp: Color, idx: NNIndex) {
            let idx = idx.0;
            let offset = HALF_DIMS * idx;

            let mut accum = &mut self.accum.accum[persp][..HALF_DIMS];
            let weights = &self.weights[offset..offset + HALF_DIMS];

            assert!(accum.len() == HALF_DIMS);
            assert!(weights.len() == HALF_DIMS);

            // for (j,a) in accum.iter_mut().enumerate() {
            //     *a -= weights[j];
            // }

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

        /// temp no simd
        pub fn reset_accum(&mut self, g: &Game) {
            self._update_accum(g, White);
            self._update_accum(g, Black);
        }

        // pub fn reset_accum(&mut self, g: &Game) {
        //     #[cfg(not(target_feature = "avx2"))]
        //     self._update_accum(g, White);
        //     #[cfg(not(target_feature = "avx2"))]
        //     self._update_accum(g, Black);
        //     #[cfg(target_feature = "avx2")]
        //     self._update_accum_simd(g, White);
        //     #[cfg(target_feature = "avx2")]
        //     self._update_accum_simd(g, Black);
        // }

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

}
