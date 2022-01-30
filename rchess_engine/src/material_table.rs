
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

mod mat_eval {
}


