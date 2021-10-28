
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

/// PV,  // Exact
/// All, // UpperBound, Fail low
/// Cut, // LowerBound, Fail high
/// Quiet,
/// // Root,
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

    #[allow(unused_doc_comments)]
    // XXX: clear old values?
    pub fn tt_insert_deepest(
        tt_r: &TTRead, tt_w: TTWrite, zb: Zobrist, si: SearchInfo) -> bool {

        let d  = si.depth_searched;
        let nt = si.node_type;

        if let Some(prevs) = tt_r.get(&zb) {
            if let Some(prev_si) = prevs.into_iter().max_by(|a,b| a.depth_searched.cmp(&b.depth_searched)) {
                if d < prev_si.depth_searched || (prev_si.node_type != Node::PV && nt == Node::PV) {
                    /// Value already in map is better, keep that instead
                    return true;
                }
            }
        }

        {
            let mut w = tt_w.lock();
            // w.clear(zb);
            // w.insert(zb, si);
            w.update(zb, si);
            w.refresh();
            // w.flush();
        }

        false
    }

}


impl SearchInfo {
    pub fn new(mv: Move, moves: Vec<Move>, depth_searched: Depth, node_type: Node, score: Score) -> Self {
        Self {
            mv,
            depth_searched,
            node_type,
            score,
            moves,
            // ..Default::default()
        }
    }
}

impl std::cmp::PartialOrd for SICanUse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}


