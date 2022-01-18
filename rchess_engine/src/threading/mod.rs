
pub mod thread_types;
pub mod lazy_smp;

use crate::pawn_hash_table::PHTable;
use crate::stack::ABStack;
use crate::tables::Tables;
use crate::tables::_TABLES;
use crate::tuning::*;
use crate::types::*;
use crate::explore::*;
use crate::searchstats::SearchStats;
use crate::lockless_map::{TransTable,DEFAULT_TT_SIZE_MB};
use crate::pawn_hash_table::*;
use crate::alphabeta::*;

pub use self::thread_types::*;

#[cfg(feature = "syzygy")]
use crate::syzygy::SyzygyTB;
use crate::sf_compat::NNUE4;

use std::path::Path;
use std::time::{Instant,Duration};
use std::sync::Arc;
use std::sync::atomic::{Ordering,Ordering::SeqCst,Ordering::Relaxed,AtomicI8,AtomicI16,AtomicBool};
use std::cell::{Cell,RefCell};
use std::thread::JoinHandle;

use derive_new::new;

use itertools::Itertools;
use parking_lot::{Mutex,RwLock,Condvar};
use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use crossbeam::thread::ScopedJoinHandle;
use crossbeam::utils::CachePadded;

#[derive(Debug,Clone,new)]
pub struct TestThread {
    pub id:              usize,
    pub wait:            Arc<(Mutex<bool>,Condvar)>,
}

#[derive(Debug,Clone)]
pub struct TestPool {
    // pub waits:        Vec<Arc<(Mutex<bool>,Condvar)>>,
    pub wait:            Arc<(Mutex<bool>,Condvar)>,
}

impl TestThread {
    pub fn idle(&self) {

        loop {

            // {
            //     let mut started = self.wait.0.lock();
            //     // eprintln!("started = {:?}", started);
            //     if !*started {
            //         self.wait.1.wait(&mut started);
            //     }
            // }

        }
    }
}

impl TestPool {
    pub fn new(num_threads: usize) -> Self {

        // let mut waits = vec![];

        let wait = Arc::new((Mutex::new(false), Condvar::new()));

        for id in 0..num_threads {

            let thread = TestThread::new(id, wait.clone());

            let handle = std::thread::spawn(move || {
                thread.idle();
            });

            // waits.push(wait);
        }

        Self {
            wait,
        }
    }
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

            threadpool:     None,

            tx,
            rx,

            cfg,
            search_params:  SParams::default(),

            #[cfg(feature = "syzygy")]
            syzygy:         None,

            nnue:           None,

            #[cfg(feature = "lockless_hashmap")]
            ptr_tt:         Arc::new(TransTable::new_mb(DEFAULT_TT_SIZE_MB)),

            #[cfg(not(feature = "lockless_hashmap"))]
            tt_rf,
            #[cfg(not(feature = "lockless_hashmap"))]
            tt_w,

            ph_rw,

            move_history:   vec![],

        }
    }
}

/// new
impl ThreadPool {
    pub fn new() -> Self {
        // let wait = crossbeam_channel::unbounded();
        // let wait = (wait.0,Arc::new(wait.1));
        // let wait = Arc::new((Mutex::new(false), Condvar::new()));
        Self {
            // handles: vec![],
            // wait,
            waits: vec![],
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
    ) -> (ExThread, Sender<ThreadUpdateType>, Arc<(Mutex<bool>,Condvar)>) {
    // ) -> Thread {

        let wait = Arc::new((Mutex::new(false), Condvar::new()));

        let (update_tx,update_rx) = crossbeam_channel::unbounded();

        let thread = ExThread {
            id,

            side:            self.side,
            game:            self.game.clone(),

            stop:            self.stop.clone(),
            best_mate:       self.best_mate.clone(),
            best_depth:      self.best_depth.clone(),

            cfg:             self.cfg.clone(),
            params:          self.search_params.clone(),

            wait:            wait.clone(),
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

            // stats:           RefCell::new(SearchStats::default()),
            // stack:           RefCell::new(ABStack::new()),
            stats:           SearchStats::default(),
            stack:           ABStack::new(),

        };

        // (thread, wait)
        (thread, update_tx, wait)
        // thread
    }
}

/// spawn_threads
impl Explorer2 {

    pub fn spawn_threads(&mut self) {

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

            let (mut thread,update_tx, wait) = self.build_thread(thread_id, self.tx.clone());
            // let thread = self.build_thread(thread_id, threadpool.wait.1.clone());

            let handle = std::thread::spawn(move || {
                thread.idle();
            });

            // threadpool.handles.push(handle);
            threadpool.waits.push(wait);
            threadpool.thread_chans.push(update_tx);

            thread_id += 1;
        }

        self.threadpool = Some(threadpool);

        /// give time for condvars to wait
        std::thread::sleep(Duration::from_millis(10));
    }

}

/// wakeup, update, sleep
impl Explorer2 {

    fn send_threads(&self, msg: &ThreadUpdateType) {
        for tx in self.threadpool.as_ref().unwrap().thread_chans.iter() {
            tx.send(msg.clone()).unwrap();
        }
    }

    pub fn search_threads(&self) {
        self.send_threads(&ThreadUpdateType::Search);
    }

    pub fn wakeup_threads(
        &self,
    ) {

        let threadpool: &ThreadPool = self.threadpool.as_ref().unwrap();

        for wait in threadpool.waits.iter() {
            let mut lock = wait.0.lock();
            let cv = &wait.1;
            *lock = true;
            // *lock = !*lock;
            cv.notify_all();
        }

        let update = ThreadUpdate::new(
            self.side,
            self.game,
            self.cfg.clone(),
            self.move_history.clone(),
        );

        self.send_threads(&ThreadUpdateType::Update(update))

    }

    pub fn sleep_threads(&self) {
        let threadpool: &ThreadPool = self.threadpool.as_ref().unwrap();
        for wait in threadpool.waits.iter() {
            let mut lock = wait.0.lock();
            *lock = false;
        }
    }

}

/// Entry points
impl Explorer2 {

    pub fn explore(&self) -> (Option<(Move,ABResult)>,SearchStats) {
        let (ress,moves,stats) = self.lazy_smp();
        if let Some(best) = ress.get_result() {
            debug!("explore: best move = {:?}", best.mv);
            // (Some((best.mv,best)),stats)
            (Some((best.mv.unwrap(),best)),stats)
        } else {
            debug!("explore: no best move? = {:?}", ress);
            // panic!();
            (None,stats)
        }
    }
}

/// misc
impl Explorer2 {

    pub fn reset_stop(&self) {
        self.stop.store(false, SeqCst);
        {
            let mut w = self.best_mate.write();
            *w = None;
        }
    }

    pub fn clear_tt(&self) {
        if self.cfg.clear_table {
            debug!("clearing tt");
            #[cfg(feature = "lockless_hashmap")]
            {
                self.ptr_tt.clear_table();
                self.ptr_tt.increment_cycle();
            }
            #[cfg(not(feature = "lockless_hashmap"))]
            {
                let mut w = self.tt_w.lock();
                w.purge();
                w.refresh();
            }
        } else {
            #[cfg(feature = "lockless_hashmap")]
            self.ptr_tt.increment_cycle();
        }
    }

    pub fn load_nnue<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        #[cfg(feature = "nnue")]
        {
            let mut nn = NNUE4::read_nnue(path)?;
            self.nnue = Some(nn);
        }
        Ok(())
    }

    pub fn update_game(&mut self, g: Game) {
        #[cfg(feature = "nnue")]
        if let Some(ref mut nnue) = self.nnue {
            // nnue.ft.accum.needs_refresh = [true; 2];
            nnue.ft.accum.stack_copies.clear();
            nnue.ft.accum.stack_delta.clear();
            nnue.ft.reset_accum(&g);
        }
        self.side = g.state.side_to_move;
        self.game = g;
    }

    pub fn update_game_movelist<'a>(
        &mut self,
        ts:          &Tables,
        fen:         &str,
        mut moves:   impl Iterator<Item = &'a str>
    ) {
        let mut g = Game::from_fen(&ts, &fen).unwrap();
        for m in moves {
            let from = &m[0..2];
            let to = &m[2..4];
            let other = &m[4..];
            let mm = g.convert_move(from, to, other).unwrap();
            g = g.make_move_unchecked(&ts, mm).unwrap();
            self.move_history.push((g.zobrist,mm));
        }

        self.update_game(g);
    }

}

/// idle, update, clear
impl ExThread {

    pub fn idle(&mut self) {

        loop {

            {
                let mut started = self.wait.0.lock();
                // eprintln!("started = {:?}", started);
                if !*started {
                    self.wait.1.wait(&mut started);
                }
            }

            match self.update_chan.recv() {
                Ok(ThreadUpdateType::Exit) => {
                    break;
                },
                Ok(ThreadUpdateType::Clear) => {
                    self.clear();
                },
                Ok(ThreadUpdateType::Search) => {
                    self.lazy_smp_single(&_TABLES);
                },
                Ok(ThreadUpdateType::Update(update)) => {
                    trace!("thread (id: {:>2}) waking up", self.id);
                    self.update(update);
                },
                Err(e)     => {
                    /// Should only be closed on program exit
                    // debug!("thread idle err: {:?}", e);
                    break;
                },
            }

        }

        trace!("exiting thread loop, id = {}", self.id);
    }

    pub fn update(&mut self, update: ThreadUpdate) {

        // trace!("thread id {} update", self.id);
        self.side         = update.side;
        self.game         = update.game;
        self.cfg          = update.cfg;
        self.move_history = update.move_history;


    }

    pub fn clear(&mut self) {
        self.stack.clear_history();
        // unimplemented!()
    }

}

