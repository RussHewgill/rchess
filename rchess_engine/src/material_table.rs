
pub use self::mat_eval::*;
pub use self::pawn_eval::*;

use crate::endgame::*;
use crate::endgame::helpers::is_kx_vs_k;
use crate::types::*;
use crate::tables::*;

use std::collections::HashMap;

// #[derive(Debug,Clone)]
// pub struct MaterialTable {
//     max_entries:  usize,
//     table:        HashMap<Zobrist, MatEval>
// }

// impl MaterialTable {
//     const DEFAULT_SIZE_MB: usize = 32;
//     pub fn new(max_size_mb: usize) -> Self {
//         let max_entries = max_size_mb * 1024 * 1024;
//         Self {
//             max_entries,
//             table:        HashMap::with_capacity(max_entries),
//         }
//     }
// }

// impl MaterialTable {
//     pub fn prune(&mut self) {
//         unimplemented!()
//     }
//     pub fn inner(&self) -> &HashMap<Zobrist, MatEval> {
//         &self.table
//     }
//     pub fn get(&self, zb: Zobrist) -> Option<&MatEval> {
//         self.table.get(&zb)
//     }
//     pub fn insert(&mut self, zb: Zobrist, v: MatEval) {
//         self.table.insert(zb, v);
//     }
// }

const LOW_FOUR_BYTES: u64 = 0x00_00_00_00_FF_FF_FF_FF;

pub type MaterialTable = VecTable<MatEval, 32>;
pub type PawnTable     = VecTable<PawnEval, 8>;

#[derive(Debug,Clone)]
pub struct VecTable<T, const SIZE_MB: usize> where T: Clone {
    vec:        Vec<Option<(u32,T)>>,
}

impl<T: Clone, const SIZE_MB: usize> Default for VecTable<T, SIZE_MB> {
    fn default() -> Self { Self::new(SIZE_MB) }
}

impl<T: Clone, const SIZE_MB: usize> VecTable<T, SIZE_MB> {
    // const DEFAULT_SIZE_MB: usize = 32;

    pub fn new(max_size_mb: usize) -> Self {
        let max_entries = max_size_mb * 1024 * 1024;
        Self {
            vec:     vec![None; max_entries],
        }
    }
}

// #[derive(Debug,Clone)]
// pub struct MaterialTable {
//     vec:        Vec<Option<(u32,MatEval)>>,
// }

// impl Default for MaterialTable {
//     fn default() -> Self { Self::new(Self::DEFAULT_SIZE_MB) }
// }

impl<T: Clone, const SIZE_MB: usize> VecTable<T, SIZE_MB> {

    pub fn used_entries(&self) -> usize { self.vec.iter().flatten().count() }

    pub fn capacity(&self) -> usize { self.vec.len() }

    pub fn calc_index(&self, zb: Zobrist) -> (usize, u32) {
        let key = (zb.0 as u128 * self.vec.len() as u128).overflowing_shr(64).0;
        let ver = (zb.0 & LOW_FOUR_BYTES) as u32;
        (key as usize, ver)
    }

    pub fn get(&self, zb: Zobrist) -> Option<&T> {
        let (idx,ver) = self.calc_index(zb);
        if let Some((ver2, ev)) = self.vec.get(idx)? {
            if ver == *ver2 {
                return Some(ev);
            }
        }
        None
    }

    pub fn get_mut(&mut self, zb: Zobrist) -> Option<&mut T> {
        let (idx,ver) = self.calc_index(zb);
        if let Some((ver2, ev)) = self.vec.get_mut(idx)? {
            if ver == *ver2 {
                return Some(ev);
            }
        }
        None
    }

    /// returns true if entry was overwritten
    pub fn insert(&mut self, zb: Zobrist, v: T) -> bool {
        let (idx,ver) = self.calc_index(zb);
        if let Some(mut e) = self.vec.get_mut(idx) {
            let replaced = e.is_some();
            *e = Some((ver,v));
            replaced
        } else {
            unreachable!();
        }
    }

}

mod mat_eval {
    use crate::endgame::*;
    use crate::endgame::helpers::is_kx_vs_k;
    use crate::evaluate::TaperedScore;
    use crate::types::*;
    use crate::tables::*;

    use super::MaterialTable;

    #[derive(Debug,Clone,Copy)]
    /// Score is only the material balance
    pub struct MatEval {

        pub material_score: TaperedScore,
        pub phase:          Phase,

        // pub scaling_func:   

        pub eg_val:         Option<EndGameType>,

    }

    impl MaterialTable {
        pub fn get_or_insert(&mut self, ts: &Tables, g: &Game) -> MatEval {
            if let Some(me) = self.get(g.zobrist) {
                return *me;
            }

            let me = MatEval::new(ts, g);

            self.insert(g.zobrist, me);

            me
        }
    }

    impl MatEval {

        /// TODO: sf quadratics?
        #[cfg(feature = "nope")]
        pub fn imbalance(mat: &Material, side: Color) -> Score {
            // let mut score = 0;
            // for pc in Piece::iter_nonking_pieces() {
            //     let n = mat.get(pc, side);
            //     if n == 0 { continue; }
            //     score += n as Score * pc.score();
            // }
            // score
            unimplemented!()
        }

        pub fn imbalance(g: &Game) -> TaperedScore {
            g.state.npm[White]
                + Pawn.score_tapered() * g.state.material.get(Pawn, White) as Score
                - g.state.npm[Black]
                - Pawn.score_tapered() * g.state.material.get(Pawn, Black) as Score
        }

        pub fn new(ts: &Tables, g: &Game) -> Self {

            // let score = g.sum_evaluate(ts, &ts.eval_params_mid, &ts.eval_params_mid, None);
            let mut material_score = Self::imbalance(g);

            // if is_kx_vs_k(g, g.state.side_to_move) {
            //     unimplemented!()
            // }

            // if g.state.npm[White] + g.state.npm[Black] == 0 && g.state.material.get_both(Pawn) > 0 {
            //     unimplemented!()
            // }

            let eg_val = None;

            Self {
                material_score,
                phase:     g.state.phase,
                eg_val,
            }

        }

        // pub fn new(g: &Game, score: Score) -> Self {
        //     Self {
        //         score,
        //         phase:     g.state.phase,
        //         // factor:    [ScaleFactor::Normal; 2],
        //         eg_val:    None,
        //         // eg_scale:  None,
        //     }
        // }

    }

}

mod pawn_eval {
    use crate::types::*;
    use crate::tables::*;
    use crate::evaluate::TaperedScore;

    use super::PawnTable;

    #[derive(Debug,Clone,Copy)]
    pub struct PawnEval {
        pub scores:          [TaperedScore; 2],
        pub passed:          BitBoard,
        pub attacks:         BitBoard,
        pub attacks_span:    BitBoard,
    }

    impl PawnTable {
        pub fn get_or_insert(&mut self, ts: &Tables, g: &Game) -> PawnEval {
            if let Some(ev) = self.get(g.zobrist) {
                return *ev;
            }

            let ev = PawnEval::new(ts, g);

            self.insert(g.zobrist, ev);

            ev
        }
    }

    impl PawnEval {
        pub fn new(ts: &Tables, g: &Game) -> Self {

            let mut passed       = BitBoard::empty();
            let mut attacks      = BitBoard::empty();
            let mut attacks_span = BitBoard::empty();

            let mut out = Self {
                scores: [TaperedScore::default(); 2],
                passed,
                attacks,
                attacks_span,
            };

            out.evaluate(ts, g, White);
            out.evaluate(ts, g, Black);

            out
        }

        fn evaluate(&mut self, ts: &Tables, g: &Game, side: Color) -> TaperedScore {

            let score = TaperedScore::default();

            let pawns_us   = g.get(Pawn, side);
            let pawns_them = g.get(Pawn, !side);

            for sq in pawns_us.into_iter() {

                let r = BitBoard::relative_rank(side, sq);

                let opposed = pawns_them & forward_file_bb(side, sq);

                // unimplemented!()
            }

            score
        }

    }

    /// Pawn Spans
    impl Game {

        pub fn pawn_attacks_span(&self, side: Color) -> BitBoard {
            let (d,dw,de) = if side == White { (N,NW,NE) } else { (S,SW,SE) };
            let pawns = self.get(Pawn, side);
            pawns.shift_dir(dw) | pawns.shift_dir(de)
        }

        pub fn _pawn_attacks_span(bb: BitBoard, side: Color) -> BitBoard {
            let (d,dw,de) = if side == White { (N,NW,NE) } else { (S,SW,SE) };
            bb.shift_dir(dw) | bb.shift_dir(de)
        }

    }

}


