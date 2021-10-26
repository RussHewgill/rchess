
use crate::explore::Explorer;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;

// use arrayvec::ArrayVec;
use parking_lot::{RwLock,Mutex};
use rustc_hash::FxHashMap;
use std::hash::Hasher;
use std::sync::Arc;

use std::hash::Hash;

use evmap::{ReadHandle,WriteHandle};
use evmap_derive::ShallowCopy;
// use rustc_hash::Fx;
use dashmap::{DashMap,DashSet};

pub type TTRead  = ReadHandle<Zobrist, SearchInfo>;
pub type TTWrite = Arc<Mutex<WriteHandle<Zobrist, SearchInfo>>>;

// pub type TransTable = FxHashMap<Zobrist, SearchInfo>;
pub type TransTable = Arc<DashMap<Zobrist, SearchInfo>>;

#[derive(Debug,Default,Clone)]
pub struct MvTable {
    set: DashSet<u64>,
}

impl MvTable {

    fn make_key(depth: Depth, zb: Zobrist, mv: Move) -> u64 {
        let mut out = 0;
        out |= zb.0;
        out |= depth as u64;
        let m = {
            let mut h = rustc_hash::FxHasher::default();
            mv.hash(&mut h);
            h.finish()
        };
        out |= m;
        out
    }

    /// Returns true if key was already in set
    pub fn insert(&self, depth: Depth, zb: Zobrist, mv: Move) -> bool {
        self.set.insert(Self::make_key(depth, zb, mv))
    }

    pub fn remove(&self, depth: Depth, zb: Zobrist, mv: Move) {
        self.set.remove(&Self::make_key(depth, zb, mv));
    }

    pub fn contains(&self, depth: Depth, zb: Zobrist, mv: Move) -> bool {
        self.set.contains(&Self::make_key(depth, zb, mv))
    }

}

impl Explorer {

    // XXX: clear old values?
    pub fn tt_insert_deepest(
        tt_r: &TTRead, tt_w: TTWrite, zb: Zobrist, si: SearchInfo) -> bool {

        // if si.depth_searched as usize != si.moves.len() {
        //     eprintln!("si = {:?}", si);
        // }

        // let w = tt_w.lock();

        let d  = si.depth_searched;
        let nt = si.node_type;

        if let Some(prevs) = tt_r.get(&zb) {
            if let Some(prev_si) = prevs.into_iter().max_by(|a,b| a.depth_searched.cmp(&b.depth_searched)) {
                if d < prev_si.depth_searched || (prev_si.node_type != Node::PV && nt == Node::PV) {
                    return true;
                }
            }
        }

        // let mut clear = false;
        // if let Some(prev_si) = tt_r.get_one(&zb) {
        //     if d < prev_si.depth_searched || (prev_si.node_type != Node::PV && nt == Node::PV) {
        //         return true;
        //     }
        //     clear = true;
        // }

        {
            let mut w = tt_w.lock();
            // if clear { w.clear(zb); }
            // w.remove(&zb, prev_si);
            w.insert(zb, si);
            w.refresh();
        }

        // if let Some(prev_si) = w.insert(zb, si) {
        //     if d < prev_si.depth_searched {
        //         self.trans_table.insert(zb, prev_si);
        //         return true;
        //     } else if prev_si.node_type != Node::PV && nt == Node::PV {
        //         self.trans_table.insert(zb, prev_si);
        //         return true;
        //     }
        // }

        false
    }

    // pub fn tt_insert_deepest(&self, zb: Zobrist, si: SearchInfo) -> bool {
    //     // if si.depth_searched as usize != si.moves.len() {
    //     //     eprintln!("si = {:?}", si);
    //     // }
    //     // let d = si.depth_searched;
    //     // let nt = si.node_type;
    //     // if let Some(prev_si) = self.trans_table.insert(zb, si) {
    //     //     if d < prev_si.depth_searched {
    //     //         self.trans_table.insert(zb, prev_si);
    //     //         return true;
    //     //     } else if prev_si.node_type != Node::PV && nt == Node::PV {
    //     //         self.trans_table.insert(zb, prev_si);
    //     //         return true;
    //     //     }
    //     // }
    //     // false
    // }

    // pub fn tt_contains(&self, zb: &Zobrist) -> bool {
    //     // self.trans_table.contains_key(&zb)
    //     unimplemented!()
    // }

}

#[derive(Debug,Default)]
pub struct TTStats {
    pub hits:    u32,
    pub misses:  u32,
    pub leaves:  u32,
}

#[derive(Debug,Eq,PartialEq,Clone,Copy)]
pub enum SICanUse {
    UseScore,
    UseOrdering
}

// #[derive(Debug,Eq,PartialEq,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
// #[derive(Debug,Eq,PartialEq,PartialOrd,Hash,Clone,Copy)]
#[derive(Debug,Eq,PartialEq,PartialOrd,Hash,ShallowCopy,Clone)]
pub struct SearchInfo {
    pub mv:                 Move,
    pub depth_searched:     Depth,
    // pub score:              Score,
    pub node_type:          Node,
    pub score:              Score,
    // pub eval:               Eval,

    pub moves:              Vec<Move>,
    // pub moves:              ArrayVec<Move, 100>,

    // pub best_move:          Move,
    // pub refutation_move:    Move,
    // pub pv:                 Move,
    // pub score:              NodeType,
    // pub age:                Duration, // or # of moves?
}

/// PV,
/// All, // UpperBound
/// Cut, // LowerBound
/// Quiet,
#[derive(Debug,Eq,PartialEq,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
// #[derive(Debug,Eq,PartialEq,PartialOrd,Hash,Clone,Copy)]
pub enum Node {
    PV,
    All, // UpperBound
    Cut, // LowerBound
    Quiet, // XXX: ?
    Root, // XXX: ??
    // NodeAll(Score), // Score = upper bound
    // NodeCut(Score), // Score = lower bound
}

// impl RwTransTable {

//     // pub fn new() -> Self {
//     //     // RwTransTable(RwLock::new(TransTable::default()), RwLock::new(Default::default()))
//     //     // let m = evmap::new();
//     //     unimplemented!()
//     // }

//     pub fn clear(&self) {
//         self.tt_with_mut(|m| {
//             m.clear();
//         });
//     }

//     /// Always replace
//     /// Returns false if new insert, true if replace
//     pub fn tt_insert_replace(&self, zb: Zobrist, search: SearchInfo) -> bool {
//         self.tt_with_mut(|m| {
//             let b = m.contains_key(&zb);
//             m.insert(zb, search);
//             b
//         })
//     }

//     pub fn tt_get(&self, zb: &Zobrist) -> Option<SearchInfo> {
//         self.tt_with(|m| {
//             let s: Option<&SearchInfo> = m.get(&zb);
//             s.copied()
//         })
//     }

//     /// Always replace
//     /// Returns false if new insert, true if replace
//     pub fn qt_insert_replace(&self, zb: Zobrist, search: SearchInfo) -> bool {
//         self.qt_with_mut(|m| {
//             let b = m.contains_key(&zb);
//             m.insert(zb, search);
//             b
//         })
//     }

//     pub fn qt_get(&self, zb: &Zobrist) -> Option<SearchInfo> {
//         self.qt_with(|m| {
//             let s: Option<&SearchInfo> = m.get(&zb);
//             s.copied()
//         })
//     }

//     pub fn tt_with<F,T>(&self, mut f: F) -> T
//     where
//         F: FnOnce(&TransTable) -> T,
//         T: Copy,
//     {
//         let r = self.trans_table.read();
//         let s = f(&r);
//         s
//     }

//     pub fn tt_with_mut<F, T>(&self, mut f: F) -> T
//     where
//         F: FnMut(&mut TransTable) -> T {
//         {
//             let mut w = self.trans_table.write();
//             f(&mut w)
//         }
//     }

//     pub fn qt_with<F,T>(&self, mut f: F) -> T
//     where
//         F: FnOnce(&TransTable) -> T,
//         T: Copy,
//     {
//         let r = self.quiescent.read();
//         let s = f(&r);
//         s
//     }

//     pub fn qt_with_mut<F, T>(&self, mut f: F) -> T
//     where
//         F: FnMut(&mut TransTable) -> T {
//         {
//             let mut w = self.quiescent.write();
//             f(&mut w)
//         }
//     }

//     // pub fn with_stats<F,T>(&self, mut f: F) -> T
//     // where
//     //     F: FnOnce(&TTStats) -> T,
//     //     T: Copy,
//     // {
//     //     let r = self.1.read();
//     //     let s = f(&r);
//     //     s
//     // }

//     // pub fn with_stats_mut<F, T>(&self, mut f: F) -> T
//     // where
//     //     F: FnMut(&mut TTStats) -> T {
//     //     {
//     //         let mut w = self.1.write();
//     //         f(&mut w)
//     //     }
//     // }

// }

// impl RwTransTable {
//     pub fn inc_hits(&self) { self.with_stats_mut(|s| s.hits += 1) }
//     pub fn inc_misses(&self) { self.with_stats_mut(|s| s.misses += 1) }
//     pub fn inc_leaves(&self) { self.with_stats_mut(|s| s.leaves += 1) }
//     pub fn hits(&self) -> u32 { self.with_stats(|s| s.hits) }
//     pub fn misses(&self) -> u32 { self.with_stats(|s| s.misses) }
//     pub fn leaves(&self) -> u32 { self.with_stats(|s| s.leaves) }
// }

impl SearchInfo {
    // pub fn new(depth_searched: u32, evaluation: Score, node_type: i32) -> Self {
    // pub fn new(pv: Move, depth_searched: u32, score: NodeType) -> Self {
    // pub fn new(depth_searched: u32, score: NodeType) -> Self {

    // pub fn new(mv: Move, depth_searched: Depth, node_type: Node, score: Score) -> Self {
    pub fn new(mv: Move, moves: Vec<Move>, depth_searched: Depth, node_type: Node, score: Score) -> Self {
        // let mut mvs = ArrayVec::new();
        // mvs.try_extend_from_slice(&moves).unwrap();
        Self {
            mv,
            depth_searched,
            node_type,
            score,
            moves,

            // ..Default::default()
        }
    }

    // pub fn score(&self) -> Score {
    //     self.score.score()
    // }

}

// impl Node {
//     pub fn score(&self) -> Score {
//         match *self {
//             Node::PV(s)         => s,
//             Node::UpperBound(s) => s,
//             Node::LowerBound(s) => s,
//         }
//     }
// }

// impl std::cmp::PartialEq for SICanUse {
//     fn eq(&self, other: Self) -> bool {
//     }
// }

impl std::cmp::PartialOrd for SICanUse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}


