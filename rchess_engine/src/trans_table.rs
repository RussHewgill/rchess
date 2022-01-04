
use crate::explore::*;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::evmap_tables::*;

use evmap::shallow_copy::CopyValue;
// use arrayvec::ArrayVec;
use parking_lot::{RwLock,Mutex};
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::hash::Hasher;
use std::path::Path;
use std::sync::Arc;

use std::hash::Hash;

use serde::{Serialize,Deserialize};

use evmap::{ReadHandle,ReadHandleFactory,WriteHandle};
use evmap_derive::ShallowCopy;
// use rustc_hash::Fx;
// use dashmap::{DashMap,DashSet};

// pub type TTReadFactory  = ReadHandleFactory<Zobrist, SearchInfo, (), FxBuildHasher>;
// pub type TTRead  = ReadHandle<Zobrist, SearchInfo, (), FxBuildHasher>;
// pub type TTWrite = Arc<Mutex<WriteHandle<Zobrist, SearchInfo, (), FxBuildHasher>>>;

pub type TTReadFactory = EVReadFactory<SearchInfo>;
pub type TTRead        = EVRead<SearchInfo>;
pub type TTWrite       = EVWrite<SearchInfo>;

// pub type TTRead  = ReadHandle<Zobrist, SearchInfo>;
// pub type TTWrite = Arc<Mutex<WriteHandle<Zobrist, SearchInfo>>>;

// pub type TransTable = FxHashMap<Zobrist, SearchInfo>;
// pub type TransTable = Arc<DashMap<Zobrist, SearchInfo>>;

// #[derive(Debug,Default,Clone)]
// pub struct MvTable {
//     set: DashSet<u64>,
// }

// #[derive(Debug,Default)]
// pub struct TTStats {
//     pub hits:    u32,
//     pub misses:  u32,
//     pub leaves:  u32,
// }

impl Explorer {
    #[cfg(not(feature = "lockless_hashmap"))]
    pub fn handle(&self) -> TTRead {
        self.tt_rf.handle()
    }
}

pub fn tt_total_size(tt_r: &TTRead) -> usize {
    let mut out = 0;
    for (k,vs) in tt_r.read().unwrap().iter() {
        out += std::mem::size_of_val(k);
        for v in vs.iter() {
            out += std::mem::size_of_val(v);
        }
    }
    out
}

pub fn save_tt<P: AsRef<Path>>(tt_r: &TTRead, path: P) -> std::io::Result<()> {
    use std::io::Write;
    let mut out: HashMap<Zobrist,SearchInfo> = HashMap::default();
    for (k,vs) in tt_r.read().unwrap().iter() {
        for v in vs.iter() {
            if let Some(_) = out.insert(*k, *v) {
                println!("save_tt: dupe: {:?}", k);
            }
        }
    }
    let b: Vec<u8> = bincode::serialize(&out).unwrap();
    let mut file = std::fs::File::create(path)?;
    file.write_all(&b)?;
    Ok(())
}

pub fn load_tt<P: AsRef<Path>>(
    path:    P,
    tt_w:    TTWrite,
) -> std::io::Result<()> {
    let map = _load_tt(path)?;

    let mut w = tt_w.lock();
    for (zb,si) in map.into_iter() {
        w.update(zb, si);
    }

    w.refresh();

    Ok(())
}

pub fn _load_tt<P: AsRef<Path>>(path: P) -> std::io::Result<HashMap<Zobrist,SearchInfo>> {
    let mut b = std::fs::read(path)?;
    let out: HashMap<Zobrist,SearchInfo> = bincode::deserialize(&b).unwrap();
    Ok(out)
}

#[derive(Debug,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy)]
pub enum SICanUse {
    UseScore,
    UseOrdering,
}

#[derive(Debug,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy,Serialize,Deserialize)]
pub struct SearchInfo {
    pub best_move:          Move,            // 5
    // pub best_move:          (u8,u8),
    pub depth_searched:     Depth,           // 1
    pub node_type:          Node,            // 1
    pub score:              Score,           // 4
    // pub eval:               Option<Score>,   // 4, 8?
    // pub eval:               Score,   // 4, 8?
}

#[derive(Debug,Clone,Copy)]
pub struct PackedSearchInfo {
    pub best_move:          [u8; 2],
    pub depth_searched:     Depth,
    pub node_type:          Node,
    pub score:              u32,
}

/// Pack, unpack
impl PackedSearchInfo {
    pub fn pack(si: SearchInfo) -> Self {
        unimplemented!()
    }
    pub fn unpack(&self, ts: &'static Tables, g: &Game) -> Option<SearchInfo> {
        // let mv = PackedMove::unpack(&self.best_move).unwrap().convert_to_move(ts, g);
        unimplemented!()
    }
}

impl SearchInfo {
    pub fn empty() -> Self {
        Self {
            // best_move:      (0,0),
            best_move:      Move::NullMove,
            depth_searched: 0,
            node_type:      Node::Empty,
            score:          0,
            // eval:           None,
            // eval:           0,
        }
    }
}

/// New
impl SearchInfo {
    // pub fn new(mv: Move, moves: Vec<Move>, depth_searched: Depth, node_type: Node, score: Score) -> Self {
    //     let moves = VMoves::from_vec(moves).into();
    pub fn new(
        best_move:          Move,
        // moves:              Vec<Move>,
        depth_searched:     Depth,
        node_type:          Node,
        score:              Score,
        // eval:               Option<Score>,
    ) -> Self {

        // let packed_move = PackedMove::convert_from_move(best_move).pack().unwrap();
        // let packed_move = (packed_move[0],packed_move[1]);

        // let mv = [packed_move.0, packed_move.1];

        // let mv = PackedMove::unpack(&mv).unwrap().convert_to_move(ts, g);
        // assert_eq!(best_move, mv);

        Self {
            // best_move: packed_move,
            best_move,
            depth_searched,
            node_type,
            score,
            // eval,
            // eval: 0,
        }
    }
}

impl PartialOrd for SearchInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.score.cmp(&other.score))

        // use std::cmp::Ordering;
        // match (self.node_type,other.node_type) {
        //     (a,b) if a == b => Some(self.score.cmp(&other.score)),
        //     (Node::PV, _)   => Some(Ordering::Greater),
        //     (_, Node::PV)   => Some(Ordering::Less),
        //     _               => Some(self.score.cmp(&other.score)),
        // }

    }
}

/// PV,  // Exact
/// All, // UpperBound, Fail low, evaluation never exceeded alpha
/// Cut, // LowerBound, Fail high, evaluation caused cutoff
/// Quiet,
/// // Root,
// #[derive(Debug,Eq,PartialEq,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Hash,ShallowCopy,Clone,Copy,Serialize,Deserialize)]
pub enum Node {
    PV,
    All, // UpperBound
    Cut, // LowerBound
    // Quiet, // XXX: ?
    // Root, // XXX: ??
    // NodeAll(Score), // Score = upper bound
    // NodeCut(Score), // Score = lower bound
    Empty,
}

// impl MvTable {

//     fn make_key(depth: Depth, zb: Zobrist, mv: Move) -> u64 {
//         let mut out = 0;
//         out |= zb.0;
//         out |= depth as u64;
//         let m = {
//             let mut h = rustc_hash::FxHasher::default();
//             mv.hash(&mut h);
//             h.finish()
//         };
//         out |= m;
//         out
//     }

//     /// Returns true if key was already in set
//     pub fn insert(&self, depth: Depth, zb: Zobrist, mv: Move) -> bool {
//         self.set.insert(Self::make_key(depth, zb, mv))
//     }

//     pub fn remove(&self, depth: Depth, zb: Zobrist, mv: Move) {
//         self.set.remove(&Self::make_key(depth, zb, mv));
//     }

//     pub fn contains(&self, depth: Depth, zb: Zobrist, mv: Move) -> bool {
//         self.set.contains(&Self::make_key(depth, zb, mv))
//     }

// }

/// tt_insert_deepest
impl ExHelper {

    #[cfg(feature = "lockless_hashmap")]
    pub fn tt_insert_deepest(&self, zb: Zobrist, si: SearchInfo) {
        // trace!("inserting zb = {:?}, si = {:?}", zb, si);
        self.ptr_tt.insert(zb, si);
    }

    #[allow(unused_doc_comments)]
    #[cfg(not(feature = "lockless_hashmap"))]
    pub fn tt_insert_deepest(&self, zb: Zobrist, si: SearchInfo) -> bool {

        let d  = si.depth_searched;
        let nt = si.node_type;

        // if zb == Zobrist(0xaa5beb342615075b) {
        //     let r = self.best_mate.read();
        //     let s = self.stop.load(std::sync::atomic::Ordering::Relaxed);
        //     eprintln!("found zb1, si = {:?}, r = {:?}, s = {:?}", si.best_move, r, s);
        // }
        // if zb == Zobrist(0xdb13044b200db2b4) {
        //     let r = self.best_mate.read();
        //     let s = self.stop.load(std::sync::atomic::Ordering::Relaxed);
        //     eprintln!("found zb2, si = {:?}, r = {:?}, s = {:?}", si.best_move, r, s);
        // }

        if let Some(prev_si) = self.tt_r.get_one(&zb) {
            if d < prev_si.depth_searched {
                /// Value already in map is better, keep that instead
                return true;
            }
        }

        // if let Some(prevs) = tt_r.get(&zb) {
        //     if let Some(prev_si) = prevs.into_iter().max_by(|a,b| a.depth_searched.cmp(&b.depth_searched)) {
        //         // if d < prev_si.depth_searched || (prev_si.node_type != Node::PV && nt == Node::PV) {

        //         // if si.score.abs() > STALEMATE_VALUE - 100 {
        //         //     /// Value already in map is better, keep that instead
        //         //     return true;
        //         // }

        //         if d < prev_si.depth_searched {
        //             /// Value already in map is better, keep that instead
        //             return true;
        //         }
        //     }
        // }

        {
            let mut w = self.tt_w.lock();
            // w.clear(zb);
            // w.insert(zb, si);
            w.update(zb, si);
            w.refresh();
            // w.flush();
        }

        false
    }

}

impl std::cmp::PartialOrd for SICanUse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}


