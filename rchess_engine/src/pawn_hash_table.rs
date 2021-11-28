
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

#[derive(Debug,Default,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy,Serialize,Deserialize,new)]
pub struct PHEntry {

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

#[derive(Debug,Default,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy,Serialize,Deserialize,new)]
pub struct PHScore {
    pub score_mid:            ByColor<Option<Score>>,
    pub score_end:            ByColor<Option<Score>>,
}

#[derive(Debug,Clone)]
pub struct PHTableFactory {
    pub ph_rf:    PHReadFactory,
    pub ph_w:    PHWrite,
    pub score_rf: EVReadFactory<PHScore>,
    pub score_w: EVWrite<PHScore>,
}

impl PHTableFactory {
    pub fn handle(&self) -> PHTable {
        // PHTable::new(self.ph_r.handle(), self.ph_w.clone())
        unimplemented!()
    }

    pub fn new() -> Self {

        let (ph_rf, ph_w) = new_hash_table();
        let (score_rf, score_w) = new_hash_table();

        Self {
            ph_rf,
            ph_w,
            score_rf,
            score_w,
        }
    }
}

#[derive(Debug,Clone,new)]
pub struct PHTable {
    pub ph_r:    PHRead,
    pub ph_w:    PHWrite,
    pub score_r: EVRead<PHScore>,
    pub score_w: EVWrite<PHScore>,
}

impl PHEntry {

    // pub fn get_score(&self, mid: bool, side: Color) -> &Option<Score> {
    //     self.get_scores(mid).get(side)
    // }

    // pub fn get_scores(&self, mid: bool) -> &ByColor<Option<Score>> {
    //     if mid {
    //         &self.score_mid
    //     } else {
    //         &self.score_end
    //     }
    // }

    // fn _update_score(&mut self, mid: bool, score: Score, side: Color) {
    //     if mid {
    //         // self.score_mid.
    //     } else {
    //     }
    // }

    // pub fn update_score(
    //     ph_rw:        &PHTable,
    //     zb:           Zobrist,
    //     mid:          bool,
    //     score:        Score,
    //     side:         Color,
    // ) {
    //     if let Some(ph) = ph_rw.0.get_one(&zb) {
    //         let mut ph = *ph;
    //         ph._update_score(mid, score, side);
    //         let mut w = ph_rw.1.lock();
    //         w.update(zb, ph);
    //         w.refresh();
    //     } else {
    //         panic!("PHEntry, updating non-existing entry");
    //     }
    // }

    pub fn get_or_insert_pawns(
        ts:           &Tables,
        g:            &Game,
        ph_rw:        Option<&PHTable>,
        // mut stats:    &mut SearchStats,
    ) -> PHEntry {
        if let Some(ph_rw) = ph_rw {
            if let Some(ph) = ph_rw.ph_r.get_one(&g.pawn_zb) {
                // stats.ph_hits += 1;
                *ph
            } else {
                // stats.ph_misses += 1;
                let ph = g.gen_ph_entry(ts);
                let mut w = ph_rw.ph_w.lock();
                w.update(g.pawn_zb, ph);
                w.refresh();
                ph
            }
        } else {
            g.gen_ph_entry(ts)
        }
    }
}

