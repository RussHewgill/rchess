
use crate::pawn_hash_table::PHTable;
use crate::stack::ABStack;
use crate::tables::Tables;
use crate::tuning::*;
use crate::types::*;
use crate::explore::*;
use crate::searchstats::SearchStats;
use crate::lockless_map::{TransTable,DEFAULT_TT_SIZE_MB};
use crate::pawn_hash_table::*;
use crate::alphabeta::*;

#[cfg(feature = "syzygy")]
use crate::syzygy::SyzygyTB;
use crate::sf_compat::NNUE4;

use std::time::{Instant,Duration};
use std::sync::Arc;
use std::sync::atomic::{Ordering,Ordering::SeqCst,Ordering::Relaxed,AtomicI16,AtomicBool};
use std::cell::{Cell,RefCell};
use std::thread::JoinHandle;

use derive_new::new;

use parking_lot::{Mutex,RwLock,Condvar};
use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use crossbeam::thread::ScopedJoinHandle;
use crossbeam::utils::CachePadded;

#[derive(Debug,Clone)]
pub struct Explorer2 {
    pub side:          Color,
    pub game:          Game,
    // pub current_ply:   Option<Depth>, // XXX: not used

    #[cfg(feature = "basic_time")]
    pub timer:         Timer,
    #[cfg(not(feature = "basic_time"))]
    pub time_settings: TimeSettings,

    pub stop:          Arc<CachePadded<AtomicBool>>,
    pub best_mate:     Arc<RwLock<Option<Depth>>>,
    pub best_depth:    Arc<CachePadded<AtomicI16>>,

    pub threadpool:    Option<ThreadPool>,

    pub tx:            ExSender,
    pub rx:            ExReceiver,

    pub cfg:           ExConfig,

    pub search_params: SParams,

    #[cfg(feature = "syzygy")]
    pub syzygy:        Option<Arc<SyzygyTB>>,
    // pub opening_book:  Option<Arc<OpeningBook>>,

    pub nnue:          Option<NNUE4>,

    #[cfg(feature = "lockless_hashmap")]
    pub ptr_tt:        Arc<TransTable>,

    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_rf:         TTReadFactory,
    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_w:          TTWrite,

    // pub eval_hashmap:  (EVReadFactory<Score>,EVWrite<Score>),

    // pub ph_rw:         (PHReadFactory,PHWrite),
    pub ph_rw:         PHTableFactory,

    pub move_history:  Vec<(Zobrist, Move)>,
    // pub pos_history:   HashMap<Zobrist,u8>,

}

#[derive(Debug,Clone)]
pub struct ThreadPool {
    // pub handles:      Vec<JoinHandle<()>>,

    pub waits:        Vec<Arc<(Mutex<bool>,Condvar)>>,
    pub thread_chans: Vec<Sender<ThreadUpdateType>>,

}

#[derive(Debug,Clone,new)]
pub enum ThreadUpdateType {
    Exit,
    Search,
    Clear,
    Update(ThreadUpdate),
}

#[derive(Debug,Clone,new)]
pub struct ThreadUpdate {
    pub side:           Color,
    pub game:           Game,
    pub cfg:            ExConfig,
    pub move_history:   Vec<(Zobrist, Move)>,
}

#[derive(Debug,Clone)]
pub struct ExThread {

    pub id:              usize,

    pub side:            Color,
    pub game:            Game,

    #[cfg(feature = "syzygy")]
    pub syzygy:          Option<Arc<SyzygyTB>>,
    pub nnue:            Option<RefCell<NNUE4>>,

    pub cfg:             ExConfig,
    pub params:          SParams,

    pub wait:            Arc<(Mutex<bool>,Condvar)>,
    pub update_chan:     Receiver<ThreadUpdateType>,

    pub stop:            Arc<CachePadded<AtomicBool>>,
    pub best_mate:       Arc<RwLock<Option<Depth>>>,
    pub best_depth:      Arc<CachePadded<AtomicI16>>,
    pub tx:              ExSender,

    #[cfg(feature = "lockless_hashmap")]
    pub ptr_tt:          Arc<TransTable>,

    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_r:            TTRead,
    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_w:            TTWrite,

    pub ph_rw:           PHTable,

    pub move_history:    Vec<(Zobrist, Move)>,

    // pub stats:           RefCell<SearchStats>,
    // pub stack:           RefCell<ABStack>,
    pub stats:           SearchStats,
    pub stack:           ABStack,

}

