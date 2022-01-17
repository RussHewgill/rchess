
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

use itertools::Itertools;
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

/// new
impl Explorer2 {
    pub fn new(
        side:          Color,
        game:          Game,
        max_depth:     Depth,
        time_settings: TimeSettings,
    ) -> Self {
        let stop = Arc::new(CachePadded::new(AtomicBool::new(false)));

        #[cfg(not(feature = "lockless_hashmap"))]
        let (tt_rf, tt_w) = new_hash_table();

        let ph_rw = PHTableFactory::new();

        let mut cfg = ExConfig::default();
        cfg.max_depth = max_depth;

        let (tx,rx): (ExSender,ExReceiver) = crossbeam::channel::unbounded();

        Self {
            side,
            game,

            #[cfg(feature = "basic_time")]
            timer:          Timer::new(time_settings),
            #[cfg(not(feature = "basic_time"))]
            time_settings,

            stop,
            best_mate:      Arc::new(RwLock::new(None)),
            best_depth:     Arc::new(CachePadded::new(AtomicI16::new(0))),

            tx,
            rx,

            cfg,
            search_params:  SParams::default(),

            #[cfg(feature = "syzygy")]
            syzygy:        None,

            nnue:          None,

            #[cfg(feature = "lockless_hashmap")]
            ptr_tt:        Arc::new(TransTable::new_mb(DEFAULT_TT_SIZE_MB)),

            #[cfg(not(feature = "lockless_hashmap"))]
            tt_rf,
            #[cfg(not(feature = "lockless_hashmap"))]
            tt_w,

            ph_rw,

            move_history:  vec![],

        }
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    pub handles:      Vec<JoinHandle<()>>,

    // pub waits:        Vec<Arc<(Mutex<bool>,Condvar)>>,
    pub thread_chans: Vec<Sender<ThreadUpdate>>,

}

#[derive(Debug,Clone,new)]
pub struct ThreadUpdate {
    pub side:           Color,
    pub game:           Game,
    pub cfg:            ExConfig,
    pub move_history:   Vec<(Zobrist, Move)>,
}

// impl ThreadUpdate {
//     pub fn new() -> Self {
//         // Self {
//         // }
//         unimplemented!()
//     }
// }

impl ThreadPool {
    pub fn new() -> Self {
        // let wait = crossbeam_channel::unbounded();
        // let wait = (wait.0,Arc::new(wait.1));
        // let wait = Arc::new((Mutex::new(false), Condvar::new()));
        Self {
            handles: vec![],
            // wait,
            // waits: vec![],
            thread_chans: vec![],
        }
    }
}

/// build_thread
impl Explorer2 {
    fn build_thread(
        &self,
        id:               usize,
        // wait:             Arc<Receiver<()>>,
        tx:               ExSender,
    // ) -> (Thread, Arc<(Mutex<bool>,Condvar)>) {
    ) -> (Thread, Sender<ThreadUpdate>) {
    // ) -> Thread {

        // let wait = Arc::new((Mutex::new(false), Condvar::new()));

        let (update_tx,update_rx) = crossbeam_channel::unbounded();

        let thread = Thread {
            id,

            side:            self.side,
            game:            self.game.clone(),

            stop:            self.stop.clone(),
            best_mate:       self.best_mate.clone(),
            best_depth:      self.best_depth.clone(),

            cfg:             self.cfg.clone(),
            params:          self.search_params.clone(),

            // wait:            wait.clone(),
            update_chan:     update_rx,

            #[cfg(feature = "syzygy")]
            syzygy:          self.syzygy.clone(),

            nnue:            self.nnue.clone().map(|x| RefCell::new(x)),

            tx,

            #[cfg(feature = "lockless_hashmap")]
            ptr_tt:          self.ptr_tt.clone(),

            #[cfg(not(feature = "lockless_hashmap"))]
            tt_r:            self.tt_rf.handle(),
            #[cfg(not(feature = "lockless_hashmap"))]
            tt_w:            self.tt_w.clone(),

            ph_rw:           self.ph_rw.handle(),

            move_history:    self.move_history.clone(),

            stats:           RefCell::new(SearchStats::default()),
            stack:           RefCell::new(ABStack::new()),

        };

        // (thread, wait)
        (thread, update_tx)
        // thread
    }
}

/// wakeup, sleep
impl ThreadPool {
    pub fn wakeup_threads(
        &self,
        side:           Color,
        game:           Game,
        cfg:            &ExConfig,
        move_history:   &Vec<(Zobrist,Move)>,
    ) {

        // for wait in self.waits.iter() {
        //     let mut lock = wait.0.lock();
        //     let cv = &wait.1;
        //     *lock = true;
        //     cv.notify_all();
        // }

        let update = ThreadUpdate::new(
            side,
            game,
            cfg.clone(),
            move_history.clone(),
        );

        for tx in self.thread_chans.iter() {
            tx.send(update.clone()).unwrap();
        }

    }

    pub fn sleep_threads(&self) {
        unimplemented!()
    }
}

/// spawn_threads
impl Explorer2 {

    pub fn spawn_threads(&mut self) -> ThreadPool {

        let mut threadpool = ThreadPool::new();

        #[cfg(feature = "one_thread")]
        let max_threads = 1;
        #[cfg(not(feature = "one_thread"))]
        let max_threads = if let Some(x) = self.cfg.num_threads {
            x as i8
        } else {
            let max_threads = num_cpus::get_physical();
            max_threads as i8
        };

        let mut thread_id = 0;

        for _ in 0..max_threads {
            trace!("Spawning thread, id = {}", thread_id);

            let best_depth = self.best_depth.clone();

            let (mut thread,update_tx) = self.build_thread(thread_id, self.tx.clone());
            // let thread = self.build_thread(thread_id, threadpool.wait.1.clone());

            let handle = std::thread::spawn(move || {
                thread.idle();
            });

            threadpool.handles.push(handle);
            // threadpool.waits.push(wait);
            threadpool.thread_chans.push(update_tx);

            thread_id += 1;
        }

        threadpool
    }

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

    // pub wait:            Arc<(Mutex<bool>,Condvar)>,
    pub update_chan:     Receiver<ThreadUpdate>,

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

    pub stats:           RefCell<SearchStats>,
    pub stack:           RefCell<ABStack>,

}

impl Thread {
}

/// idle, update, clear
impl Thread {

    pub fn idle(&mut self) {

        // let (lock,cvar) = &*self.wait;
        // // cvar.wait(lock.load(Relaxed));
        // let mut lock = lock.lock();
        // cvar.wait(&mut lock);

        match self.update_chan.recv() {
            Ok(update) => {
                self.update(update);
            },
            Err(e)     => panic!("wat"),
        }

        println!("thread (id: {:>2}) waking up", self.id);

    }

    pub fn update(&mut self, update: ThreadUpdate) {

        println!("thread id {} update", self.id);

    }

    pub fn clear(&self, g: Game) {
        let mut stack = self.stack.borrow_mut();
        stack.clear_history();
        // unimplemented!()
    }

}

/// ab_search_single
impl Thread {

    pub fn ab_search_single(
        &self,
        ts:             &'static Tables,
        mut stats:      &mut SearchStats,
        mut stack:      &mut ABStack,
        ab:             Option<(Score,Score)>,
        depth:          Depth,
    ) -> ABResults {
        unimplemented!()
    }

}

/// Lazy SMP Iterative Deepening loop
impl Thread {

    const SKIP_LEN: usize = 20;
    const SKIP_SIZE: [Depth; Self::SKIP_LEN] =
        [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
    const START_PLY: [Depth; Self::SKIP_LEN] =
        [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];

    fn lazy_smp_single(
        &self,
        ts:               &'static Tables,
    ) {

        let mut stack = ABStack::new_with_moves(&self.move_history);
        let mut stats = SearchStats::default();

        let skip_size = Self::SKIP_SIZE[self.id % Self::SKIP_LEN];
        let start_ply = Self::START_PLY[self.id % Self::SKIP_LEN];
        let mut depth = start_ply + 1;

        // let mut prev_best_move: Option<Move> = None;
        // let mut best_move_changes = 0;

        /// Iterative deepening
        // while !self.stop.load(SeqCst)
        while !self.stop.load(Relaxed)
            && depth <= self.cfg.max_depth
            && self.best_mate.read().is_none()
        {

            // XXX: needed?
            stack.pvs.fill(Move::NullMove);

            let res = self.ab_search_single(ts, &mut stats, &mut stack, None, depth);

            // /// If the best move hasn't changed for several iterations, use less time
            // if let Some(mv) = res.get_result().and_then(|res| res.mv) {
            //     if Some(mv) != prev_best_move {
            //         best_move_changes += 1;
            //     }
            //     prev_best_move = Some(mv);
            // }

            // if !self.stop.load(SeqCst) && depth >= self.best_depth.load(SeqCst) {
            if !self.stop.load(Relaxed) && depth >= self.best_depth.load(Relaxed) {
                let moves = if self.cfg.return_moves {
                    let mut v = stack.pvs.to_vec();
                    v.retain(|&mv| mv != Move::NullMove);
                    v
                } else { vec![] };
                match self.tx.try_send(ExMessage::Message(depth, res, moves, stats)) {
                    Ok(_)  => {
                        stats = SearchStats::default();
                    },
                    Err(_) => {
                        trace!("tx send error 0: id: {}, depth {}", self.id, depth);
                        break;
                    },
                }
            }

            depth += skip_size;
        }

        // for (ply,s) in stack.stacks.iter().enumerate() {
        //     let ks = s.killers;
        //     eprintln!("ply {} = {:?}", ply, ks);
        // }

        match self.tx.try_send(ExMessage::End(self.id)) {
            Ok(_)  => {},
            Err(_) => {
                trace!("tx send error 1: id: {}, depth {}", self.id, depth);
            },
        }

        if self.id == 0 {
            let mut w = DEBUG_ABSTACK.lock();
            *w = stack;
        }

        trace!("exiting lazy_smp_single, id = {}", self.id);
    }

}





