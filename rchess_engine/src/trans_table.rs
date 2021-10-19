
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;

use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use evmap::{ReadHandle,WriteHandle};
// use evmap_derive::ShallowCopy;
// use rustc_hash::Fx;

#[derive(Debug,Default)]
// pub struct RwTransTable(pub RwLock<TransTable>, pub RwLock<TTStats>);
pub struct RwTransTable(pub RwLock<TransTable>);

#[derive(Debug,Default)]
pub struct TransTable {
    pub map:    FxHashMap<Zobrist, SearchInfo>,
    // pub map_r:    ReadHandle<Zobrist, SearchInfo>,
    // pub map_w:    WriteHandle<Zobrist, SearchInfo>,
    // pub hits:   u32,
    // pub misses: u32,
}

// impl Default for TransTable {
//     fn default() -> Self {
//         let (r,w) = evmap::Options::default()
//             // .with_hasher()
//             .construct();
//         Self {
//             map_r: r,
//             map_w: w,
//         }
//         // unimplemented!()
//     }
// }

#[derive(Debug,Default)]
pub struct TTStats {
    pub hits:    u32,
    pub misses:  u32,
    pub leaves:  u32,
}

// #[derive(Debug,Eq,PartialEq,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
#[derive(Debug,Eq,PartialEq,PartialOrd,Hash,Clone,Copy)]
pub struct SearchInfo {
    pub mv:                 Move,
    pub depth_searched:     Depth,
    // pub score:              Score,
    pub score:              Node,
    // pub eval:               Eval,

    // pub best_move:          Move,
    // pub refutation_move:    Move,
    // pub pv:                 Move,
    // pub score:              NodeType,
    // pub age:                Duration, // or # of moves?
}

// #[derive(Debug,Eq,PartialEq,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
#[derive(Debug,Eq,PartialEq,PartialOrd,Hash,Clone,Copy)]
pub enum Node {
    PV(Score),
    UpperBound(Score), // NodeAll
    LowerBound(Score), // NodeCut
    // NodeAll(Score), // Score = upper bound
    // NodeCut(Score), // Score = lower bound
}

impl RwTransTable {

    // pub fn new() -> Self {
    //     // RwTransTable(RwLock::new(TransTable::default()), RwLock::new(Default::default()))
    //     // let m = evmap::new();
    //     unimplemented!()
    // }

    /// Always replace
    /// Returns false if new insert, true if replace
    pub fn insert_replace(&self, zb: Zobrist, search: SearchInfo) -> bool {
        self.with_mut(|m| {
            let b = m.map.contains_key(&zb);
            m.map.insert(zb, search);
            b
        })
    }

    pub fn get(&self, zb: &Zobrist) -> Option<SearchInfo> {
        self.with(|m| {
            let s: Option<&SearchInfo> = m.map.get(&zb);
            s.copied()
        })
    }

    pub fn clear(&self) {
        // self.with_stats_mut(|s| {
        //     *s = TTStats::default();
        // });
        self.with_mut(|m| {
            m.map.clear();
        });
        // unimplemented!()
    }

    pub fn with<F,T>(&self, mut f: F) -> T
    where
        F: FnOnce(&TransTable) -> T,
        T: Copy,
    {
        let r = self.0.read();
        let s = f(&r);
        s
        // unimplemented!()
    }

    pub fn with_mut<F, T>(&self, mut f: F) -> T
    where
        F: FnMut(&mut TransTable) -> T {
        {
            let mut w = self.0.write();
            f(&mut w)
        }
    }

    // pub fn with_stats<F,T>(&self, mut f: F) -> T
    // where
    //     F: FnOnce(&TTStats) -> T,
    //     T: Copy,
    // {
    //     let r = self.1.read();
    //     let s = f(&r);
    //     s
    // }

    // pub fn with_stats_mut<F, T>(&self, mut f: F) -> T
    // where
    //     F: FnMut(&mut TTStats) -> T {
    //     {
    //         let mut w = self.1.write();
    //         f(&mut w)
    //     }
    // }

}

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

    pub fn new(mv: Move, depth_searched: Depth, score: Node) -> Self {
        Self {
            mv,
            depth_searched,
            score,
            // node_type,
            // ..Default::default()
        }
    }

    pub fn score(&self) -> Score {
        self.score.score()
    }

}

impl Node {
    pub fn score(&self) -> Score {
        match *self {
            Node::PV(s)         => s,
            Node::UpperBound(s) => s,
            Node::LowerBound(s) => s,
        }
    }
}

