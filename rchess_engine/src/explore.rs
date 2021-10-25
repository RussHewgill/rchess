
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
pub use crate::timer::*;
pub use crate::trans_table::*;
pub use crate::searchstats::*;

use std::hash::Hasher;
use std::iter::successors;
use std::sync::atomic::AtomicU8;
use std::thread;
use std::time::Duration;

use either::Either;

use itertools::Itertools;
use parking_lot::{Mutex,RwLock};

use rand::prelude::{SliceRandom,thread_rng};
use rayon::prelude::*;

// use rustc_hash::FxHashMap;
use evmap::{ReadHandle,WriteHandle};

#[derive(Debug)]
pub struct Explorer {
    pub side:          Color,
    pub game:          Game,
    // pub stats:      ABStats
    pub max_depth:     Depth,
    pub timer:         Timer,
    pub parallel:      bool,
    // pub trans_table:   RwTransTable,
    pub trans_table:   TransTable,
    // pub trans_table:   RwLock<TransTable>,
    // pub trans_table_r:   ReadHandle<Zobrist, SearchInfo>,
    // pub trans_table_w:   WriteHandle<Zobrist, SearchInfo>,
}

impl Explorer {
    pub fn new(side:          Color,
               game:          Game,
               max_depth:     Depth,
               should_stop:   Arc<AtomicBool>,
               settings:      TimeSettings,
    ) -> Self {
        // let tt = RwTransTable::default();
        // let tt = TransTable::default();
        // let tt = TransTable::new();
        // let (tr,tw) = evmap::new();
        Self {
            side,
            game,
            // stats: ABStats::new(),
            max_depth,
            timer: Timer::new(side, should_stop, settings),
            parallel: true,
            trans_table: TransTable::default(),
            // trans_table: tt,
            // trans_table_r: tr,
            // trans_table_w: tw,
        }
    }
}

/// Entry points
impl Explorer {

    pub fn explore(&self, ts: &Tables, depth: Option<Depth>) -> (Option<Move>,SearchStats) {

        // let (moves,stats) = self.iterative_deepening(&ts, false, false);
        // (moves.get(0).map(|x| x.0),stats)

        let (moves,stats) = self.lazy_smp(&ts, false, false);
        (moves.get(0).map(|x| x.0),stats)

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

        self.trans_table.clear();

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

        // std::thread::spawn(|| {
        //     loop {
        //         std::thread::sleep_ms(100);
        //     }
        // });

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

            #[cfg(feature = "par")]
            {
                (out,ss) = gs.par_iter().map(|(mv,g2)| {
                    let mut stats = SearchStats::default();
                    let (mut mv_seq,score) = {
                        let alpha = i32::MIN;
                        let beta  = i32::MAX;
                        self._ab_search(
                            &ts, &g2, depth, 1, alpha, beta, false, &mut stats, *mv)
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
                            &ts, &g2, depth, 1, alpha, beta, false, &mut stats, *mv)
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
        // results:          Arc<RwLock<(Depth,Vec<(Move,Vec<Move>,Score)>)>>,
        stats:            Arc<RwLock<SearchStats>>,
    ) -> (Depth,Vec<(Move,Vec<Move>,Score)>) {

        // // XXX: wtf
        // let mut rng = thread_rng();
        // gs.shuffle(&mut rng);

        let (out,ss): (Vec<(Move,Vec<Move>,Score)>,SearchStats) = {
            let mut stats = SearchStats::default();
            let out = gs.into_iter().map(|(mv,g2)| {
                let mut ss = SearchStats::default();
                let (mut mv_seq,score) = {
                    let alpha = i32::MIN;
                    let beta  = i32::MAX;
                    self._ab_search(
                        &ts, &g2, depth, 1, alpha, beta, false, &mut stats, mv)
                };

                mv_seq.push(mv);
                mv_seq.reverse();
                stats = stats + ss;
                (mv,mv_seq,score)
            }).collect();

            (out,stats)
        };

        let mut s = stats.write();
        *s = *s + ss;
        s.max_depth = depth;

        // let d = results.read().0;
        // if depth > d {
        //     // eprintln!("_lazy_smp_single: 0, depth = {:?}", depth);
        //     drop(d);
        //     let mut r = results.write();
        //     r.0 = depth;
        //     r.1 = out;
        //     let mut s = stats.write();
        //     *s = *s + ss;
        //     s.max_depth = depth;
        // } else {
        //     // eprintln!("_lazy_smp_single: 1, depth = {:?}", depth);
        //     drop(d);
        //     let mut s = stats.write();
        //     *s = *s + ss;
        // }

        (depth, out)
        // unimplemented!()
    }

    #[allow(unused_doc_comments)]
    pub fn lazy_smp(&self, ts: &Tables, print: bool, strict_depth: bool)
                    -> (Vec<(Move,Vec<Move>,Score)>,SearchStats) {
        // 12, 6
        let (n_cpus,np_cpus)  = (num_cpus::get(),num_cpus::get_physical());

        self.trans_table.clear();

        // self.trans_table_w.purge();
        // self.trans_table_w.refresh();

        let mut timer = self.timer.clone();
        timer.reset();

        let mut depth = 1;

        let results: Arc<RwLock<(Depth,Vec<(Move,Vec<Move>,Score)>)>> =
            Arc::new(RwLock::new((depth,vec![])));

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
        let depths = vec![
            0,
            1,
            2,
        ];

        if print { eprintln!("threadcount = {:?}", threadcount); }

        let stats2 = stats.clone();


        use futures::prelude::*;
        use futures::executor::ThreadPool;

        let pool = ThreadPool::new().expect("Failed to build pool");

        let fut_values = async {

            let (tx, rx) = futures::channel::mpsc::unbounded::<usize>();

            let results2 = results.clone();
            let stats2 = stats.clone();

            let fut_result = async move {
                (0..np_cpus).for_each(|k| {

                    tx.unbounded_send(k).expect("Failed to send");

                    // let (d, out) = self._lazy_smp_single(&ts, gs.clone(), depth, stats2.clone());
                    // let d = results2.read().0;
                    // if depth > d {
                    //     // eprintln!("_lazy_smp_single: 0, depth = {:?}", depth);
                    //     drop(d);
                    //     let mut r = results2.write();
                    //     r.0 = depth;
                    //     r.1 = out;
                    // } else {
                    //     // eprintln!("_lazy_smp_single: 1, depth = {:?}", depth);
                    //     drop(d);
                    // }

                })
            };

            pool.spawn_ok(fut_result);

            let fut_values = rx
                .map(|v| v * 2)
                .collect();

            fut_values.await
        };

        let values: Vec<usize> = futures::executor::block_on(fut_values);

        eprintln!("values = {:?}", values);

        // use crossbeam::channel::{Sender,Receiver};
        // use crossbeam::thread::ScopedJoinHandle;
        // crossbeam::scope(|s| {

        //     // let (tx,rx): (Sender<ScopedJoinHandle<(Depth,Vec<(Move,Vec<Move>,Score)>)>>,
        //     //               Receiver<ScopedJoinHandle<(Depth,Vec<(Move,Vec<Move>,Score)>)>>) =
        //     //     crossbeam::channel::bounded(np_cpus * 2);

        //     // s.spawn(move |s| {
        //     //     let mut handles = vec![];
        //     //     loop {
        //     //         match rx.try_recv() {
        //     //             Ok(h)  => {
        //     //                 handles.push(h);
        //     //             },
        //     //             Err(_) => unimplemented!(),
        //     //         }
        //     //     }
        //     // });

        //     let mut thread_counter = 0;
        //     let mut search_id: Depth = 0;

        //     loop {
        //         // if timer.should_search(self.side, depth)
        //         if !timer.should_search2(self.side, depth) {
        //             drop(tx);
        //             break;
        //         }

        //         if thread_counter < np_cpus {
        //             let cur_depth = depth + 1 + search_id.trailing_zeros() as Depth;

        //             let handle = s.spawn(|s| {
        //                 unimplemented!()
        //             });
        //         }

        //         break;
        //     }

        //     // {
        //     //     let mut s = stats2.write();
        //     //     s.max_depth = depth;
        //     // }
        //     // depth += 1;
        //     // let r = stats2.read();
        //     // timer.update_times(self.side, r.nodes);

        //     // while (depth <= self.max_depth) && (strict_depth ||
        //     //                                     (|| timer.should_search(self.side, depth - 1))()) {
        //     // // (0..k_max).into_iter().for_each(|k| {
        //     // //     let r3 = r2.clone();
        //     // //     let stats3 = stats2.clone();
        //     // //     let mut gs3 = gs2.clone();
        //     // //     let x = gs3.len();
        //     // //     gs3.rotate_right((x / k_max) * k);
        //     // //     s.spawn(move |_| {
        //     // //         self._lazy_smp_single(&ts, gs3, d, r3, stats3);
        //     // //     });
        //     // // });
        //     // }

        // }).unwrap();

        // while !true && (depth <= self.max_depth) && (strict_depth ||
        //                                     (|| timer.should_search(self.side, depth - 1))()) {

        //     let r2  = results.clone();
        //     let gs2 = gs.clone();
        //     let threadcount = threadcount.clone();
        //     let depths = depths.clone();
        //     let stats2 = stats.clone();

        //     // let k_max = np_cpus;
        //     // let k = gs2.len();
        //     // let gs3 = gs2.into_par_iter().chunks(k / k_max);
        //     // gs3.for_each(|gs4| {
        //     //     self._lazy_smp_single(
        //     //         &ts,
        //     //         gs4,
        //     //         depth,
        //     //         r2.clone(),
        //     //         stats2.clone(),
        //     //     );
        //     // });

        //     // self._lazy_smp_single(
        //     //     &ts,
        //     //     gs2.clone(),
        //     //     depth,
        //     //     r2.clone(),
        //     //     stats2.clone(),
        //     // );

        //     crossbeam::scope(move |s| {

        //         // let k_max = np_cpus;
        //         // let k_max = threadcount[0];
        //         let k_max = 6;
        //         let d = depth;
        //         (0..k_max).into_iter().for_each(|k| {
        //             let r3 = r2.clone();
        //             let stats3 = stats2.clone();
        //             let mut gs3 = gs2.clone();
        //             let x = gs3.len();
        //             gs3.rotate_right((x / k_max) * k);
        //             s.spawn(move |_| {
        //                 self._lazy_smp_single(&ts, gs3, d, r3, stats3);
        //             });
        //         });

        //         // // let k_max = threadcount[1];
        //         // let k_max = 2;
        //         // let d = depth + 1;
        //         // (0..k_max).into_iter().for_each(|k| {
        //         //     let r3 = r2.clone();
        //         //     let stats3 = stats2.clone();
        //         //     let mut gs3 = gs2.clone();
        //         //     let x = gs3.len();
        //         //     gs3.rotate_right((x / k_max) * k);
        //         //     s.spawn(move |_| {
        //         //         self._lazy_smp_single(&ts, gs3, d, r3, stats3);
        //         //     });
        //         // });

        //     }).unwrap();

        //     {
        //         let mut s = stats.write();
        //         s.max_depth = depth;
        //     }

        //     if print
        //     {
        //         let r = stats.read();
        //         eprintln!("depth = {:?}", depth);
        //         eprintln!("nodes = {:?}", r.nodes);
        //     }

        //     depth += 1;
        //     let r = stats.read();
        //     timer.update_times(self.side, r.nodes);
        // }

        let d = results.read();
        let mut out = d.1.clone();
        let mut stats = stats.read().clone();
        stats.max_depth = d.0;

        #[cfg(feature = "par")]
        out.par_sort_unstable_by(|a,b| a.2.cmp(&b.2));
        #[cfg(not(feature = "par"))]
        out.sort_unstable_by(|a,b| a.2.cmp(&b.2));

        if self.side == self.game.state.side_to_move {
            out.reverse();
        }

        (out,stats)
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
                mut stats:      &mut SearchStats,
    ) -> Option<(SICanUse,SearchInfo)> {
        // if let Some(si) = self.trans_table.tt_get(&g.zobrist) {

        // if g.zobrist == Zobrist(0x1eebfbac03c62e9d) { println!("wat wat 0 {}", depth); }

        if let Some(si) = self.trans_table.get(&g.zobrist) {
        // if let Some(si) = self.trans_table_r.get(&g.zobrist) {

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
        depth:              Depth,
        k:                  i16,
        mut alpha:          i32,
        mut beta:           i32,
        maximizing:         bool,
        mut stats:          &mut SearchStats,
        mv0:                Move,
    ) -> (Vec<Move>, Score) {

        let moves = g.search_all(&ts, None);

        let moves: Vec<Move> = match moves {
            Outcome::Checkmate(c) => {
                let score = 100_000_000 - k as Score;
                stats.leaves += 1;
                stats.checkmates += 1;
                if maximizing {
                    return (vec![mv0], -score);
                } else {
                    return (vec![mv0], score);
                }

            },
            Outcome::Stalemate    => {
                let score = -100_000_000 + k as Score;
                stats.leaves += 1;
                stats.stalemates += 1;
                return (vec![],score);
            },
            Outcome::Moves(ms)    => ms,
        };

        if depth == 0 {
            let score = g.evaluate(&ts).sum();

            stats.leaves += 1;
            if self.side == Black {
                // return (vec![mv0], -score);
                return (vec![], -score);
            } else {
                // return (vec![mv0], score);
                return (vec![], score);
            }
        }

        // #[cfg(feature = "par")]
        let mut gs: Vec<(Move,Game,Option<(SICanUse,SearchInfo)>)> = {
            // let mut gs0 = moves.into_par_iter().flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, m) {
            let mut gs0 = moves.into_iter().flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, m) {
                let mut ss = SearchStats::default();
                let tt = self.check_tt(&ts, &g2, depth, maximizing, &mut ss);
                // Some(((m,g2,tt), ss))
                Some((m,g2,tt))
            } else {
                None
            });

            // let mut gs = gs.map(|x| x.0);

            // stats = stats + 
            // let ss: SearchInfo = ss.sum();

            gs0.collect()
            // gs
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

        let moves = match self.trans_table.get(&g.zobrist) {
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

        for (mv,g2,tt) in gs {
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
            stats.nodes += 1;

            let (can_use,mut mv_seq,score) = match tt {
                Some((SICanUse::UseScore,si)) => {
                    // return (si.moves.clone(),si.score);
                    (true,si.moves.clone(),si.score)
                },
                _ => {
                    let (mv_seq,score) = self._ab_search(
                        &ts, &g2, depth - 1, k + 1, alpha, beta, !maximizing, &mut stats, mv);
                    (false,mv_seq,score)
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
                (mv,&g2),
                (can_use,mv_seq,score),
                &mut val,
                depth,
                &mut alpha,
                &mut beta,
                maximizing,
                mv0);
            if b {
                node_type = Node::Cut;
                break;
            }

        }

        // XXX: depth or depth - 1 ?
        if let Some((zb,mv,mv_seq)) = &val.0 {
            // if *zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 0 {:?}, {:?}", mv, mv_seq); }
            let mut mv_seq = mv_seq.clone();
            // mv_seq.push(*mv);
            // mv_seq.push(mv0);
            self.tt_insert_deepest(
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
            Some((zb,mv,mv_seq)) => (mv_seq.clone(),val.1),
            // _                    => (vec![mv0], val.1),
            _                    => panic!("_ab_search val is None ?"),
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
            if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 3"); }
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

/// AB search 2
impl Explorer {

    #[allow(unused_doc_comments)]
    /// alpha: the MIN score that the maximizing player is assured of
    /// beta:  the MAX score that the minimizing player is assured of
    pub fn _ab_search2(
        &self,
        ts:                 &Tables,
        g_prev:             &Game,
        depth:              Depth,
        k:                  i16,
        mut alpha:          i32,
        mut beta:           i32,
        maximizing:         bool,
        mut stats:          &mut SearchStats,
        mv0:                Move,
    ) -> (Vec<Move>, Score) {

        /// Make move, return game, if terminal node return score
        let g = match g_prev.make_move_unchecked(&ts, mv0) {
            Ok(g)                          => g,
            Err(GameEnd::Checkmate { .. }) => {
                let score = 100_000_000 - k as Score;
                stats.leaves += 1;
                stats.checkmates += 1;

                // return (vec![mv0], score);
                if maximizing {
                    return (vec![mv0], -score);
                } else {
                    return (vec![mv0], score);
                }
            },
            Err(GameEnd::Stalemate | GameEnd::Draw) => {
                let score = -100_000_000 + k as Score;
                stats.leaves += 1;
                stats.stalemates += 1;
                return (vec![],score);
            },
            Err(win)                => panic!("other win? {:?}: {:?}\n{:?}", mv0, win, g_prev),
        };

        /// lookup game in trans table
        let node_type = match self.check_tt(&ts, &g, depth, maximizing, &mut stats) {
            Some((SICanUse::UseScore,si))    => {
                /// use saved score if appropriate
                return (si.moves.clone(), si.score);
            },
            Some((SICanUse::UseOrdering,si)) => Some(si.node_type),
            None                             => None,
        };

        /// Search for all legal moves, return score if none found
        let moves = match g.search_all(&ts, None) {
            Outcome::Checkmate(_) => {
                let score = 100_000_000 - k as Score;
                stats.leaves += 1;
                stats.checkmates += 1;

                // return (vec![mv0], score);
                if maximizing {
                    return (vec![mv0], -score);
                } else {
                    return (vec![mv0], score);
                }
            },
            Outcome::Stalemate => {
                let score = -100_000_000 + k as Score;
                stats.leaves += 1;
                stats.stalemates += 1;
                return (vec![],score);
            },
            Outcome::Moves(ms) => ms,
        };

        /// Evaluate node at max depth
        if depth == 0 {
            let score = g.evaluate(&ts).sum();
            stats.leaves += 1;
            // return (vec![mv0], score);
            if self.side == Black {
                return (vec![mv0], -score);
            } else {
                return (vec![mv0], score);
            }
        }

        let mut bestscore: (Option<Vec<Move>>, Score) = (None,Score::MIN);

        for mv in moves {
            let (mut mv_seq,score) = self._ab_search2(
                &ts, &g, depth - 1, k + 1, alpha, beta, !maximizing, &mut stats, mv);

            if maximizing {
                if score > bestscore.1 {
                    bestscore.1 = score;
                    mv_seq.push(mv);
                    bestscore.0 = Some(mv_seq);
                }
                alpha = i32::max(alpha, bestscore.1);
                if bestscore.1 >= beta {
                    break;
                }
            } else {

                if score < bestscore.1 {
                    bestscore.1 = score;
                    mv_seq.push(mv);
                    bestscore.0 = Some(mv_seq);
                }
                beta = i32::min(beta, bestscore.1);
                if bestscore.1 <= alpha {
                    break;
                }

            }

        }

        match bestscore.0 {
            Some(mv_seq) => (mv_seq, bestscore.1),
            None         => (vec![mv0], bestscore.1),
        }

        // unimplemented!()
    }

}

/// Static Exchange
impl Explorer {

    pub fn static_exchange(&self, ts: &Tables, g: &Game, c0: Coord) -> Option<Score> {
        let mut val = 0;

        let attackers_own   = g.find_attackers_to(&ts, c0, !g.state.side_to_move);
        if attackers_own.is_empty() { return None; }

        let attackers_other = g.find_attackers_to(&ts, c0, g.state.side_to_move);

        // let attackers = attackers_own | attackers_other;

        // let mut attackers_own = attackers_own.into_iter()
        //     .flat_map(|sq| {
        //         let c1: Coord = sq.into();
        //         if let Some((col,pc)) = g.get_at(c1) {
        //             Some((c1,pc))
        //         } else { None }
        //     }).collect::<Vec<_>>();
        // attackers_own.sort_by(|a,b| a.1.score().cmp(&b.1.score()));

        // for (c1,pc) in attackers_own.iter() {
        //     eprintln!("(c1,pc) = {:?}", (c1,pc));
        // }


        unimplemented!()
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

