
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;
use crate::alphabeta::*;

pub use crate::move_ordering::*;
pub use crate::timer::*;
pub use crate::trans_table::*;
pub use crate::searchstats::*;

use std::collections::VecDeque;
use std::sync::atomic::{Ordering,Ordering::SeqCst,AtomicU8};
use std::time::{Instant,Duration};

use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use crossbeam::thread::ScopedJoinHandle;

use itertools::Itertools;
use parking_lot::{Mutex,RwLock};

use rand::prelude::{SliceRandom,thread_rng};
use rayon::prelude::*;

use evmap::{ReadHandle,WriteHandle};

// #[derive(Debug)]
#[derive(Debug,Clone)]
pub struct Explorer {
    pub side:          Color,
    pub game:          Game,
    pub max_depth:     Depth,
    pub timer:         Timer,
    pub stop:          Arc<AtomicBool>,
    pub best_mate:     Arc<RwLock<Option<Depth>>>,
    // pub move_history:  VecDeque<(Zobrist, Move)>,
    // pub 
    // pub prev_eval:     Arc<>
}

#[derive(Debug,Default,Clone,Copy)]
pub struct ABConfig {
    pub max_depth:        Depth,
    // pub depth:            Depth,
    // pub ply:              Depth,
    // pub tt_r:             &'a TTRead,
    pub root:             bool,
    pub do_null:          bool,
}

impl ABConfig {
    pub fn new_depth(max_depth: Depth) -> Self {
        Self {
            max_depth,
            root: false,
            do_null: true,
        }
    }
}

/// New
impl Explorer {
    pub fn new(side:          Color,
               game:          Game,
               max_depth:     Depth,
               should_stop:   Arc<AtomicBool>,
               settings:      TimeSettings,
    ) -> Self {
        Self {
            side,
            game,
            max_depth,
            timer:          Timer::new(side, should_stop, settings),
            stop:           Arc::new(AtomicBool::new(false)),
            best_mate:      Arc::new(RwLock::new(None)),
            // move_history:   VecDeque::default(),
        }
    }
}

/// Entry points
impl Explorer {

    pub fn explore(&self, ts: &Tables, depth: Option<Depth>)
                   // -> (Option<(Move,Score)>,SearchStats) {
                   -> (Option<(Move,ABResult)>,SearchStats) {

        // let (moves,stats) = self.iterative_deepening(&ts, false, false);
        // (moves.get(0).map(|x| x.0),stats)

        // let (moves,stats,_) = self.lazy_smp(&ts, false, false);

        // let mv = moves.get(0).map(|x| x.0);
        // let mv = moves.get(0).map(|x| (x.0,x.2));

        let ((best, scores),stats,(tt_r,tt_w)) = self.lazy_smp_negamax(&ts, false, false);
        let mv = best.moves[0];
        let score = best.score;

        debug!("explore: best move = {:?}", mv);
        // (Some((mv,score)),stats)
        (Some((mv,best)),stats)

        // unimplemented!()

        // unimplemented!()
        // if iterative {
        //     let (moves,stats) = self.iterative_deepening(&ts, false);
        //     (moves.get(0).map(|x| x.0),stats)
        // } else {
        //     let (moves,stats) = self.rank_moves(&ts, false);
        //     (moves.get(0).map(|x| x.0),stats)
        // }

    }

}

/// iterative_deepening
#[cfg(feature = "iterative")]
impl Explorer {

    pub fn iterative_deepening(&self, ts: &Tables, print: bool, strict_depth: bool)
                               -> (Vec<(Move,Vec<Move>,Score)>,SearchStats) {

        let ms = self.game.search_all(&ts, None);
        let ms = ms.get_moves_unsafe();
        let (ms,stats) = self._iterative_deepening(&ts, print, ms, strict_depth);

        // (ms.get(0).map(|x| x.0),stats)
        // (ms.get(0).copied(),stats)
        (ms,stats)
    }

    pub fn _iterative_deepening(&self, ts: &Tables, print: bool, moves: Vec<Move>, strict_depth: bool)
                                -> (Vec<(Move,Vec<Move>,Score)>,SearchStats) {

        // self.trans_table_w.purge();
        // self.trans_table_w.refresh();

        // self.trans_table.clear();

        let mut timer = self.timer.clone();
        timer.reset();

        // let mut out: Vec<(Move,i32)> = vec![];
        let mut out = vec![];
        let mut ss:  Vec<SearchStats>;
        let mut stats = SearchStats::default();
        let mut depth = 0;

        #[cfg(feature = "par")]
        let gs: Vec<(Move,Game)> = moves.par_iter().flat_map(|mv| {
            if let Ok(g2) = self.game.make_move_unchecked(&ts, *mv) {
                Some((*mv,g2))
            } else { None }
        }).collect();
        #[cfg(not(feature = "par"))]
        let gs: Vec<(Move,Game)> = moves.iter().flat_map(|mv| {
            if let Ok(g2) = self.game.make_move_unchecked(&ts, *mv) {
                Some((*mv,g2))
            } else { None }
        }).collect();

        // let alpha = Arc::new(AtomicI32::new(i32::MIN));
        // let beta  = Arc::new(AtomicI32::new(i32::MAX));

        // while (depth <= self.max_depth) && (strict_depth || (|| timer.should_search(self.side, depth))()) {
        while (depth <= self.max_depth) && (strict_depth || (timer.should_search(self.side, depth))) {
        // while (depth <= self.max_depth) && (timer.should_search(self.side, depth)) {

            // // eprintln!("Explorer: not parallel");
            // // (out,ss) = gs.iter().map(|(mv,g2)| {
            // (out,ss) = gs.par_iter().map(|(mv,g2)| {
            //     let alpha = Arc::new(AtomicI32::new(i32::MIN));
            //     let beta  = Arc::new(AtomicI32::new(i32::MAX));
            //     // let alpha = i32::MIN;
            //     // let beta  = i32::MAX;
            //     let mut stats = SearchStats::default();
            //     let (mut mv_seq,score) = self._ab_search(
            //         &ts, &g2, depth, 1, alpha, beta, false, &mut stats, *mv);
            //     mv_seq.push(*mv);
            //     mv_seq.reverse();
            //     ((*mv,mv_seq,score),stats)
            // }).unzip();

            let stop = Arc::new(AtomicBool::new(false));

            // let (tt_r, tt_w) = evmap::new();
            // let tt_w = Arc::new(Mutex::new(tt_w));

            #[cfg(feature = "par")]
            {
                // let gs2 = gs.into_iter().map(|(mv,g2)| {
                //     ((mv,g2),tt_r.clone(),tt_w.clone())
                // }).collect::<Vec<_>>();
                // (out,ss) = gs2.par_iter().map(|((mv,g2),tt_r2,tt_w2)| {
                (out,ss) = gs.par_iter().map(|(mv,g2)| {
                    let (tt_r, tt_w) = evmap::new();
                    let tt_w = Arc::new(Mutex::new(tt_w));
                    let mut stats = SearchStats::default();
                    let ((mut mv_seq,score),_) = {
                        let alpha = i32::MIN;
                        let beta  = i32::MAX;
                        self._ab_search(
                            &ts, &g2, depth, depth, 1, alpha, beta, false, &mut stats, *mv,
                            // &tt_r2, tt_w2,
                            &tt_r, tt_w,
                        )
                            .unwrap()
                    };
                    mv_seq.push(*mv);
                    mv_seq.reverse();
                    ((*mv,mv_seq,score),stats)
                }).unzip();
            }
            #[cfg(not(feature = "par"))]
            {
                // eprintln!("Explorer: not parallel");
                (out,ss) = gs.iter().map(|(mv,g2)| {
                // (out,ss) = gs.par_iter().map(|(mv,g2)| {
                //     let mut stats = SearchStats::default();
                    let (mut mv_seq,score) = {
                        let alpha = i32::MIN;
                        let beta  = i32::MAX;
                        self._ab_search(
                            &ts, &g2, depth, 1, alpha, beta, false, &mut stats, stop, *mv)
                    };
                    mv_seq.push(*mv);
                    mv_seq.reverse();
                    ((*mv,mv_seq,score),stats)
                }).unzip();
            }

            stats = ss.iter().sum();

            #[cfg(feature = "par")]
            out.par_sort_unstable_by(|a,b| a.2.cmp(&b.2));
            #[cfg(not(feature = "par"))]
            out.sort_unstable_by(|a,b| a.2.cmp(&b.2));

            if self.side == self.game.state.side_to_move {
                out.reverse();
            }

            if print {
                eprintln!("depth = {:?}", depth);
                eprintln!("nodes = {:?}", stats.nodes);
            }

            stats.max_depth = depth;
            depth += 1;
            timer.update_times(self.side, stats.nodes);
            if print {
                eprintln!("depth, time = {:?}, {:.2}", depth-1, timer.time_left[self.side]);
            }
        }
        // if print {
        //     print!("\n");
        //     for (m,s) in out.iter() {
        //         eprintln!("{:>8} = {:?}", s, m);
        //     }
        // }
        (out,stats)
    }

}

/// Lazy SMP Negamax
impl Explorer {

    fn _lazy_smp_single_negamax(
        &self,
        ts:               &Tables,
        depth:            Depth,
        tx:               Sender<(Depth,ABResults,SearchStats)>,
        // stats:            Arc<RwLock<SearchStats>>,
        // mut history:      [[[Score; 64]; 64]; 2],
        tt_r:             TTRead,
        tt_w:             TTWrite,
    ) {
        let mut history = [[[0; 64]; 64]; 2];

        let (alpha,beta) = (i32::MIN,i32::MAX);
        let (alpha,beta) = (alpha + 200,beta - 200);

        let mut stats = SearchStats::default();
        let mut stop_counter = 0;

        let mut cfg = ABConfig::new_depth(depth);
        cfg.root = true;

        let res = self._ab_search_negamax(
            &ts, &self.game, cfg, depth,
            0, &mut stop_counter, (alpha, beta),
            &mut stats,
            VecDeque::new(),
            &mut history,
            // &tt_r, tt_w.clone(),true,true);
            &tt_r, tt_w.clone());

        // XXX: ??
        match tx.send((depth,res,stats)) {
        // match tx.try_send((depth,res,stats)) {
            Ok(_) => {},
            // Err(_) => panic!("tx send error: depth {}", depth),
            Err(_) => trace!("tx send error: depth {}", depth),
            // Err(_) => {},
        }
        drop(tx);

    }

    #[allow(unused_doc_comments)]
    pub fn lazy_smp_negamax(&self, ts: &Tables, print: bool, strict_depth: bool)
                            -> ((ABResult, Vec<ABResult>),SearchStats,(TTRead,TTWrite)) {

        let out: Arc<RwLock<(Depth,ABResults,SearchStats)>> =
            Arc::new(RwLock::new((0, ABResults::ABNone, SearchStats::default())));

        let (tt_r, tt_w) = evmap::new();
        let tt_w = Arc::new(Mutex::new(tt_w));

        let sleep_time = Duration::from_millis(1);

        self.stop.store(false, SeqCst);
        {
            let mut w = self.best_mate.write();
            *w = None;
        }

        // let stop3 = self.stop.clone();
        // ctrlc::set_handler(move || {
        //     debug!("Ctrl-C recieved, halting");
        //     stop3.store(true, SeqCst);
        // })

        crossbeam::scope(|s| {

            let (tx,rx): (Sender<(Depth,ABResults,SearchStats)>,
                          Receiver<(Depth,ABResults,SearchStats)>) =
                crossbeam::channel::bounded(12);

            let mut timer = self.timer.clone();
            timer.reset();

            let mut stats = SearchStats::default();

            let best_depth      = Arc::new(AtomicU8::new(0));
            let best_depth2     = best_depth.clone();
            let search_id       = Arc::new(AtomicU8::new(0));
            let thread_counter  = Arc::new(AtomicU8::new(0));
            let thread_counter2 = thread_counter.clone();

            let out1            = out.clone();
            let out2            = out.clone();
            let stop            = self.stop.clone();
            let stop2           = self.stop.clone();

            #[cfg(feature = "one_thread")]
            let max_threads = 1;
            #[cfg(not(feature = "one_thread"))]
            // let max_threads = 6;
            // let max_threads = np_cpus;
            let max_threads = 6;

            let depths = vec![
                0, 1, 0, 2, 0, 1,
                0, 1, 0, 2, 0, 1,
            ];

            let t0 = Instant::now();
            let t_max = self.timer.settings.increment[self.side];
                // + (self.timer.settings.safety / 2.0);
            let t_max = Duration::from_secs_f64(t_max);

            let rx_thread = s.builder()
                // .stack_size(4 * 1024 * 1024) // 4 MiB
                .spawn(move |_| loop {

                    match rx.try_recv() {
                        Err(TryRecvError::Empty)    => {
                            std::thread::sleep(sleep_time);
                            // std::thread::sleep(Duration::from_millis(1));
                        },
                        Err(TryRecvError::Disconnected)    => {
                            trace!("Breaking thread counter loop (Disconnect)");
                            break;
                        },
                        Ok((depth,abres,stats0)) => {
                            if stop2.load(SeqCst) {
                                trace!("Breaking thread counter loop (Force Stop)");
                                let mut w = out1.write();
                                w.2 = w.2 + stats0;
                                break;
                            }
                            match abres.clone() {
                                ABResults::ABList(bestres, ress) if depth > best_depth2.load(SeqCst) => {
                                    best_depth2.store(depth,SeqCst);

                                    debug!("new best move d({}): {:.3}s: {}: {:?}",
                                        depth, t0.elapsed().as_secs_f64(),
                                        bestres.score, bestres.moves.front());

                                    if bestres.score > 100_000_000 - 50 {
                                        let k = 100_000_000 - bestres.score.abs();
                                        debug!("Found mate in {}: d({}), {:?}",
                                               k, depth, bestres.moves.front());
                                        let mut best = self.best_mate.write();
                                        *best = Some(k as u8);
                                        let mut w = out1.write();
                                        *w = (depth, abres, w.2 + stats0);
                                        // *w = (depth, scores, None);
                                        break;
                                    } else {
                                        let mut w = out1.write();
                                        *w = (depth, abres, w.2 + stats0);
                                    }

                                },
                                _ => {
                                    let mut w = out1.write();
                                    w.2 = w.2 + stats0;
                                },

                            }
                            thread_counter2.fetch_sub(1, SeqCst);
                            trace!("decrementing thread counter, new val = {}", thread_counter2.load(SeqCst));
                        },
                    }
                }).unwrap();

            'outer: loop {

                {
                    let r = self.best_mate.read();
                    if r.is_some() {
                        let d = best_depth.load(SeqCst);
                        debug!("breaking loop (Mate),  d: {}, t0: {:.3}",
                               d, t0.elapsed().as_secs_f64());
                        stop.store(true, SeqCst);
                        break 'outer;
                    }
                }

                // if best_depth.load(SeqCst) + 1 + depths_largest > self.max_depth {
                if best_depth.load(SeqCst) + 1 > self.max_depth {
                // if thread_counter.load(SeqCst) != 0 && best_depth.load(SeqCst) + 1 > self.max_depth {
                    let d = best_depth.load(SeqCst);
                    debug!("breaking loop (Depth), d: {}, t0: {:.3}", d, t0.elapsed().as_secs_f64());
                    // drop(tx);

                    loop {

                        {
                            let r = self.best_mate.read();
                            if r.is_some() {
                                let d = best_depth.load(SeqCst);
                                debug!("breaking loop (Depth -> Mate),  d: {}, t0: {:.3}",
                                       d, t0.elapsed().as_secs_f64());
                                stop.store(true, SeqCst);
                                drop(tx);
                                break 'outer;
                            }
                        }

                        if thread_counter.load(SeqCst) == 0 {
                            let d = best_depth.load(SeqCst);
                            debug!("breaking loop (Depth -> Threads),  d: {}, t0: {:.3}",
                                   d, t0.elapsed().as_secs_f64());
                            stop.store(true, SeqCst);
                            drop(tx);
                            break 'outer;
                        }
                        if t0.elapsed() > t_max {
                            let d = best_depth.load(SeqCst);
                            debug!("breaking loop (Depth -> Time),  d: {}, t0: {:.3}",
                                   d, t0.elapsed().as_secs_f64());
                            stop.store(true, SeqCst);
                            drop(tx);
                            break 'outer;
                        } else {
                            // trace!("t0.elapsed(), t_max: {:?}, {:?}", t0.elapsed(), t_max);
                        }
                        std::thread::sleep(sleep_time);
                    }

                    // break 'outer;
                // } if thread_counter.load(SeqCst) != 0 && cur_depth > self.max_depth {
                }

                let sid       = search_id.load(SeqCst);
                let cur_depth = best_depth.load(SeqCst) + 1 + depths[sid as usize];

                if t0.elapsed() > t_max {
                    let d = best_depth.load(SeqCst);
                    debug!("breaking loop (Time),  d: {}, t0: {:.3}", d, t0.elapsed().as_secs_f64());
                    // XXX: Only force threads to stop if out of time ?
                    stop.store(true, SeqCst);
                    drop(tx);
                    break;
                }
                if cur_depth > self.max_depth {
                    // debug!("cur_depth > self.max_depth");
                    continue;
                }
                if thread_counter.load(SeqCst) < max_threads {

                    let tx2    = tx.clone();
                    let stop2  = stop.clone();
                    let tt_r2  = tt_r.clone();
                    let tt_w2  = tt_w.clone();

                    trace!("spawning thread: (sid: {}) = cur_depth {:?}", sid, cur_depth);
                    s.builder()
                        // .stack_size(4 * 1024 * 1024) // 4 MiB
                        // .stack_size(1 * 512 * 1024) // 0.5 MiB
                        // .stack_size(64 * 1024) // 64 KiB
                        .spawn(move |_| {
                            self._lazy_smp_single_negamax(
                                // self._lazy_smp_single_aspiration(
                                &ts, cur_depth, tx2, tt_r2, tt_w2.clone());
                        }).unwrap();

                    thread_counter.fetch_add(1, SeqCst);

                    search_id.fetch_update(SeqCst, SeqCst, |sid| {
                        if sid >= max_threads - 1 {
                            Some(0)
                        } else { Some(sid + 1) }
                    }).unwrap();

                } else {
                    std::thread::sleep(sleep_time);
                }

                #[cfg(feature = "keep_stats")]
                timer.update_times(self.side, stats.nodes);
                #[cfg(not(feature = "keep_stats"))]
                timer.update_times(self.side, 0);
            }

            rx_thread.join().unwrap();

        }).unwrap();

        let (d,mut out,mut stats) = {
            let r = out.read();
            r.clone()
        };
        stats!(stats.max_depth = d);

        match out {
            ABResults::ABList(best, ress) => {

                debug!("finished lazy_smp_negamax: moves {:?}", best.moves);

                ((best,ress),stats,(tt_r,tt_w))
            },
            ABResults::ABSingle(_) => {
                panic!("single result only?");
            }
            _ => {
                // ((ABResults::ABNone, vec![]), stats, (tt_r,tt_w))
                unimplemented!()
            },
        }
    }

}

/// Misc
impl Explorer {
}

