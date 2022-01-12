
use crate::pawn_hash_table::PHTable;
use crate::stack::ABStack;
use crate::tuning::SParams;
use crate::types::*;
use crate::explore::{ExHelper,ExSender,ExConfig};
use crate::searchstats::SearchStats;
use crate::lockless_map::TransTable;

#[cfg(feature = "syzygy")]
use crate::syzygy::SyzygyTB;
use crate::sf_compat::NNUE4;

use std::time::{Instant,Duration};
use std::sync::Arc;
use std::sync::atomic::{Ordering,Ordering::SeqCst,Ordering::Relaxed,AtomicI16,AtomicBool};
use std::cell::{Cell,RefCell};
use std::thread::JoinHandle;

use itertools::Itertools;
use parking_lot::{Mutex,RwLock,Condvar};
use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use crossbeam::thread::ScopedJoinHandle;

pub struct ThreadPool {
    handles:      Vec<JoinHandle<()>>,
}

#[derive(Debug,Clone)]
pub struct Thread {

    pub id:              usize,

    pub side:            Color,
    pub game:            Game,

    #[cfg(feature = "syzygy")]
    pub syzygy:          Option<Arc<SyzygyTB>>,
    pub nnue:            Option<RefCell<NNUE4>>,

    pub cfg:             ExConfig,
    pub params:          SParams,

    pub wait:            Arc<(Mutex<bool>,Condvar)>,

    pub stop:            Arc<AtomicBool>,
    pub best_mate:       Arc<RwLock<Option<Depth>>>,
    pub best_depth:      Arc<AtomicI16>,
    pub tx:              ExSender,

    #[cfg(feature = "lockless_hashmap")]
    pub ptr_tt:          Arc<TransTable>,

    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_r:            TTRead,
    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_w:            TTWrite,

    pub ph_rw:           PHTable,

    pub move_history:    Vec<(Zobrist, Move)>,

    pub stats:           SearchStats,
    pub stack:           ABStack,

}

impl Thread {

    pub fn idle(&self) {
        let (lock,cvar) = &*self.wait;
        // cvar.wait(lock.load(Relaxed));
        let mut lock = lock.lock();
        cvar.wait(&mut lock);
    }

    pub fn new_search(&self, g: Game) {
        unimplemented!()
    }

}







