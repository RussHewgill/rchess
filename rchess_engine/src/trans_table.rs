
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;

use parking_lot::RwLock;
use rustc_hash::FxHashMap;

#[derive(Debug,Default)]
pub struct RwTransTable(pub RwLock<TransTable>, pub RwLock<TTStats>);

#[derive(Debug,Default)]
pub struct TransTable {
    pub map:    FxHashMap<Zobrist, SearchInfo>,
    // pub hits:   u32,
    // pub misses: u32,
}

#[derive(Debug,Default)]
pub struct TTStats {
    pub hits:    u32,
    pub misses:  u32,
    pub leaves:  u32,
}

#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub struct SearchInfo {
    pub mv:                 Move,
    pub depth_searched:     u32,
    // pub score:              Score,
    pub score:              Node,
    // pub eval:               Eval,

    // pub best_move:          Move,
    // pub refutation_move:    Move,
    // pub pv:                 Move,
    // pub score:              NodeType,
    // pub age:                Duration, // or # of moves?
}

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
    //     RwTransTable(RwLock::new(TransTable::default()), RwLock::new(Default::default()))
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
        self.with_stats_mut(|s| {
            *s = TTStats::default();
        });
        self.with_mut(|m| {
            m.map.clear();
        });
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

    pub fn with_stats<F,T>(&self, mut f: F) -> T
    where
        F: FnOnce(&TTStats) -> T,
        T: Copy,
    {
        let r = self.1.read();
        let s = f(&r);
        s
    }

    pub fn with_stats_mut<F, T>(&self, mut f: F) -> T
    where
        F: FnMut(&mut TTStats) -> T {
        {
            let mut w = self.1.write();
            f(&mut w)
        }
    }

}

impl RwTransTable {

    pub fn inc_hits(&self) { self.with_stats_mut(|s| s.hits += 1) }
    pub fn inc_misses(&self) { self.with_stats_mut(|s| s.misses += 1) }
    pub fn inc_leaves(&self) { self.with_stats_mut(|s| s.leaves += 1) }

    pub fn hits(&self) -> u32 { self.with_stats(|s| s.hits) }
    pub fn misses(&self) -> u32 { self.with_stats(|s| s.misses) }
    pub fn leaves(&self) -> u32 { self.with_stats(|s| s.leaves) }
}

impl SearchInfo {
    // pub fn new(depth_searched: u32, evaluation: Score, node_type: i32) -> Self {
    // pub fn new(pv: Move, depth_searched: u32, score: NodeType) -> Self {
    // pub fn new(depth_searched: u32, score: NodeType) -> Self {

    pub fn new(mv: Move, depth_searched: u32, score: Node) -> Self {
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

