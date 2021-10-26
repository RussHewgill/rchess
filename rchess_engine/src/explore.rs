
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;
pub use crate::timer::*;
pub use crate::trans_table::*;
pub use crate::searchstats::*;

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
}

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
        }
    }
}

/// Entry points
impl Explorer {

    pub fn explore(&self, ts: &Tables, depth: Option<Depth>) -> (Option<Move>,SearchStats) {

        // let (moves,stats) = self.iterative_deepening(&ts, false, false);
        // (moves.get(0).map(|x| x.0),stats)

        let (moves,stats,_) = self.lazy_smp(&ts, false, false);
        let mv = moves.get(0).map(|x| x.0);
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

    pub fn rank_moves(&self, ts: &Tables, print: bool) -> (Vec<(Move,Score)>,SearchStats) {
        let moves = self.game.search_all(&ts, None);

        if moves.is_end() {
            return (vec![], SearchStats::default());
        }
        let moves = moves.get_moves_unsafe();

        // self.rank_moves_list(&ts, print, moves, par)
        // self.rank_moves_list(&ts, print, moves)
        unimplemented!()
    }

}

/// iterative_deepening
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

        while (depth <= self.max_depth) && (strict_depth || (|| timer.should_search(self.side, depth))()) {
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
                    let (mut mv_seq,score) = {
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

    fn _lazy_smp_single(
        &self,
        ts:               &Tables,
        mut gs:           Vec<(Move,Game)>,
        depth:            Depth,
        tx:               Sender<(Depth,Vec<(Move,Vec<Move>,Score)>)>,
        stats:            Arc<RwLock<SearchStats>>,
        tt_r:             TTRead,
        tt_w:             TTWrite,
    ) {

        // // XXX: wtf
        // let mut rng = thread_rng();
        // gs.shuffle(&mut rng);

        let mut out = vec![];

        let mut best_alpha = i32::MIN;
        let mut best_beta  = i32::MAX;

        let mut ss = SearchStats::default();
        for (mv,g2) in gs.into_iter() {

            match self.check_tt(&ts, &g2, depth, false, &tt_r, &mut ss) {
                Some((SICanUse::UseScore,si)) => {
                    out.push((mv,si.moves,si.score));
                    continue;
                },
                _ => {},
            }

            if let Some((mut mv_seq,score)) = {
                let alpha = i32::MIN;
                let beta  = i32::MAX;
                self._ab_search(
                    &ts, &g2, depth, depth, 1, alpha, beta, false, &mut ss, mv,
                    &tt_r, tt_w.clone(),
                )
            } {
                mv_seq.push(mv);
                mv_seq.reverse();

                Self::tt_insert_deepest(
                    &tt_r, tt_w.clone(),
                    g2.zobrist, SearchInfo::new(mv, mv_seq.clone(), depth, Node::Root, score));

                out.push((mv,mv_seq,score));
            } else {
                break;
            }
        }

        {
            let mut s = stats.write();
            *s = *s + ss;
            s.max_depth = depth;
        }

        trace!("sending tx, depth = {}", depth);
        match tx.send((depth,out)) {
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

        // #[cfg(feature = "par")]
        // let mut gs: Vec<(Move,Game)> = moves.par_iter().flat_map(|mv| {
        //     if let Ok(g2) = self.game.make_move_unchecked(&ts, *mv) {
        //         Some((*mv,g2))
        //     } else { None }
        // }).collect();
        // #[cfg(not(feature = "par"))]
        // let mut gs: Vec<(Move,Game)> = moves.iter().flat_map(|mv| {
        //     if let Ok(g2) = self.game.make_move_unchecked(&ts, *mv) {
        //         Some((*mv,g2))
        //     } else { None }
        // }).collect();

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

        let out = Arc::new(RwLock::new((0,vec![])));

        let (tt_r, tt_w) = evmap::new();
        let tt_w = Arc::new(Mutex::new(tt_w));

        let sleep_time = Duration::from_millis(10);

        crossbeam::scope(|s| {

            let out = out.clone();

            // let (tx,rx): (Sender<ScopedJoinHandle<(Depth,Vec<(Move,Vec<Move>,Score)>)>>,
            //               Receiver<ScopedJoinHandle<(Depth,Vec<(Move,Vec<Move>,Score)>)>>) =
            //     crossbeam::channel::bounded(np_cpus * 2);

            let (tx,rx): (Sender<(Depth,Vec<(Move,Vec<Move>,Score)>)>,
                          Receiver<(Depth,Vec<(Move,Vec<Move>,Score)>)>) =
                crossbeam::channel::bounded(np_cpus as usize * 2);

            // let mut thread_counter: u8 = 0;

            self.stop.store(false, SeqCst);

            let thread_counter  = Arc::new(AtomicU8::new(0));
            let thread_counter2 = thread_counter.clone();
            let search_id       = Arc::new(AtomicU8::new(0));
            let stop            = self.stop.clone();
            let stop2           = self.stop.clone();
            let best_depth      = Arc::new(AtomicU8::new(0));
            let best_depth2     = best_depth.clone();

            let max_threads = np_cpus;
            // let max_threads = 1;

            let depths = vec![
                0, 1, 0, 2, 0, 1,
                // 2, 1, 0, 0, 1, 0,
            ];

            let t0 = Instant::now();
            let t_max = self.timer.settings.increment[self.side]
                + self.timer.settings.safety;
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
                    Ok((depth,mut scores)) => {
                        if stop2.load(SeqCst) {
                            trace!("Breaking thread counter loop (Force Stop)");
                            break;
                        }
                        if scores.len() > 0 && depth > best_depth2.load(SeqCst) {
                            best_depth2.store(depth,SeqCst);

                            let (mv,_,score) = scores.iter().max_by(|a,b| a.2.cmp(&b.2)).unwrap();

                            // let (mv,_,score) = scores.get(0).unwrap();
                            // trace!("new best move ({}), {:?} = {:?}", depth, score, mv);
                            debug!("new best move d({}): {:.3}s: {}: {:?}",
                                   depth, t0.elapsed().as_secs_f64(), score, mv,
                            );

                            // let (worst,_,_) = scores.iter().min_by(|a,b| a.2.cmp(&b.2)).unwrap();
                            // debug!("worst move: {:?}", worst);

                            if (self.side == White && *score > 100_000_000 - 50)
                                || (self.side == Black && *score < -100_000_000 + 50) {
                                    let k = 100_000_000 - score.abs();
                                    debug!("Found mate in {}: d({}), {:?}", k, depth, mv);
                                    let mut best = self.best_mate.write();
                                    *best = Some(k as u8);
                                    let mut w = out.write();
                                    *w = (depth, scores);
                                    break;
                            } else {
                                    let mut w = out.write();
                                    *w = (depth, scores);
                            }
                        }
                        thread_counter2.fetch_sub(1, SeqCst);
                        trace!("decrementing thread counter, new val = {}", thread_counter2.load(SeqCst));
                    },
                }
            });

            // let mut k = 3;
            'outer: loop {
                // if timer.should_search(self.side, depth)

                // let cur_depth = best_depth.load(SeqCst) + 1;
                // let tc        = thread_counter.load(SeqCst);

                let sid       = search_id.load(SeqCst);
                // let cur_depth = depth + 1 + sid.trailing_zeros() as u8;
                let cur_depth = best_depth.load(SeqCst) + 1 + depths[sid as usize];

                {
                    let r = self.best_mate.read();
                    if r.is_some() {
                        stop.store(true, SeqCst);
                        break 'outer;
                    }
                }

                if best_depth.load(SeqCst) > self.max_depth {
                    let d = best_depth.load(SeqCst);
                    debug!("breaking loop (Depth), d: {}, t0: {:.3}", d, t0.elapsed().as_secs_f64());
                    drop(tx);

                    loop {
                        if t0.elapsed() > t_max {
                            debug!("breaking loop (Depth -> Time),  d: {}, t0: {:.3}",
                                   d, t0.elapsed().as_secs_f64());
                            stop.store(true, SeqCst);
                            break 'outer;
                        }
                        std::thread::sleep(sleep_time);
                    }

                    // break 'outer;
                } else if t0.elapsed() > t_max {
                    let d = best_depth.load(SeqCst);
                    debug!("breaking loop (Time),  d: {}, t0: {:.3}", d, t0.elapsed().as_secs_f64());
                    // Only force threads to stop if out of time
                    stop.store(true, SeqCst);
                    drop(tx);
                    break;
                } else if thread_counter.load(SeqCst) < max_threads {
                    let gs2    = gs.clone();
                    let stats2 = stats.clone();
                    let tx2    = tx.clone();
                    let stop2  = stop.clone();
                    let tt_r2  = tt_r.clone();
                    let tt_w2  = tt_w.clone();

                    trace!("spawning thread: (sid: {}) = cur_depth {:?}", sid, cur_depth);
                    s.spawn(move |_| {
                        self._lazy_smp_single(
                            &ts, gs2, cur_depth, tx2, stats2, tt_r2, tt_w2);
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

        let (d,mut out) = {
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

        if out.len() == 0 {
            debug!("no moves found");
        }

        // if out.len() == 0 {
        //     debug!("no moves found");
        //     // panic!("no moves found");
        //     let mut gs = gs;
        //     gs.sort_unstable_by(|a,b| {
        //         let x = a.1.evaluate(&ts).sum();
        //         let y = b.1.evaluate(&ts).sum();
        //         x.cmp(&y)
        //     });
        //     let out = vec![(gs[0].0,vec![],gs[0].1.evaluate(&ts).sum())];
        //     (out,stats)
        // } else {
        //     (out,stats)
        // }

        (out, stats, (tt_r,tt_w))
    }

}

/// AB search
impl Explorer {

    /// returns (can_use, SearchInfo)
    /// true: when score can be used instead of searching
    /// false: score can be used ONLY for move ordering
    fn check_tt(&self,
                ts:             &Tables,
                g:              &Game,
                depth:          Depth,
                maximizing:     bool,
                tt_r:           &TTRead,
                mut stats:      &mut SearchStats,
    ) -> Option<(SICanUse,SearchInfo)> {
        // if let Some(si) = self.trans_table.tt_get(&g.zobrist) {

        // if g.zobrist == Zobrist(0x1eebfbac03c62e9d) { println!("wat wat 0 {}", depth); }

        // if let Some(si) = self.trans_table.get(&g.zobrist) {
        if let Some(si) = tt_r.get_one(&g.zobrist) {

            // if si.depth_searched == depth {
            if si.depth_searched >= depth {
                // if g.zobrist == Zobrist(0x1eebfbac03c62e9d) { println!("wat wat 1"); }
                stats.tt_hits += 1;
                // Some((SICanUse::UseScore,*si))
                Some((SICanUse::UseScore,si.clone()))
            } else {
                // if g.zobrist == Zobrist(0x1eebfbac03c62e9d) { println!("wat wat 2"); }
                stats.tt_misses += 1;
                // Some((SICanUse::UseOrdering,*si))
                Some((SICanUse::UseOrdering,si.clone()))
                // None
            }

        } else {
            // if g.zobrist == Zobrist(0x1eebfbac03c62e9d) { println!("wat wat 3"); }
            stats.tt_misses += 1;
            // let score = self._ab_search(&ts, &g, depth - 1, k + 1, alpha, beta, !maximizing);
            // score
            None
        }
    }

    /// alpha: the MIN score that the maximizing player is assured of
    /// beta:  the MAX score that the minimizing player is assured of
    pub fn _ab_search(
        &self,
        ts:                 &Tables,
        g:                  &Game,
        max_depth:          Depth,
        depth:              Depth,
        k:                  i16,
        mut alpha:          i32,
        mut beta:           i32,
        maximizing:         bool,
        mut stats:          &mut SearchStats,
        mv0:                Move,
        tt_r:               &TTRead,
        tt_w:               TTWrite,
    ) -> Option<(Vec<Move>, Score)> {
    // ) -> Option<((Vec<Move>, Score), (i32, i32))> {

        if self.stop.load(SeqCst) {
            return None;
        }

        {
            let r = self.best_mate.read();
            if let Some(best) = *r {
                drop(r);
                if best <= max_depth {
                    debug!("halting search of depth {}, faster mate found", max_depth);
                    return None;
                }
            }
        }

        let moves = g.search_all(&ts, None);

        let moves: Vec<Move> = match moves {
            Outcome::Checkmate(c) => {
                let score = 100_000_000 - k as Score;
                // if !self.tt_contains(&g.zobrist) {
                if !tt_r.contains_key(&g.zobrist) {
                    stats.leaves += 1;
                    stats.checkmates += 1;
                }
                if maximizing {
                    return Some((vec![mv0], -score));
                } else {
                    return Some((vec![mv0], score));
                }

            },
            Outcome::Stalemate    => {
                let score = -100_000_000 + k as Score;
                // if !self.tt_contains(&g.zobrist) {
                if !tt_r.contains_key(&g.zobrist) {
                    stats.leaves += 1;
                    stats.stalemates += 1;
                }
                return Some((vec![],score));
            },
            Outcome::Moves(ms)    => ms,
        };

        if depth == 0 {
            let score = g.evaluate(&ts).sum();

            // if !self.tt_contains(&g.zobrist) {
            if !tt_r.contains_key(&g.zobrist) {
                stats.leaves += 1;
            }
            if self.side == Black {
                // return (vec![mv0], -score);
                return Some((vec![], -score));
            } else {
                // return (vec![mv0], score);
                return Some((vec![], score));
            }
        }

        // #[cfg(feature = "par")]
        let mut gs: Vec<(Move,Game,Option<(SICanUse,SearchInfo)>)> = {
            // let mut gs0 = moves.into_par_iter().flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, m) {
            let mut gs0 = moves.into_iter().flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, m) {
                let mut ss = SearchStats::default();
                let tt = self.check_tt(&ts, &g2, depth, maximizing, &tt_r, &mut ss);
                // Some(((m,g2,tt), ss))
                *stats = *stats + ss;
                Some((m,g2,tt))
            } else {
                None
            });
            gs0.collect()
        };

        // // #[cfg(not(feature = "par"))]
        // let mut gs: Vec<(Move,Game,Option<(SICanUse,SearchInfo)>)> = {
        //     let gs = moves.into_iter().flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, &m) {
        //         let mut ss = SearchStats::default();
        //         // let tt = self.check_tt(&ts, &g2, depth, maximizing, &mut stats);
        //         let tt = self.check_tt(&ts, &g2, depth, maximizing, &mut ss);
        //         Some((m,g2,tt))
        //     } else {
        //         None
        //     });
        //     gs.collect()
        // };

        // #[cfg(feature = "par")]
        // gs.par_sort_unstable_by(|a,b| a.2.partial_cmp(&b.2).unwrap());
        // #[cfg(not(feature = "par"))]
        // gs.sort_unstable_by(|a,b| a.2.partial_cmp(&b.2).unwrap());
        // if !maximizing {
        //     gs.reverse();
        // }

        Explorer::order_searchinfo(maximizing, &mut gs[..]);

        let mut node_type = Node::PV;

        // let moves = match self.trans_table.get(&g.zobrist) {
        let moves = match tt_r.get_one(&g.zobrist) {
            None     => {},
            Some(si) => {
                match si.node_type {
                    Node::Cut => {
                        gs.truncate(1);
                    },
                    // All children of an All node are Cut nodes
                    Node::All => node_type = Node::Cut,
                    _         => {},
                }
            }
        };

        let mut val = if maximizing { i32::MIN } else { i32::MAX };
        let mut val: (Option<(Zobrist,Move,Vec<Move>)>,i32) = (None,val);

        // {
        //     let (mv,g2,tt) = gs.pop().unwrap();
        //     let zb = g2.zobrist;
        //     stats.nodes += 1;
        //     let alpha2 = Arc::new(AtomicI32::new(alpha.load(Ordering::Relaxed)));
        //     let beta2  = Arc::new(AtomicI32::new(beta.load(Ordering::Relaxed)));
        //     let (mut mv_seq,score) = self._ab_search(
        //         &ts, &g2, depth - 1, k + 1, alpha2, beta2, !maximizing, &mut stats, mv);
        //     let b = self._ab_score(
        //         (mv,&g2,tt),
        //         (mv_seq,score),
        //         &mut val,
        //         depth,
        //         alpha.clone(),
        //         beta.clone(),
        //         maximizing,
        //         mv0);
        // }

        // use crossbeam_channel::unbounded;
        // let (chan_tx,chan_rx) = unbounded();
        // // let (chan_tx,chan_rx) = std::sync::mpsc::sync_channel(4);

        // let mut i_max = gs.len() as i16;
        // // let gs2 = gs.into_par_iter()
        // let gs2 = gs.into_iter()
        //     .enumerate()
        //     .for_each(|(i,(mv,g2,tt))| {
        //     // .map(|(i,(m,g2,tt))| {
        //         let mut ss = SearchStats::default();
        //         let alpha2 = Arc::new(AtomicI32::new(alpha.load(Ordering::SeqCst)));
        //         let beta2  = Arc::new(AtomicI32::new(beta.load(Ordering::SeqCst)));
        //         let score = self._ab_search(
        //             &ts, &g2, depth - 1, k + 1, alpha2, beta2, !maximizing, &mut ss, mv);
        //         chan_tx.send((mv,g2,tt,score,ss)).unwrap();
        //         // (m,g2,tt,score,ss)
        //     });

        // let mut gs2 = vec![];
        // loop {
        //     gs.extend_from_slice(&gs2[..]);
        //     gs2.clear();
        //     if gs2.len() == 0 { break; }
        //     gs.clear();
        // }

        for (mv,g2,tt) in gs.iter() {
        // for (mv,g2,tt) in gs.into_iter() {
        // for (mv,g2,tt,score,ss) in gs2 {
        //     *stats += ss;

        // loop {
        //     if i_max == 0 { break; } else { i_max -= 1; }
        //     let (mv,g2,tt,score,ss) = match chan_rx.recv() {
        //         Ok(x)  => x,
        //         Err(_) => break,
        //     };
        //     *stats += ss;

            let zb = g2.zobrist;
            // if !self.tt_contains(&zb) {
            if !tt_r.contains_key(&g.zobrist) {
                stats.nodes += 1;
            }

            // if self.move_table.contains(depth, zb, *mv) {
            //     gs2.push((*mv,*g2,tt.clone()));
            //     continue;
            // } else {
            //     self.move_table.insert(depth, zb, *mv);
            // }

            let (can_use,mut mv_seq,score) = match tt {
                Some((SICanUse::UseScore,si)) => {
                    // return (si.moves.clone(),si.score);
                    (true,si.moves.clone(),si.score)
                },
                _ => {

                    // XXX: Check extension
                    let depth2 = if g2.state.checkers.unwrap().is_not_empty() && k < 40 {
                        depth
                    } else {
                        depth - 1
                    };

                    let depth2 = depth - 1;

                    if let Some((mv_seq,score)) = self._ab_search(
                        &ts, &g2, max_depth, depth2, k + 1,
                        alpha, beta, !maximizing, &mut stats, *mv,
                        tt_r, tt_w.clone(),
                    ) {
                        (false,mv_seq,score)
                    } else {
                        break;
                    }
                },
            };

            // let alpha2 = Arc::new(AtomicI32::new(alpha.load(Ordering::Relaxed)));
            // let beta2  = Arc::new(AtomicI32::new(beta.load(Ordering::Relaxed)));
            // let (mut mv_seq,score) = self._ab_search(
            //     &ts, &g2, depth - 1, k + 1, alpha2, beta2, !maximizing, &mut stats, mv);

            // if maximizing {
            //     val.1 = i32::max(val.1, score);
            // } else {
            //     val.1 = i32::min(val.1, score);
            // }

            let b = self._ab_score(
                (*mv,&g2),
                (can_use,mv_seq,score),
                &mut val,
                depth,
                &mut alpha,
                &mut beta,
                maximizing,
                mv0);
            if b {
                node_type = Node::Cut;
                // self.move_table.remove(depth, zb, *mv);
                break;
            } else {
                // self.move_table.remove(depth, zb, *mv);
            }

        }


        // XXX: depth or depth - 1 ?
        if let Some((zb,mv,mv_seq)) = &val.0 {
            // if *zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 0 {:?}, {:?}", mv, mv_seq); }
            let mut mv_seq = mv_seq.clone();
            // mv_seq.push(*mv);
            // mv_seq.push(mv0);
            Self::tt_insert_deepest(
                &tt_r, tt_w,
            // self.trans_table.insert(
                // zb, SearchInfo::new(mv, depth, Node::PV, val.1));
                // *zb, SearchInfo::new(*mv, depth, Node::PV, val.1));
                // *zb, SearchInfo::new(*mv,mv_seq.clone(), depth, Node::PV, val.1));
                *zb, SearchInfo::new(*mv, mv_seq, depth - 1, node_type, val.1));
        }

        // match node_type {
        //     Node::PV         => {
        //         if let Some((zb,mv)) = val.0 {
        //             self.trans_table.insert_replace(
        //                 zb, SearchInfo::new(mv, depth, Node::PV, val.1));
        //         }
        //     },
        //     Node::All => {
        //     },
        //     Node::Cut => {
        //     },
        // }

        stats.alpha = stats.alpha.max(alpha);
        stats.beta = stats.beta.max(beta);

        match &val.0 {
            Some((zb,mv,mv_seq)) => Some((mv_seq.clone(),val.1)),
            // _                    => (vec![mv0], val.1),
            // _                    => panic!("_ab_search val is None ?"),
            _                    => None,
        }

    }


    pub fn _ab_score(
        &self,
        (mv,g2):                       (Move,&Game),
        (can_use,mut mv_seq,score):    (bool,Vec<Move>,Score),
        mut val:                       &mut (Option<(Zobrist,Move,Vec<Move>)>,i32),
        depth:                         Depth,
        mut alpha:                     &mut i32,
        mut beta:                      &mut i32,
        maximizing:                    bool,
        mv0:                           Move,
    ) -> bool {
        let zb = g2.zobrist;
        if maximizing {
            // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 0"); }
            if score > val.1 {
                // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 1"); }
                // mv_seq.push(mv);
                if !can_use { mv_seq.push(mv) };
                *val = (Some((zb,mv,mv_seq.clone())),score);
            }

            *alpha = i32::max(*alpha, val.1);
            if val.1 >= *beta { // Beta cutoff
                // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 2"); }
                // self.trans_table.insert(
                //     zb, SearchInfo::new(mv, mv_seq.clone(), depth, Node::Cut, val.1));
                return true;
            }

            // self.trans_table.insert_replace(
            //     zb, SearchInfo::new(mv, depth, Node::All, val.1));
        } else {
            // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 3"); }
            if score < val.1 {
                // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 4"); }
                // mv_seq.push(mv);
                if !can_use { mv_seq.push(mv) };
                *val = (Some((zb,mv,mv_seq.clone())),score);
            }

            *beta = i32::min(*beta, val.1);
            if val.1 <= *alpha {
                // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 5"); }
                // self.trans_table.insert(
                //     zb, SearchInfo::new(mv, mv_seq.clone(), depth, Node::Cut, val.1));
                return true;
            }

            // node_type = Node::All;
            // self.trans_table.insert_replace(
            //     zb, SearchInfo::new(mv, depth, Node::All, val.1));
        }
        false
    }

}

/// Quiescence
impl Explorer {

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
        // debug!("quiescence {}", k);

        stats.qt_nodes += 1;
        let stand_pat = g.evaluate(&ts).sum();
        // stats.leaves += 1;
        return stand_pat; // correct

        if stand_pat >= beta {
            // debug!("quiescence beta cutoff: {}", k);
            // return score; // fail soft
            return beta; // fail hard
        }
        // return stand_pat; // correct

        // Delta prune
        let mut big_delta = Queen.score();
        if m0.filter_promotion() {
            big_delta += Queen.score() - Pawn.score();
        }
        if !maximizing {
            if stand_pat >= (beta + big_delta) {
                return beta;
            }
        }
        // return stand_pat; // correct
        unimplemented!();

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

        captures.reverse();

        // if !maximizing {
        //     captures.reverse();
        // }

        for mv in captures.into_iter() {
            if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
                match g2.search_all(&ts, None) {
                    Outcome::Moves(ms2) => {
                        stats.nodes += 1;
                        let score = -self.quiescence(
                            &ts, &g2, ms2, k + 1, -alpha, -beta,
                            !maximizing, &mut stats, mv);

                        if maximizing {
                            if score >= beta {
                                break;
                            }
                            if score > alpha {
                                alpha = score;
                            }
                        } else {
                            if score <= alpha {
                                break;
                            }
                            if score < beta {
                                beta = score;
                            }
                        }

                        // if score >= beta {
                        //     // stats.leaves += 1;
                        //     return beta;
                        // }
                        // if score > alpha {
                        //     alpha = score;
                        // }

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

    pub fn order_searchinfo(
        maximizing: bool, mut xs: &mut [(Move,Game,Option<(SICanUse,SearchInfo)>)])
    {

        // #[cfg(feature = "par")]
        // xs.par_sort_unstable_by(|a,b| a.2.partial_cmp(&b.2).unwrap());
        // #[cfg(not(feature = "par"))]
        // xs.sort_unstable_by(|a,b| a.2.partial_cmp(&b.2).unwrap());
        // if !maximizing {
        //     xs.reverse();
        // }

        #[cfg(feature = "par")]
        {
            if maximizing {
                xs.par_sort_unstable_by(|a,b| {

                    match (a.2.as_ref(),b.2.as_ref()) {
                        (Some((_,a)),Some((_,b))) => a.score.partial_cmp(&b.score).unwrap(),
                        (a,b)                     => a.partial_cmp(&b).unwrap(),
                    }
                });
                xs.reverse();
            } else {
                xs.par_sort_unstable_by(|a,b| {
                    match (a.2.as_ref(),b.2.as_ref()) {
                        (Some((_,a)),Some((_,b))) => a.score.partial_cmp(&b.score).unwrap().reverse(),
                        (a,b)                     => a.partial_cmp(&b).unwrap(),
                    }
                });
                xs.reverse();
            }
        }

        #[cfg(not(feature = "par"))]
        {
            if maximizing {
                xs.sort_unstable_by(|a,b| {
                    match (a.2.as_ref(),b.2.as_ref()) {
                        // (Some((_,a)),Some((_,b))) => a.partial_cmp(&b).unwrap(),
                        (Some((_,a)),Some((_,b))) => a.score.partial_cmp(&b.score).unwrap(),
                        _                         => a.partial_cmp(&b).unwrap(),
                    }
                });
                xs.reverse();
            } else {
                xs.sort_unstable_by(|a,b| {
                    match (a.2.as_ref(),b.2.as_ref()) {
                        (Some((_,a)),Some((_,b))) => a.score.partial_cmp(&b.score).unwrap().reverse(),
                        _                         => a.partial_cmp(&b).unwrap(),
                    }
                });
                xs.reverse();
            }
        }

    }

    pub fn order_moves(ts: &Tables, g: &Game, maximizing: bool, mut moves: &mut [Move]) {

        // if maximizing {
        //     moves.reverse();
        // } else {
        //     vs.sort_by(|a,b| {
        //         match (a,b) {
        //             (Some(a),Some(b)) => a.partial_cmp(&b).unwrap().reverse(),
        //             _                 => a.partial_cmp(&b).unwrap(),
        //         }
        //     });
        //     moves.reverse();
        // }

    }

    // pub fn order_moves(&self, ts: &Tables, mut moves: &mut [(Move,Score,Game)]) {
    //     use std::cmp::Ordering;
    //     moves.sort_unstable_by(|a,b| {
    //         let a_cap = a.0.filter_all_captures();
    //         let b_cap = b.0.filter_all_captures();
    //         let a_castle = a.0.filter_castle();
    //         let b_castle = b.0.filter_castle();
    //         if a_cap & !b_cap {
    //             Ordering::Greater
    //         } else if !a_cap & b_cap {
    //             Ordering::Less
    //         } else if a_castle & !b_castle {
    //             Ordering::Greater
    //         } else if !a_castle & b_castle {
    //             Ordering::Less
    //         } else {
    //             Ordering::Equal
    //         }
    //     });
    // }

}

