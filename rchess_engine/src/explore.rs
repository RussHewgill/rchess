
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;

pub use crate::move_ordering::*;
pub use crate::timer::*;
pub use crate::trans_table::*;
pub use crate::searchstats::*;

use std::collections::VecDeque;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Instant,Duration};
use atom::Atom;

use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use crossbeam::thread::ScopedJoinHandle;

use either::Either;

use itertools::Itertools;
use parking_lot::{Mutex,RwLock};

use dashmap::{DashMap,DashSet};

use rand::prelude::{SliceRandom,thread_rng};
use rayon::prelude::*;

// use rustc_hash::FxHashMap;
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
    move_history:      VecDeque<(Zobrist, Move)>,
    // pub prev_eval:     Arc<>
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
            move_history:   VecDeque::default(),
        }
    }
}

/// Entry points
impl Explorer {

    pub fn explore(&self, ts: &Tables, depth: Option<Depth>) -> (Option<(Move,Score)>,SearchStats) {

        // let (moves,stats) = self.iterative_deepening(&ts, false, false);
        // (moves.get(0).map(|x| x.0),stats)

        let (moves,stats,_) = self.lazy_smp(&ts, false, false);
        // let mv = moves.get(0).map(|x| x.0);
        let mv = moves.get(0).map(|x| (x.0,x.2));
        debug!("explore: best move = {:?}", mv);
        (mv,stats)


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

/// Lazy SMP
impl Explorer {

    #[cfg(feature = "aspiration")]
    fn _lazy_smp_single_aspiration(
        &self,
        ts:               &Tables,
        mut gs:           Vec<(Move,Game)>,
        prev_best:        Option<Score>,
        depth:            Depth,
        tx:               Sender<(Depth,Vec<(Move,Vec<Move>,Score)>,Option<(Move,Score)>)>,
        stats:            Arc<RwLock<SearchStats>>,
        tt_r:             TTRead,
        tt_w:             TTWrite,
    ) {
        let mut out: Vec<(Move,Vec<Move>,Score)> = vec![];

        let mut best_alpha = i32::MIN;
        let mut best_beta  = i32::MAX;

        let (mut alpha,mut beta) = match prev_best {
            Some(prev) => {
                let delta = 100;
                let alpha = i32::max(prev - delta, i32::MIN);
                let beta  = i32::min(prev + delta, i32::MAX);
                trace!("Using aspiration window ({}, {})", alpha, beta);
                (alpha,beta)
            },
            None       => (i32::MIN,i32::MAX),
        };
        let mut ss = SearchStats::default();

        let (mv,score): (Move,Score) = 'outer: loop {
            for (mv,g2) in gs.iter() {

                match self.check_tt(&ts, &g2, depth, false, &tt_r, &mut ss) {
                    Some((SICanUse::UseScore,si)) => {
                        out.push((*mv,si.moves,si.score));
                        continue;
                    },
                    _ => {},
                }

                if let Some((mut mv_seq,score)) = {

                    let (mut alpha,mut beta) = (i32::MIN,i32::MAX);

                    self._ab_search(
                        // &ts, &g2, depth, depth, 1, alpha, beta, false, &mut ss, *mv,
                        &ts, &g2, depth, depth, 1, alpha, beta, false, &mut ss, vec![(g.zobrist, *mv)],
                        &tt_r, tt_w.clone()).map(|x| x.0)

                } {
                    mv_seq.push(*mv);
                    mv_seq.reverse();

                    // XXX: ??
                    Self::tt_insert_deepest(
                        &tt_r, tt_w.clone(),
                        g2.zobrist, SearchInfo::new(*mv, mv_seq.clone(), depth, Node::Root, score));

                    out.push((*mv,mv_seq,score));
                } else {
                    break;
                }
            }

            if out.len() == 0 {
                trace!("Aspiration window failed (no moves) a/b {}, {}",
                       alpha, beta);
                if alpha == i32::MIN && beta == i32::MAX {
                    {
                        let mut s = stats.write();
                        *s = *s + ss;
                        s.max_depth = depth;
                    }
                    return;
                }
                alpha = i32::MIN;
                beta  = i32::MAX;
            } else {
                let (mv,_,score) = out.iter().max_by(|a,b| a.2.cmp(&b.2)).unwrap();
                let (mv,score) = (*mv,*score);

                if score > 99_000_000 || score < -99_000_000 {
                    break 'outer (mv,score);
                } else if score <= alpha {
                    trace!("Aspiration window failed with score/a/b {}, {}, {}",
                            score, alpha, beta);
                    // fail low
                    ss.window_fails.0 += 1;
                    alpha = i32::MIN;
                } else if score >= beta {
                    // fail high
                    ss.window_fails.1 += 1;
                    beta = i32::MAX;
                } else {
                    break 'outer (mv,score);
                }
            }

        };

        {
            let mut s = stats.write();
            *s = *s + ss;
            s.max_depth = depth;
        }

        trace!("sending tx, depth = {}", depth);
        match tx.send((depth,out,Some((mv,score)))) {
            Ok(_)  => {},
            Err(_) => {},
        }
        drop(tx);
    }

    fn _lazy_smp_single(
        &self,
        ts:               &Tables,
        mut gs:           Vec<(Move,Game)>,
        prev_best:        Option<Score>,
        depth:            Depth,
        // tx:               Sender<(Depth,Vec<(Move,Vec<Move>,Score)>)>,
        tx:               Sender<(Depth,Vec<(Move,Vec<Move>,Score)>,Option<(Move,Score)>)>,
        stats:            Arc<RwLock<SearchStats>>,
        tt_r:             TTRead,
        tt_w:             TTWrite,
    ) {

        // // XXX: wtf
        // let mut rng = thread_rng();
        // gs.shuffle(&mut rng);

        let mut out: Vec<(Move,Vec<Move>,Score)> = vec![];

        let (alpha,beta) = (i32::MIN,i32::MAX);
        let mut ss = SearchStats::default();

        let mut history = [[[0; 64]; 64]; 2];

        for (mv,g2) in gs.iter() {

            match self.check_tt(&ts, &g2, depth, false, &tt_r, &mut ss) {
                Some((SICanUse::UseScore,si)) => {
                    out.push((*mv,si.moves,si.score));
                    // out.push((*mv,si.moves.to_vec(),si.score));
                    continue;
                },
                _ => {},
            }

            if let Some((mut mv_seq,score)) = {

                let (mut alpha,mut beta) = (i32::MIN,i32::MAX);

                self._ab_search(
                    // &ts, &g2, depth, depth, 1, alpha, beta, false, &mut ss, *mv,
                    &ts, &g2, depth, depth, 1, alpha, beta, false, &mut ss,
                    VecDeque::from([(self.game.zobrist,*mv)]),
                    &mut history,
                    &tt_r, tt_w.clone()).map(|x| x.0)

            } {
                mv_seq.push(*mv);
                mv_seq.reverse();

                // XXX: ??
                Self::tt_insert_deepest(
                    &tt_r, tt_w.clone(),
                    g2.zobrist, SearchInfo::new(*mv, mv_seq.clone(), depth, Node::Root, score));

                out.push((*mv,mv_seq,score));
            } else {
                break;
            }
        }

        {
            let mut s = stats.write();
            *s = *s + ss;
            s.max_depth = depth;
        }

        if out.len() == 0 {
            return;
        }
        let (mv,_,score) = out.iter().max_by(|a,b| a.2.cmp(&b.2)).unwrap();
        let (mv,score) = (*mv,*score);

        trace!("sending tx, depth = {}", depth);
        match tx.send((depth,out,Some((mv,score)))) {
            Ok(_)  => {},
            Err(_) => {},
        }
        drop(tx);
    }

    #[allow(unused_doc_comments)]
    pub fn lazy_smp(&self, ts: &Tables, print: bool, strict_depth: bool)
                    -> (Vec<(Move,Vec<Move>,Score)>,SearchStats,(TTRead,TTWrite)) {
        // 12, 6
        let (n_cpus,np_cpus)  = (num_cpus::get() as u8,num_cpus::get_physical() as u8);

        // self.trans_table.clear();

        // self.trans_table_w.purge();
        // self.trans_table_w.refresh();

        let mut timer = self.timer.clone();
        timer.reset();

        let mut depth = 1;

        // let results: Arc<RwLock<(Depth,Vec<(Move,Vec<Move>,Score)>)>> =
        //     Arc::new(RwLock::new((depth,vec![])));

        let stats: Arc<RwLock<SearchStats>> =
            Arc::new(RwLock::new(SearchStats::default()));

        let moves = self.game.search_all(&ts, None).get_moves_unsafe();

        let mut gs = moves.par_iter().flat_map(|mv| {
            if let Ok(g2) = self.game.make_move_unchecked(&ts, *mv) {
                Some((*mv,g2))
            } else { None }
        });

        // // XXX: captures first is slower?
        // let gs0 = gs.clone().filter(|(m,_)| m.filter_all_captures());
        // let gs1 = gs.filter(|(m,_)| !m.filter_all_captures());
        // let gs = gs0.chain(gs1).collect::<Vec<_>>();

        let gs = gs.collect::<Vec<_>>();

        let threadcount = vec![
            // 6,
            // 4,
            // 2,
            3,
            2,
            1,
        ];

        if print { eprintln!("threadcount = {:?}", threadcount); }

        let out: Arc<RwLock<(Depth,Vec<_>,Option<Score>)>> = Arc::new(RwLock::new((0, vec![], None)));

        let (tt_r, tt_w) = evmap::new();
        let tt_w = Arc::new(Mutex::new(tt_w));

        // let sleep_time = Duration::from_millis(10);
        let sleep_time = Duration::from_millis(1);

        crossbeam::scope(|s| {


            // let (tx,rx): (Sender<ScopedJoinHandle<(Depth,Vec<(Move,Vec<Move>,Score)>)>>,
            //               Receiver<ScopedJoinHandle<(Depth,Vec<(Move,Vec<Move>,Score)>)>>) =
            //     crossbeam::channel::bounded(np_cpus * 2);

            // let (tx,rx): (Sender<(Depth,Vec<(Move,Vec<Move>,Score)>)>,
            //               Receiver<(Depth,Vec<(Move,Vec<Move>,Score)>)>) =
            let (tx,rx): (Sender<(Depth,Vec<(Move,Vec<Move>,Score)>,Option<(Move,Score)>)>,
                          Receiver<(Depth,Vec<(Move,Vec<Move>,Score)>,Option<(Move,Score)>)>) =
                crossbeam::channel::bounded(np_cpus as usize * 2);

            // let mut thread_counter: u8 = 0;

            self.stop.store(false, SeqCst);

            let out1            = out.clone();
            let out2            = out.clone();
            let thread_counter  = Arc::new(AtomicU8::new(0));
            let thread_counter2 = thread_counter.clone();
            let search_id       = Arc::new(AtomicU8::new(0));
            let stop            = self.stop.clone();
            let stop2           = self.stop.clone();
            let best_depth      = Arc::new(AtomicU8::new(0));
            let best_depth2     = best_depth.clone();

            #[cfg(feature = "one_thread")]
            let max_threads = 1;
            #[cfg(not(feature = "one_thread"))]
            let max_threads = np_cpus;
            // let max_threads = np_cpus - 4;
            // let max_threads = 1;

            let depths = vec![
                0, 1, 0, 2, 0, 1,
                0, 1, 0, 2, 0, 1,
                // 2, 1, 0, 0, 1, 0,
            ];
            let depths_largest: Depth = *depths.iter().max().unwrap();

            let t0 = Instant::now();
            let t_max = self.timer.settings.increment[self.side];
                // + (self.timer.settings.safety / 2.0);
            let t_max = Duration::from_secs_f64(t_max);

            s.spawn(move |_| loop {
                match rx.try_recv() {
                    Err(TryRecvError::Empty)    => {
                        // std::thread::sleep(sleep_time);
                        std::thread::sleep(Duration::from_millis(1));
                    },
                    Err(TryRecvError::Disconnected)    => {
                        trace!("Breaking thread counter loop (Disconnect)");
                        break;
                    },
                    Ok((depth,mut scores,mv)) => {
                        if stop2.load(SeqCst) {
                            trace!("Breaking thread counter loop (Force Stop)");
                            break;
                        }
                        if scores.len() > 0 && depth > best_depth2.load(SeqCst) {
                            best_depth2.store(depth,SeqCst);

                            let (mv,score) = mv.unwrap();

                            // let (mv,_,score) = scores.iter().max_by(|a,b| a.2.cmp(&b.2)).unwrap();
                            // let score = *score;

                            // let (mv,_,score) = scores.get(0).unwrap();
                            // trace!("new best move ({}), {:?} = {:?}", depth, score, mv);
                            debug!("new best move d({}): {:.3}s: {}: {:?}",
                                   depth, t0.elapsed().as_secs_f64(), score, mv,
                            );

                            // let (worst,_,_) = scores.iter().min_by(|a,b| a.2.cmp(&b.2)).unwrap();
                            // debug!("worst move: {:?}", worst);

                            // if (self.side == White && score > 100_000_000 - 50)
                            //     || (self.side == Black && score < -100_000_000 + 50) {
                            if score > 100_000_000 - 50 {
                                    let k = 100_000_000 - score.abs();
                                    debug!("Found mate in {}: d({}), {:?}", k, depth, mv);
                                    let mut best = self.best_mate.write();
                                    *best = Some(k as u8);
                                    let mut w = out1.write();
                                    *w = (depth, scores, Some(score));
                                    // *w = (depth, scores, None);
                                    break;
                            } else {
                                    let mut w = out1.write();
                                    *w = (depth, scores, Some(score));
                                    // *w = (depth, scores, None);
                            }
                        }
                        thread_counter2.fetch_sub(1, SeqCst);
                        trace!("decrementing thread counter, new val = {}", thread_counter2.load(SeqCst));
                    },
                }
            });

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
                                break 'outer;
                            }
                        }

                        if thread_counter.load(SeqCst) == 0 {
                            let d = best_depth.load(SeqCst);
                            debug!("breaking loop (Depth -> Threads),  d: {}, t0: {:.3}",
                                   d, t0.elapsed().as_secs_f64());
                            stop.store(true, SeqCst);
                            break 'outer;
                        }
                        if t0.elapsed() > t_max {
                            let d = best_depth.load(SeqCst);
                            debug!("breaking loop (Depth -> Time),  d: {}, t0: {:.3}",
                                   d, t0.elapsed().as_secs_f64());
                            stop.store(true, SeqCst);
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
                    // Only force threads to stop if out of time
                    stop.store(true, SeqCst);
                    drop(tx);
                    break;
                }
                if cur_depth > self.max_depth {
                    continue;
                }
                if thread_counter.load(SeqCst) < max_threads {

                    let gs2    = gs.clone();
                    let stats2 = stats.clone();
                    let tx2    = tx.clone();
                    let stop2  = stop.clone();
                    let tt_r2  = tt_r.clone();
                    let tt_w2  = tt_w.clone();

                    let best: Option<Score> = {
                        let r = out2.read();
                        r.2
                    };

                    trace!("spawning thread: (sid: {}) = cur_depth {:?}", sid, cur_depth);
                    s.spawn(move |_| {
                        self._lazy_smp_single(
                        // self._lazy_smp_single_aspiration(
                            &ts, gs2, best, cur_depth, tx2, stats2, tt_r2, tt_w2);
                    });

                    thread_counter.fetch_add(1, SeqCst);

                    search_id.fetch_update(SeqCst, SeqCst, |sid| {
                        if sid >= max_threads - 1 {
                            Some(0)
                        } else { Some(sid + 1) }
                    }).unwrap();

                } else {
                    std::thread::sleep(sleep_time);
                }

                timer.update_times(self.side, stats.read().nodes);
            }

        }).unwrap();

        let (d,mut out,_) = {
            let r = out.read();
            r.clone()
        };

        let mut stats = stats.read().clone();
        stats.max_depth = d;

        #[cfg(feature = "par")]
        out.par_sort_unstable_by(|a,b| a.2.cmp(&b.2));
        #[cfg(not(feature = "par"))]
        out.sort_unstable_by(|a,b| a.2.cmp(&b.2));

        out.reverse();

        // for (mv,_,score) in out.iter() {
        //     debug!("mv: {}: {:?}", score, mv);
        // }

        // if out.len() == 0 {
        //     debug!("no moves found");
        // }

        if out.len() == 0 {
            debug!("no moves found");
            // panic!("no moves found");
            let mut gs = gs;
            gs.sort_unstable_by(|a,b| {
                let x = a.1.evaluate(&ts).sum();
                let y = b.1.evaluate(&ts).sum();
                x.cmp(&y)
            });
            let out = vec![(gs[0].0,vec![],gs[0].1.evaluate(&ts).sum())];
            (out, stats, (tt_r,tt_w))
        } else {
            (out, stats, (tt_r,tt_w))
        }

        // (out, stats, (tt_r,tt_w))
    }

}

/// Quiescence
impl Explorer {

    #[allow(unused_doc_comments)]
    #[allow(unreachable_code)]
    pub fn quiescence(
        &self,
        ts:             &Tables,
        g:              &Game,
        ms:             Vec<Move>,
        k:              i16,
        mut alpha:      i32,
        mut beta:       i32,
        maximizing:     bool,
        mut stats:      &mut SearchStats,
        m0:             Move,
    ) -> Score {
        trace!("quiescence {}", k);

        stats.qt_nodes += 1;
        let stand_pat = g.evaluate(&ts).sum();
        // stats.leaves += 1;
        return stand_pat; // correct

        if stand_pat >= beta {
            // debug!("quiescence beta cutoff: {}", k);
            // return score; // fail soft
            trace!("Quiescence returning beta: {}", beta);
            return beta; // fail hard
        }
        // return stand_pat; // correct

        // /// Delta prune
        // let mut big_delta = Queen.score();
        // if m0.filter_promotion() {
        //     big_delta += Queen.score() - Pawn.score();
        // }
        // if !maximizing {
        //     if stand_pat >= (beta + big_delta) {
        //         return beta;
        //     }
        // }
        // // return stand_pat; // correct

        if alpha < stand_pat {
            alpha = stand_pat;
        }
        // return stand_pat;

        let mut captures = ms.into_iter().filter(|m| m.filter_all_captures()).collect::<Vec<_>>();

        // // TODO: sort reverse for max / min ?
        // #[cfg(feature = "par")]
        // // captures.par_sort_unstable();
        // captures.par_sort();
        // #[cfg(not(feature = "par"))]
        // captures.sort();

        // captures.reverse();

        // if !maximizing {
        //     captures.reverse();
        // }

        for mv in captures.into_iter() {
            if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
                match g2.search_all(&ts, None) {
                    Outcome::Moves(ms2) => {
                        // stats.nodes += 1;

                        // let eval = -self.quiescence(
                        //     &ts, &g2, ms2, k + 1, -alpha, -beta,
                        //     !maximizing, &mut stats, mv);

                        let score = self.quiescence(
                            &ts, &g2, ms2, k + 1, alpha, beta,
                            !maximizing, &mut stats, mv);

                        if maximizing {
                            if score > alpha {
                                alpha = score;
                            }
                            if score >= beta {
                                return beta;
                            }
                        } else {
                            if score < beta {
                                beta = score;
                            }
                            if score <= alpha {
                                return alpha;
                            }
                        }

                    },
                    Outcome::Checkmate(_) => {
                        // panic!("checkmate in quiescent");
                    },
                    Outcome::Stalemate => {
                        // panic!("stalemate in quiescent");
                    },
                }
            }
        }

        // debug!("quiescence return alpha: {}", k);
        alpha
    }

}

/// Pruning
impl Explorer {

    pub fn prune_delta(&self, ts: &Tables, g: Game) -> bool {
        unimplemented!()
    }


}

/// Misc
impl Explorer {
}

