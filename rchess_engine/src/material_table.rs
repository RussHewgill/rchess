
use std::collections::HashMap;

use crate::endgame::*;
use crate::types::*;
use crate::tables::*;

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

#[derive(Debug,Clone)]
pub struct MaterialTable {
    vec:        Vec<Option<(u32,MatEval)>>,
}

impl Default for MaterialTable {
    fn default() -> Self { Self::new(Self::DEFAULT_SIZE_MB) }
}

impl MaterialTable {

    const DEFAULT_SIZE_MB: usize = 32;

    pub fn new(max_size_mb: usize) -> Self {
        let max_entries = max_size_mb * 1024 * 1024;
        Self {
            vec:     vec![None; max_entries],
        }
    }
}

impl MaterialTable {

    pub fn used_entries(&self) -> usize { self.vec.iter().flatten().count() }

    pub fn capacity(&self) -> usize { self.vec.len() }

    pub fn calc_index(&self, zb: Zobrist) -> (usize, u32) {
        let key = (zb.0 as u128 * self.vec.len() as u128).overflowing_shr(64).0;
        let ver = (zb.0 & LOW_FOUR_BYTES) as u32;
        (key as usize, ver)
    }

    pub fn get(&self, zb: Zobrist) -> Option<&MatEval> {

        let (idx,ver) = self.calc_index(zb);

        if let Some((ver2, ev)) = self.vec.get(idx)? {
            if ver == *ver2 {
                return Some(ev);
            }
        }
        None
    }

    /// returns true if entry was overwritten
    pub fn insert(&mut self, zb: Zobrist, ev: MatEval) -> bool {
        let (idx,ver) = self.calc_index(zb);
        if let Some(mut e) = self.vec.get_mut(idx) {
            let replaced = e.is_some();
            *e = Some((ver,ev));
            replaced
        } else {
            unreachable!();
        }
    }

}

#[derive(Debug,Clone)]
pub struct MatEval {
    pub score:     Score,
    pub phase:     Phase,
    // factor:    [ScaleFactor; 2],

    // eg_val:    Option<Box<dyn EndGame>>,
    // eg_scale:  Option<[Box<dyn EndGame>; 2]>,
    pub eg_val:    Option<EndGameType>,
    // eg_scale:  Option<[EndGameType; 2]>,

}

impl MatEval {
    pub fn new(g: &Game, score: Score) -> Self {
        Self {
            score,
            phase:     g.state.phase,
            // factor:    [ScaleFactor::Normal; 2],

            eg_val:    None,
            // eg_scale:  None,

        }
    }
}

// impl Evaluation {
//     pub fn score(&self) -> Score { self.score }
//     pub fn phase(&self) -> Phase { self.phase }
// }




