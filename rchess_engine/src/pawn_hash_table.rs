
pub use self::table::*;

use crate::types::*;
use crate::tables::*;
use crate::explore::*;
use crate::evaluate::*;
use crate::evmap_tables::*;

use std::hash::Hash;
use std::sync::Arc;

use derive_new::new;
use evmap::ShallowCopy;
use serde::{Serialize,Deserialize};
use evmap::{ReadHandle,ReadHandleFactory,WriteHandle};
use evmap_derive::ShallowCopy;
use parking_lot::Mutex;

pub type PHReadFactory = EVReadFactory<PHEntry>;
pub type PHRead        = EVRead<PHEntry>;
pub type PHWrite       = EVWrite<PHEntry>;

mod table {
    use std::sync::atomic::AtomicU32;

    use super::*;

    #[derive(Debug,Clone)]
    pub struct PHTableFactory {
        pub ph_rf:    PHReadFactory,
        pub ph_w:    PHWrite,
        // pub score_rf: EVReadFactory<PHScore>,
        // pub score_w: EVWrite<PHScore>,
        pub hits:    Arc<AtomicU32>,
        pub misses:  Arc<AtomicU32>,
    }

    impl PHTableFactory {
        pub fn new() -> Self {

            let (ph_rf, ph_w) = new_hash_table();
            // let (score_rf, score_w) = new_hash_table();

            Self {
                ph_rf,
                ph_w,
                // score_rf,
                // score_w,
                hits:     Arc::new(AtomicU32::new(0)),
                misses:   Arc::new(AtomicU32::new(0)),
            }
        }
        pub fn handle(&self) -> PHTable {
            // PHTable::new(self.ph_r.handle(), self.ph_w.clone())
            PHTable {
                ph_r:       self.ph_rf.handle(),
                ph_w:       self.ph_w.clone(),
                // score_r:    self.score_rf.handle(),
                // score_w:    self.score_w.clone(),
                hits:       self.hits.clone(),
                misses:     self.misses.clone(),
            }
        }
    }

    #[derive(Debug,Clone,new)]
    pub struct PHTable {
        pub ph_r:    PHRead,
        pub ph_w:    PHWrite,
        // pub score_r: EVRead<PHScore>,
        // pub score_w: EVWrite<PHScore>,
        pub hits:    Arc<AtomicU32>,
        pub misses:  Arc<AtomicU32>,
    }

    impl PHTable {
        pub fn purge(&self) {
            let mut w = self.ph_w.lock();
            w.purge();
            w.refresh();
        }
    }

    // impl PHTable {
    //     // pub fn get_score(&self, mid: bool, side: Color) -> &Option<Score> {
    //     //     self.get_scores(mid).get(side)
    //     // }
    //     pub fn get_scores(&self, zb: &Zobrist) -> Option<PHScore> {
    //         if let Some(x) = self.score_r.get_one(zb) {
    //             Some(*x)
    //         } else { None }
    //     }
    // }


}

#[derive(Debug,Default,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy,Serialize,Deserialize,new)]
pub struct PHEntry {

    pub score_mid:            ByColor<Score>,
    pub score_end:            ByColor<Score>,

    pub connected:            BitBoard,
    pub supported_1:          BitBoard,
    pub supported_2:          BitBoard,
    pub phalanx:              BitBoard,
    pub passed:               BitBoard,
    pub candidate:            BitBoard,
    pub blocked:              BitBoard,
    pub opposed:              BitBoard,

    pub doubled:              BitBoard,
    pub isolated:             BitBoard,
    // pub doubled_isolated:     BitBoard,
    pub backward:             BitBoard,

}

// #[derive(Debug,Default,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy,Serialize,Deserialize,new)]
// pub struct PHScore {
//     pub score_mid:            ByColor<Option<Score>>,
//     pub score_end:            ByColor<Option<Score>>,
// }

impl PHEntry {

    pub fn get_scores(&self, mid: bool) -> [Score; 2] {
        if mid {
            [self.score_mid.white,self.score_mid.black]
        } else {
            [self.score_end.white,self.score_end.black]
        }
    }

    pub fn update_score_mut(&mut self, score: Score, mid: bool, side: Color) {
        let mut bc = if mid {
            &mut self.score_mid
        } else {
            &mut self.score_end
        };
        bc.insert_mut(side, score);
    }

    pub fn get_or_insert_pawns(
        ts:           &Tables,
        g:            &Game,
        ev_mid:       &EvalParams,
        ev_end:       &EvalParams,
        ph_rw:        Option<&PHTable>,
        // mut stats:    &mut SearchStats,
    ) -> PHEntry {
        if let Some(ph_rw) = ph_rw {
            if let Some(ph) = ph_rw.ph_r.get_one(&g.pawn_zb) {
                // stats.ph_hits += 1;
                ph_rw.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                *ph
            } else {
                ph_rw.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let mut ph = g.gen_ph_entry(ts, ev_mid, ev_end);

                for side in [White,Black] {
                    g._score_pawns_mut(&mut ph, ev_mid, side);
                    g._score_pawns_mut(&mut ph, ev_end, side);
                }
                let mut w = ph_rw.ph_w.lock();
                w.update(g.pawn_zb, ph);
                w.refresh();
                ph
            }
        } else {
            let mut ph = g.gen_ph_entry(ts, ev_mid, ev_end);
            for side in [White,Black] {
                g._score_pawns_mut(&mut ph, ev_mid, side);
                g._score_pawns_mut(&mut ph, ev_end, side);
            }
            ph
        }
    }
}

