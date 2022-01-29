
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

pub type MaterialTable = VecTable<MatEval>;
pub type PawnTable     = VecTable<PawnEval>;

#[derive(Debug,Clone)]
pub struct VecTable<T> where T: Clone {
    vec:        Vec<Option<(u32,T)>>,
}

impl<T: Clone> Default for VecTable<T> {
    fn default() -> Self { Self::new(Self::DEFAULT_SIZE_MB) }
}

impl<T: Clone> VecTable<T> {
    const DEFAULT_SIZE_MB: usize = 32;
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

impl<T: Clone> VecTable<T> {

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
    use crate::types::*;
    use crate::tables::*;

    #[derive(Debug,Clone,Copy)]
    pub struct MatEval {

        pub score:          Score,
        pub phase:          Phase,

        // pub scaling_func:   

        pub eg_val:         Option<EndGameType>,

    }

    impl MatEval {

        pub fn new(ts: &Tables, g: &Game) -> Self {

            let score = g.sum_evaluate(ts, &ts.eval_params_mid, &ts.eval_params_mid, None);

            if is_kx_vs_k(g, g.state.side_to_move) {
                unimplemented!()
            }

            if g.state.npm[White] + g.state.npm[Black] == 0 && g.state.material.get_both(Pawn) > 0 {
                unimplemented!()
            }

            let eg_val = None;

            Self {
                score,
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

    #[derive(Debug,Clone,Copy)]
    pub struct PawnEval {
    }

}

// impl Evaluation {
//     pub fn score(&self) -> Score { self.score }
//     pub fn phase(&self) -> Phase { self.phase }
// }




