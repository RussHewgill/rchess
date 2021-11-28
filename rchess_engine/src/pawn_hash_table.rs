
use crate::types::*;
use crate::tables::*;
use crate::explore::*;
use crate::trans_table::FxBuildHasher;

use std::sync::Arc;

use derive_new::new;
use serde::{Serialize,Deserialize};
use evmap::{ReadHandle,ReadHandleFactory,WriteHandle};
use evmap_derive::ShallowCopy;
use parking_lot::Mutex;

pub type PHReadFactory  = ReadHandleFactory<Zobrist, PHEntry, (), FxBuildHasher>;

pub type PHRead  = ReadHandle<Zobrist, PHEntry, (), FxBuildHasher>;
pub type PHWrite = Arc<Mutex<WriteHandle<Zobrist, PHEntry, (), FxBuildHasher>>>;

#[derive(Debug,Default,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy,Serialize,Deserialize,new)]
pub struct PHEntry {

    pub connected:            BitBoard,
    pub supported_1:          BitBoard,
    pub supported_2:          BitBoard,
    pub phalanx:              BitBoard,
    pub passed:               BitBoard,
    pub candidate:            BitBoard,
    pub blocked:              BitBoard,

    pub doubled:              BitBoard,
    pub isolated:             BitBoard,
    // pub doubled_isolated:     BitBoard,
    pub backward:             BitBoard,

}

impl PHEntry {
    pub fn get_or_insert_pawns(
        ts:           &Tables,
        g:            &Game,
        ph_rw:        Option<&(PHRead,PHWrite)>,
        // mut stats:    &mut SearchStats,
    ) -> PHEntry {
        if let Some((ph_r,ph_w)) = ph_rw {
            if let Some(ph) = ph_r.get_one(&g.pawn_zb) {
                // stats.ph_hits += 1;
                *ph
            } else {
                // stats.ph_misses += 1;
                let ph = g.gen_ph_entry(ts);
                let mut w = ph_w.lock();
                w.update(g.pawn_zb, ph);
                ph
            }
        } else {
            g.gen_ph_entry(ts)
        }
    }
}

