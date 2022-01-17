
pub mod thread_types;

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
    ) -> (ExThread, Sender<ThreadUpdate>, Arc<(Mutex<bool>,Condvar)>) {
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

/// wakeup, sleep
impl Explorer2 {

    pub fn wakeup_threads(
        &self,
    ) {
        self.threadpool.as_ref().unwrap().wakeup_threads(
            self.side,
            self.game,
            &self.cfg,
            &self.move_history);
    }

}

/// wakeup, sleep
impl ThreadPool {
    fn wakeup_threads(
        &self,
        side:           Color,
        game:           Game,
        cfg:            &ExConfig,
        move_history:   &Vec<(Zobrist,Move)>,
    ) {

        for wait in self.waits.iter() {
            let mut lock = wait.0.lock();
            let cv = &wait.1;
            *lock = true;
            // *lock = !*lock;
            cv.notify_all();
        }

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
        for wait in self.waits.iter() {
            let mut lock = wait.0.lock();
            *lock = false;
        }
    }

}

/// Entry points
impl Explorer2 {

    pub fn explore(&self, ts: &'static Tables) -> (Option<(Move,ABResult)>,SearchStats) {
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

/// Lazy SMP Dispatcher
impl Explorer2 {

    pub fn lazy_smp(&self) -> (ABResults,Vec<Move>,SearchStats) {

        self.clear_tt();
        self.reset_stop();

        let out: Arc<RwLock<(Depth,ABResults,Vec<Move>, SearchStats)>> =
            Arc::new(RwLock::new((0, ABResults::ABNone, vec![], SearchStats::default())));

        let t0 = Instant::now();

        #[cfg(feature = "basic_time")]
        let t_max = Duration::from_secs_f64(self.timer.settings.increment[self.side]);
        #[cfg(feature = "basic_time")]
        debug!("searching with t_max = {:?}", t_max);

        #[cfg(not(feature = "basic_time"))]
        let mut timer = TimeManager::new(self.time_settings);
        // let mut timer = TimeManager::new(self.time_settings);
        #[cfg(not(feature = "basic_time"))]
        debug!("searching with time limit (soft,hard) = ({:.3},{:.3})",
               timer.limit_soft as f64 / 1000.0,
               timer.limit_hard as f64 / 1000.0);

        {
            let rx         = self.rx.clone();
            let best_depth = self.best_depth.clone();
            let best_mate  = self.best_mate.clone();
            let stop       = self.stop.clone();
            let out        = out.clone();

            std::thread::spawn(move || {
                Self::lazy_smp_listener(
                    rx,
                    best_depth,
                    best_mate,
                    stop,
                    t0,
                    out);
            });
        }

        self.wakeup_threads();

        'outer: loop {

            /// Check for out of time stop
            if timer.should_stop() {
                debug!("breaking loop (Time),  d: {}", self.best_depth.load(Relaxed));
                self.stop.store(true, SeqCst);
                break 'outer;
            }

            let d = self.best_depth.load(Relaxed);
            /// Max depth reached, halt
            if d >= self.cfg.max_depth {
                debug!("max depth ({}) reached, breaking", d);
                self.stop.store(true, SeqCst);
                break 'outer;
            }

            /// Found mate, halt
            if self.best_mate.read().is_some() {
                #[cfg(not(feature = "basic_time"))]
                // let t1 = Instant::now().checked_duration_since(t0).unwrap();
                debug!("breaking loop (Mate),  d: {}", d);
                self.stop.store(true, SeqCst);
                break 'outer;
            }

            if self.stop.load(Relaxed) {
                break 'outer;
            }

            std::thread::sleep(Duration::from_millis(1));
        }
        trace!("exiting lazy_smp loop");

        let (d,mut out,moves,mut stats) = {
            let r = out.read();
            r.clone()
        };
        stats.max_depth = d as u8;

        stats.ph_hits   = self.ph_rw.hits.load(Ordering::Relaxed);
        stats.ph_misses = self.ph_rw.misses.load(Ordering::Relaxed);

        if let Some(res) = out.get_result() {
            let out = if self.game.move_is_legal(&_TABLES, res.mv.unwrap(), self.game.state.side_to_move) {
                out
            } else {
                debug!("best move wasn't legal? {:?}\n{:?}\n{:?}", self.game, self.game.to_fen(), res);
                ABResults::ABNone
            };
            (out,moves,stats)
        } else {
            (out,moves,stats)
        }

    }

}

/// Lazy SMP Listener
impl Explorer2 {

    fn lazy_smp_listener(
        rx:               ExReceiver,
        // thread_counter:   Arc<CachePadded<AtomicI8>>,
        best_depth:       Arc<CachePadded<AtomicI16>>,
        best_mate:        Arc<RwLock<Option<Depth>>>,
        stop:             Arc<CachePadded<AtomicBool>>,
        t0:               Instant,
        out:              Arc<RwLock<(Depth,ABResults,Vec<Move>,SearchStats)>>,
    ) {
        loop {
            // match rx.try_recv() {
            match rx.recv() {
                Ok(ExMessage::Stop) => {
                    trace!("lazy_smp_listener Stop");
                    break;
                },
                Ok(ExMessage::End(id)) => {
                    // thread_counter.fetch_sub(1, SeqCst);
                    trace!("lazy_smp_listener End, id = {:?}", id);
                    // trace!("decrementing thread counter id = {}, new val = {}",
                    //         id, thread_counter.load(SeqCst));
                    // break;
                },
                Ok(ExMessage::Message(depth,res,moves,stats)) => {
                    match res.clone() {
                        ABResults::ABList(bestres, _)
                            | ABResults::ABSingle(bestres)
                            | ABResults::ABSyzygy(bestres) => {
                            if depth > best_depth.load(SeqCst) {

                                // let t1 = t0.elapsed();
                                let t1 = Instant::now().checked_duration_since(t0).unwrap();
                                debug!("new best move d({}): {:.3}s: {}: {:?}",
                                       depth, t1.as_secs_f64(),
                                       // bestres.score, bestres.moves.front());
                                       bestres.score, bestres.mv);

                                if bestres.score.abs() == CHECKMATE_VALUE {
                                    stop.store(true, SeqCst);
                                    debug!("in mate, nothing to do");
                                    break;
                                }

                                if bestres.score > CHECKMATE_VALUE - 50 {
                                    let k = CHECKMATE_VALUE - bestres.score.abs();
                                    debug!("Found mate in {}: d({}), {:?}",
                                           // bestres.score, bestres.moves.front());
                                           k, depth, bestres.mv);
                                    let mut best = best_mate.write();
                                    *best = Some(k as Depth);

                                    stop.store(true, SeqCst);

                                    let mut w = out.write();
                                    *w = (depth, res, moves, w.3 + stats);
                                    // *w = (depth, scores, None);
                                    best_depth.store(depth,SeqCst);
                                    break;
                                } else {
                                    let mut w = out.write();
                                    *w = (depth, res, moves, w.3 + stats);
                                    best_depth.store(depth,SeqCst);
                                }
                            } else {
                                // XXX: add stats?
                            }
                        },
                        // ABResults::ABSyzygy(res) => {
                        //     panic!("TODO: Syzygy {:?}", res);
                        // }
                        ABResults::ABPrune(score, prune) => {
                            panic!("TODO: Prune at root ?? {:?}, {:?}", score, prune);
                        }
                        x => {
                            let mut w = out.write();
                            w.3 = w.3 + stats;
                            // panic!("rx: ?? {:?}", x);
                        },
                    }

                    // if let Some(id) = thread_dec {
                    //     thread_counter.fetch_sub(1, SeqCst);
                    //     trace!("decrementing thread counter id = {}, new val = {}",
                    //            id, thread_counter.load(SeqCst));
                    // }

                },
                // Err(TryRecvError::Empty)    => {
                //     // std::thread::sleep(Duration::from_millis(1));
                // },
                Err(_)    => {
                    trace!("Breaking thread counter loop (Disconnect)");
                    break;
                },
            }
        }
        trace!("exiting listener");
    }

}

/// idle, update, clear
impl ExThread {

    pub fn idle(&mut self) {

        loop {

            {
                let mut started = self.wait.0.lock();
                eprintln!("started = {:?}", started);
                self.wait.1.wait(&mut started);
            }

            match self.update_chan.recv() {
                Ok(update) => {
                    trace!("thread (id: {:>2}) waking up", self.id);
                    self.update(update);
                    self.clear();
                    self.lazy_smp_single(&_TABLES);
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

/// ab_search_single
impl ExThread {

    pub fn ab_search_single(
        &self,
        ts:             &'static Tables,
        mut stats:      &mut SearchStats,
        mut stack:      &mut ABStack,
        ab:             Option<(Score,Score)>,
        depth:          Depth,
    ) -> ABResults {

        let (alpha,beta) = if let Some(ab) = ab { ab } else {
            (Score::MIN + 200, Score::MAX - 200)
        };

        let mut g = self.game.clone();

        #[cfg(feature = "new_search")]
        let res = self.ab_search::<{ABNodeType::Root}>(
            ts,
            &g,
            (depth,0),
            (alpha,beta),
            stats,
            stack,
            false);

        res
    }

}

/// Lazy SMP Iterative Deepening loop
impl ExThread {

    const SKIP_LEN: usize = 20;
    const SKIP_SIZE: [Depth; Self::SKIP_LEN] =
        [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
    const START_PLY: [Depth; Self::SKIP_LEN] =
        [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];

    fn lazy_smp_single(
        &mut self,
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

        trace!("idling lazy_smp_single, id = {}", self.id);

        self.idle();
    }

}





