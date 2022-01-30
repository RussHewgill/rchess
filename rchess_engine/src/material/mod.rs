
pub mod mat_table;
pub mod pawn_table;

pub use mat_table::*;
pub use pawn_table::*;

use crate::endgame::*;
use crate::endgame::helpers::is_kx_vs_k;
use crate::types::*;
use crate::tables::*;

const LOW_FOUR_BYTES: u64 = 0x00_00_00_00_FF_FF_FF_FF;

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


