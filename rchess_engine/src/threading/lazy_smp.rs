
use crate::threading::thread_types::*;
use crate::pawn_hash_table::PHTable;
use crate::stack::ABStack;
use crate::tables::*;
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

/// Lazy SMP Dispatcher
impl Explorer2 {

    pub fn lazy_smp(&self) -> (ABResults,Vec<Move>,SearchStats) {

        self.clear_tt();
        self.reset_stop();

        // *self.best_mate.write() = None;
        self.best_mate.store(-1, SeqCst);
        self.best_depth.store(0, SeqCst);

        let out: Arc<RwLock<(Depth,ABResults,Vec<Move>, SearchStats)>> =
            Arc::new(RwLock::new((0, ABResults::ABUninit, vec![], SearchStats::default())));

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

        debug!("searching with time limit (soft,hard) = ({:?},{:?})",
               timer.limit_soft,
               timer.limit_hard);

        let handle_listener = {
            let rx          = self.rx.clone();
            let best_depth  = self.best_depth.clone();
            let best_mate   = self.best_mate.clone();
            let stop        = self.stop.clone();
            let out         = out.clone();
            let max_threads = self.max_threads;

            std::thread::spawn(move || {
                Self::lazy_smp_listener(
                    max_threads,
                    rx,
                    best_depth,
                    best_mate,
                    stop,
                    t0,
                    out);
            })
        };

        self.wakeup_threads();

        self.search_threads();

        'outer: loop {

            /// Check for out of time stop
            if timer.should_stop() {
                debug!("breaking loop (Time),  d: {}", self.best_depth.load(Relaxed));
                self.stop.store(true, SeqCst);
                break 'outer;
            }

            let d = self.best_depth.load(SeqCst);
            /// Max depth reached, halt
            if d >= self.cfg.max_depth {
                debug!("max depth ({}) reached, breaking", d);
                self.stop.store(true, SeqCst);
                break 'outer;
            }

            /// Found mate, halt
            // if self.best_mate.read().is_some() {
            if self.best_mate.load(SeqCst) != -1 {
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

        // self.tx.send(ExMessage::Stop).unwrap();

        /// Wait for listener to save results
        handle_listener.join().unwrap();

        let (d,mut out,moves,mut stats) = {
            let r = out.read();
            r.clone()
        };
        stats.max_depth = d as u8;

        stats.ph_hits   = self.ph_rw.hits.load(Ordering::Relaxed);
        stats.ph_misses = self.ph_rw.misses.load(Ordering::Relaxed);

        if let Some(res) = out.get_result() {

            // let out = if self.game.move_is_legal(&_TABLES, res.mv.unwrap(), self.game.state.side_to_move) {
            //     out
            // } else {
            //     debug!("best move wasn't legal? {:?}\n{:?}\n{:?}", self.game, self.game.to_fen(), res);
            //     // ABResults::ABNone
            //     panic!();
            // };
            // (out,moves,stats)

            unimplemented!()
        } else {

            debug!("lazy_smp, no result? res = {:?}", out);

            (out,moves,stats)
        }

    }

}

/// Lazy SMP Listener
impl Explorer2 {

    fn lazy_smp_listener(
        max_threads:      usize,
        rx:               ExReceiver,
        best_depth:       Arc<CachePadded<AtomicI16>>,
        // best_mate:        Arc<RwLock<Option<Depth>>>,
        best_mate:        Arc<CachePadded<AtomicI16>>,
        stop:             Arc<CachePadded<AtomicBool>>,
        t0:               Instant,
        out:              Arc<RwLock<(Depth,ABResults,Vec<Move>,SearchStats)>>,
    ) {
        // trace!("lazy_smp_listener start");
        let mut threads = max_threads as i32;
        let mut any_move_stored = false;

        /// Empty old messages
        while let Ok(_) = rx.try_recv() {}

        loop {
            match rx.recv() {
                Ok(ExMessage::Stop) => {
                    trace!("lazy_smp_listener Stop");
                    break;
                },
                Ok(ExMessage::End(id)) => {
                    // thread_counter.fetch_sub(1, SeqCst);
                    // trace!("lazy_smp_listener End, id = {:?}", id);
                    // trace!("decrementing thread counter id = {}, new val = {}",
                    //         id, thread_counter.load(SeqCst));
                    // break;

                    threads -= 1;
                    if threads <= 0 {
                        trace!("lazy_smp_listener breaking, all threads ended");
                        break;
                    }
                },
                Ok(ExMessage::Message(depth,res,moves,stats)) => {
                    // debug!("listener recv: d: {}", depth);
                    match res.clone() {
                        ABResults::ABList(bestres, _)
                            | ABResults::ABSingle(bestres)
                            | ABResults::ABSyzygy(bestres) => {
                                if depth > best_depth.load(SeqCst)
                                    || bestres.score > CHECKMATE_VALUE - 50
                                    || !any_move_stored
                                {
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

                                        // let mut best = best_mate.write();
                                        // *best = Some(k as Depth);

                                        best_mate.store(k as Depth, SeqCst);

                                        stop.store(true, SeqCst);

                                        let mut w = out.write();
                                        *w = (depth, res, moves, w.3 + stats);
                                        // *w = (depth, scores, None);
                                        best_depth.store(depth,SeqCst);
                                        break;
                                    } else {
                                        let mut w = out.write();
                                        // debug!("writing res: {:?}", res.get_result());
                                        *w = (depth, res, moves, w.3 + stats);
                                        best_depth.store(depth,SeqCst);
                                        any_move_stored = true;
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
                    trace!("Breaking lazy_smp_listener loop (Disconnect)");
                    break;
                },
            }
        }
        trace!("exiting lazy_smp_listener");
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

    pub fn lazy_smp_single(
        &mut self,
        ts:               &'static Tables,
    ) {

        // let mut stack = ABStack::new_with_moves(&self.move_history);

        // /// TODO: save stack
        // debug!("TODO: save stack");

        let mut stack = self.stack.clone();
        stack.move_history = self.move_history.clone();

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
            // && self.best_mate.read().is_none()
            && self.best_mate.load(Relaxed) == -1
        {

            // XXX: needed?
            stack.pvs.fill(Move::NullMove);

            let res: ABResults = self.ab_search_single(ts, &mut stats, &mut stack, None, depth);

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

                if res.get_result().is_some() {
                    // debug!("lazy_smp sending, res = {:?}", res.get_result().unwrap());
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
                // } else { debug!("lazy_smp, res is None"); }
            }
            // } else { debug!("lazy_smp skipping send"); }

            depth += skip_size;
        }

        if self.id == 0 {
            let mut w = DEBUG_ABSTACK.lock();
            *w = stack.clone();
        }

        self.stack = stack;

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

        trace!("idling lazy_smp_single, id = {}", self.id);

        // self.idle();
    }

}


