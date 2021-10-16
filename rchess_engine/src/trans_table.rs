
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;

use rustc_hash::FxHashMap;

#[derive(Debug,Default,PartialEq,Clone)]
pub struct TransTable(FxHashMap<Zobrist, SearchInfo>);

impl TransTable {
    pub fn new() -> Self {
        Self(FxHashMap::default())
    }

    /// Always replace
    pub fn insert(&mut self, zb: Zobrist, search: SearchInfo) {
        self.0.insert(zb, search);
    }

    pub fn get(&self, zb: Zobrist) -> Option<&SearchInfo> {
        self.0.get(&zb)
    }

}

#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub struct SearchInfo {
    // pub best_move:          Move,
    // pub refutation_move:    Move,
    // pub pv:                 Move,
    pub depth_searched:     u32,
    pub score:              NodeType,
    // pub age:                Duration, // or # of moves?
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Hash,Clone,Copy)]
pub enum NodeType {
    NodePV(Score),
    NodeUpperBound(Score), // NodeAll
    NodeLowerBound(Score), // NodeCut
    // NodeAll(Score), // Score = upper bound
    // NodeCut(Score), // Score = lower bound
}

impl SearchInfo {
    // pub fn new(depth_searched: u32, evaluation: Score, node_type: i32) -> Self {
    // pub fn new(pv: Move, depth_searched: u32, score: NodeType) -> Self {
    pub fn new(depth_searched: u32, score: NodeType) -> Self {
        Self {
            // pv,
            depth_searched,
            score,
            // node_type,
        }
    }
}

impl NodeType {
    pub fn score(&self) -> Score {
        match *self {
            NodeType::NodePV(s)         => s,
            NodeType::NodeUpperBound(s) => s,
            NodeType::NodeLowerBound(s) => s,
        }
    }
}

