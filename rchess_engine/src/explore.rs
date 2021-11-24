
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;
use crate::alphabeta::*;
use crate::opening_book::*;

#[cfg(feature = "syzygy")]
use crate::syzygy::SyzygyTB;

pub use crate::move_ordering::*;
pub use crate::timer::*;
pub use crate::trans_table::*;
pub use crate::searchstats::*;

use std::path::Path;
use std::collections::{VecDeque,HashMap,HashSet};
use std::hash::BuildHasher;
use std::sync::atomic::{Ordering,Ordering::SeqCst,AtomicU8};
use std::time::{Instant,Duration};

use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use crossbeam::thread::ScopedJoinHandle;

use itertools::Itertools;
use parking_lot::{Mutex,RwLock};

use rand::prelude::{SliceRandom,thread_rng};
use rayon::prelude::*;

use evmap::{ReadHandle,ReadHandleFactory,WriteHandle};

// #[derive(Debug)]
#[derive(Debug,Clone)]
pub struct Explorer {
    pub side:          Color,
    pub game:          Game,
    pub max_depth:     Depth,
    pub timer:         Timer,
    pub stop:          Arc<AtomicBool>,
    pub best_mate:     Arc<RwLock<Option<Depth>>>,

    pub num_threads:   Option<u8>,

    #[cfg(feature = "syzygy")]
    pub syzygy:        Option<Arc<SyzygyTB>>,
    pub opening_book:  Option<Arc<OpeningBook>>,

    pub blocked_moves: HashSet<Move>,

    pub tt_rf:         TTReadFactory,
    pub tt_w:          TTWrite,

    // pub move_history:  Vec<(Zobrist, Move)>,
    // pub pos_history:   HashMap<Zobrist,u8>,
}

#[derive(Debug,Clone)]
pub struct ExHelper {

    pub id:              usize,

    pub side:            Color,
    pub game:            Game,

    pub max_depth:       Depth,
    pub stop:            Arc<AtomicBool>,
    pub best_mate:       Arc<RwLock<Option<Depth>>>,

    #[cfg(feature = "syzygy")]
    pub syzygy:          Option<Arc<SyzygyTB>>,

    pub blocked_moves:   HashSet<Move>,

    pub best_depth:      Arc<AtomicU8>,
    pub tx:              ExSender,
    // pub thread_dec:      Sender<usize>,

    pub tt_r:            TTRead,
    pub tt_w:            TTWrite,
}

#[derive(Debug,Clone)]
pub enum ExMessage {
    Message(Depth,ABResults,Vec<Move>,SearchStats),
    End(usize),
}

// pub type ExReceiver = Receiver<(Depth,ABResults,SearchStats,Option<usize>)>;
// pub type ExSender   = Sender<(Depth,ABResults,SearchStats,Option<usize>)>;
pub type ExReceiver = Receiver<ExMessage>;
pub type ExSender   = Sender<ExMessage>;

#[derive(Debug,Default,Clone,Copy)]
pub struct ABConfig {
    pub max_depth:        Depth,
    // pub depth:            Depth,
    // pub ply:              Depth,
    // pub tt_r:             &'a TTRead,
    pub root:             bool,
    pub do_null:          bool,
    pub inside_null:      bool,
    pub use_ob:           bool,
}

impl ABConfig {
    pub fn new_depth(max_depth: Depth) -> Self {
        Self {
            max_depth,
            root:         false,
            do_null:      true,
            inside_null:  false,
            use_ob:       false,
        }
    }
}

/// New, misc
impl Explorer {
    pub fn new(side:          Color,
               game:          Game,
               max_depth:     Depth,
               // should_stop:   Arc<AtomicBool>,
               settings:      TimeSettings,
    ) -> Self {

        let stop = Arc::new(AtomicBool::new(false));

        let (tt_r, tt_w) = evmap::Options::default()
            .with_hasher(FxBuildHasher::default())
            .construct();
        let tt_rf = tt_w.factory();
        let tt_w = Arc::new(Mutex::new(tt_w));

        Self {
            side,
            game,
            max_depth,
            timer:          Timer::new(side, stop.clone(), settings),
            stop,
            best_mate:      Arc::new(RwLock::new(None)),

            num_threads:    None,

            // move_history:   VecDeque::default(),
            // syzygy:         Arc::new(None),
            #[cfg(feature = "syzygy")]
            syzygy:         None,
            opening_book:   None,

            blocked_moves:  HashSet::default(),

            tt_rf,
            tt_w,

            // move_history:   vec![],
            // pos_history:    HashMap::default(),
        }
    }

    // pub fn add_move_to_history(&mut self, zb: Zobrist, mv: Move) {
    //     self.move_history.push((zb,mv));
    // }

    pub fn clear_tt(&self) {
        let mut w = self.tt_w.lock();
        w.purge();
        w.refresh();
    }

    pub fn update_game(&mut self, g: Game) {
        self.side = g.state.side_to_move;
        self.game = g;
    }

    pub fn load_syzygy<P: AsRef<Path>>(&mut self, dir: P) -> std::io::Result<()> {
        #[cfg(feature = "syzygy")]
        {
            let mut tb = SyzygyTB::new();
            tb.add_directory(&dir)?;
            self.syzygy = Some(Arc::new(tb));
        }
        Ok(())
    }

    pub fn load_opening_book<P: AsRef<Path>>(&mut self, ts: &Tables, path: P) -> std::io::Result<()> {
        let b = OpeningBook::read_from_file(ts, &path)?;
        self.opening_book = Some(Arc::new(b));
        Ok(())
    }

}

/// Get PV
impl Explorer {

    pub fn get_pv(&self, ts: &Tables, g: &Game) -> Vec<Move> {
        let tt_r = self.tt_rf.handle();
        Self::_get_pv(ts, g, &tt_r)
    }

    pub fn _get_pv(ts: &Tables, g: &Game, tt_r: &TTRead) -> Vec<Move> {
        let mut moves = vec![];

        let mut g2 = g.clone();
        let mut zb = g2.zobrist;

        while let Some(si) = tt_r.get_one(&zb) {

            // eprintln!("si.node_type {:>3} = {:?}", k, si.node_type);
            // eprintln!("si.best_move {:>3} = {:?}", k, si.best_move);
            // eprintln!();

            let mv = si.best_move;
            moves.push(mv);

            g2 = g2.make_move_unchecked(&ts, mv).unwrap();
            zb = g2.zobrist;
        }

        moves
    }

}

/// Entry points
impl Explorer {

    pub fn explore_mult(&self, ts: &Tables)
                        -> ((ABResult,Vec<ABResult>),SearchStats) {

        let ((best, mut scores),stats,(tt_r,tt_w)) = self.lazy_smp_negamax(ts, false, false);

        scores.sort_by_key(|x| x.score);
        scores.reverse();

        ((best,scores), stats)
    }

    pub fn explore(&self, ts: &Tables, _: Option<Depth>)
                   // -> (Option<(Move,Score)>,SearchStats) {
                   -> (Option<(Move,ABResult)>,SearchStats) {

        // let (moves,stats) = self.iterative_deepening(&ts, false, false);
        // (moves.get(0).map(|x| x.0),stats)

        // let (moves,stats,_) = self.lazy_smp(&ts, false, false);

        // let mv = moves.get(0).map(|x| x.0);
        // let mv = moves.get(0).map(|x| (x.0,x.2));

        // if let Some(ob) = &self.opening_book {
        //     if let Some(mvs) = ob.best_moves(&self.game) {
        //         unimplemented!()
        //     }
        // }

        let ((best, scores),stats,(tt_r,tt_w)) = self.lazy_smp_negamax(ts, false, false);

        debug!("explore: best move = {:?}", best.mv);
        (Some((best.mv,best)),stats)

        // // if let Some(mv) = best.moves.get(0) {
        // if let Some(mv) = best.mv {
        //     let score = best.score;
        //     debug!("explore: best move = {:?}", mv);
        //     // (Some((mv,score)),stats)
        //     (Some((mv,best)),stats)
        // } else {
        //     (None, stats)
        // }

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

/// Lazy SMP Negamax 2
impl Explorer {

    pub fn reset_stop(&self) {
        self.stop.store(false, SeqCst);
        {
            let mut w = self.best_mate.write();
            *w = None;
        }
    }

    #[allow(unused_labels,unused_doc_comments)]
    pub fn lazy_smp_2(
        &self,
        ts:         &Tables,
    ) -> (ABResults,Vec<Move>,SearchStats) {

        // let mut threads = vec![];

        let max_threads = if let Some(x) = self.num_threads {
            x
        } else {
            #[cfg(feature = "one_thread")]
            let max_threads = 1;
            #[cfg(not(feature = "one_thread"))]
            let max_threads = 6;
            max_threads
        };

        self.reset_stop();

        let (tx,rx): (ExSender,ExReceiver) = crossbeam::channel::unbounded();
        // let (dec_tx,dec_rx): (Sender<usize>, Receiver<usize>) = crossbeam::channel::unbounded();

        let out: Arc<RwLock<(Depth,ABResults,Vec<Move>, SearchStats)>> =
            Arc::new(RwLock::new((0, ABResults::ABNone, vec![], SearchStats::default())));

        let thread_counter = Arc::new(AtomicU8::new(0));
        let best_depth     = Arc::new(AtomicU8::new(0));

        let t0 = Instant::now();
        // std::thread::sleep(Duration::from_micros(100));
        let t_max = self.timer.settings.increment[self.side];
        let t_max = Duration::from_secs_f64(t_max);

        crossbeam::scope(|s| {

            s.spawn(|_| {
                self.lazy_smp_listener(
                    rx,
                    // dec_rx,
                    best_depth.clone(),
                    thread_counter.clone(),
                    t0,
                    out.clone(),
                );
            });

            let mut thread_id = 0;

            'outer: loop {

                // let t1 = t0.elapsed();
                let t1 = Instant::now().checked_duration_since(t0).unwrap();

                if self.best_mate.read().is_some() {
                    let d = best_depth.load(SeqCst);
                    debug!("breaking loop (Mate),  d: {}, t0: {:.3}",
                            d, t1.as_secs_f64());
                    self.stop.store(true, SeqCst);
                    break 'outer;
                }

                if t1 > t_max {
                    let d = best_depth.load(SeqCst);
                    debug!("breaking loop (Time),  d: {}, t0: {:.3}", d, t1.as_secs_f64());
                    // XXX: Only force threads to stop if out of time ?
                    self.stop.store(true, SeqCst);
                    drop(tx);
                    break 'outer;
                }

                if best_depth.load(SeqCst) >= self.max_depth {
                    break 'outer;
                }

                if thread_counter.load(SeqCst) < max_threads {

                    trace!("Spawning thread, id = {}", thread_id);

                    let helper = self.build_exhelper(
                        thread_id, self.max_depth, best_depth.clone(), tx.clone());

                    s.builder()
                        // .stack_size(size)
                        .spawn(move |_| {
                            helper.lazy_smp_single(ts);
                        }).unwrap();

                    thread_id += 1;
                    thread_counter.fetch_add(1, SeqCst);
                    trace!("Spawned thread, count = {}", thread_counter.load(SeqCst));
                    std::thread::sleep(Duration::from_millis(1));
                }

                if self.stop.load(SeqCst) {
                    break 'outer;
                }

            }

        }).unwrap();

        let (d,mut out,moves,mut stats) = {
            let r = out.read();
            r.clone()
        };
        stats.max_depth = d;

        (out,moves,stats)

    }

    fn lazy_smp_listener(
        &self,
        rx:               ExReceiver,
        best_depth:       Arc<AtomicU8>,
        thread_counter:   Arc<AtomicU8>,
        t0:               Instant,
        out:              Arc<RwLock<(Depth,ABResults,Vec<Move>,SearchStats)>>,
    ) {
        loop {
            match rx.try_recv() {
                Ok(ExMessage::End(id)) => {
                    thread_counter.fetch_sub(1, SeqCst);
                    trace!("decrementing thread counter id = {}, new val = {}",
                            id, thread_counter.load(SeqCst));
                    break;
                },
                Ok(ExMessage::Message(depth,res,moves,stats)) => {
                    match res.clone() {
                        ABResults::ABList(bestres, _)
                            | ABResults::ABSingle(bestres)
                            | ABResults::ABSyzygy(bestres) => {
                            if depth > best_depth.load(SeqCst) {
                                best_depth.store(depth,SeqCst);

                                // let t1 = t0.elapsed();
                                let t1 = Instant::now().checked_duration_since(t0).unwrap();
                                debug!("new best move d({}): {:.3}s: {}: {:?}",
                                       depth, t1.as_secs_f64(),
                                       // bestres.score, bestres.moves.front());
                                       bestres.score, bestres.mv);

                                if bestres.score > 100_000_000 - 50 {
                                    let k = 100_000_000 - bestres.score.abs();
                                    debug!("Found mate in {}: d({}), {:?}",
                                           // bestres.score, bestres.moves.front());
                                           k, depth, bestres.mv);
                                    let mut best = self.best_mate.write();
                                    *best = Some(k as u8);

                                    self.stop.store(true, SeqCst);

                                    let mut w = out.write();
                                    *w = (depth, res, moves, w.3 + stats);
                                    // *w = (depth, scores, None);
                                    break;
                                } else {
                                    let mut w = out.write();
                                    *w = (depth, res, moves, w.3 + stats);
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
                Err(TryRecvError::Empty)    => {
                    // std::thread::sleep(Duration::from_millis(1));
                },
                Err(TryRecvError::Disconnected)    => {
                    trace!("Breaking thread counter loop (Disconnect)");
                    break;
                },
            }
        }
    }

    pub fn build_exhelper(
        &self,
        id:               usize,
        max_depth:        Depth,
        best_depth:       Arc<AtomicU8>,
        tx:               ExSender,
    ) -> ExHelper {
        ExHelper {
            id,

            side:            self.side,
            game:            self.game.clone(),

            max_depth,
            stop:            self.stop.clone(),
            best_mate:       self.best_mate.clone(),

            #[cfg(feature = "syzygy")]
            syzygy:          self.syzygy.clone(),

            blocked_moves:   self.blocked_moves.clone(),

            best_depth,
            tx,
            // thread_dec,

            tt_r:            self.tt_rf.handle(),
            tt_w:            self.tt_w.clone(),
        }
    }

}

/// Lazy SMP Negamax 2
impl ExHelper {

    const SKIP_LEN: usize = 20;
    const SKIP_SIZE: [Depth; Self::SKIP_LEN] = [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
    const START_PLY: [Depth; Self::SKIP_LEN] = [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];

    #[allow(unused_doc_comments)]
    fn lazy_smp_single(
        &self,
        ts:               &Tables,
    ) {

        let mut history = [[[0; 64]; 64]; 2];
        let mut stats = SearchStats::default();

        let skip_size = Self::SKIP_SIZE[self.id % Self::SKIP_LEN];
        let start_ply = Self::START_PLY[self.id % Self::SKIP_LEN];
        let mut depth = start_ply + 1;

        trace!("self.max_depth = {:?}", self.max_depth);
        trace!("iterative skip_size {}", skip_size);
        trace!("iterative start_ply {}", start_ply);

        /// Iterative deepening
        while !self.stop.load(SeqCst)
            && depth <= self.max_depth
            && self.best_mate.read().is_none()
        {
            // trace!("iterative depth {}", depth);

            let res = self.ab_search_single(ts, &mut stats, &mut history, depth);
            // debug!("res = {:?}", res);

            let moves = Explorer::_get_pv(ts, &self.game, &self.tt_r);

            if !self.stop.load(SeqCst) && depth >= self.best_depth.load(SeqCst) {
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

        match self.tx.try_send(ExMessage::End(self.id)) {
            Ok(_)  => {},
            Err(_) => {
                trace!("tx send error 1: id: {}, depth {}", self.id, depth);
            },
        }

    }

}

/// Lazy SMP Negamax Old
impl Explorer {

    fn _lazy_smp_single_negamax(
        &self,
        ts:               &Tables,
        depth:            Depth,
        tx:               Sender<(Depth,ABResults,SearchStats)>,
        // mut history:      [[[Score; 64]; 64]; 2],
    ) {
        let mut history = [[[0; 64]; 64]; 2];

        let tt_r = self.tt_rf.handle();
        let tt_w = self.tt_w.clone();

        let (alpha,beta) = (i32::MIN,i32::MAX);
        let (alpha,beta) = (alpha + 200,beta - 200);

        let mut stats = SearchStats::default();
        let mut stop_counter = 0;

        let mut cfg = ABConfig::new_depth(depth);
        cfg.root = true;

        let mut g       = self.game.clone();
        // let mut prev_ms = VecDeque::new();

        // let res = self._ab_search_negamax(
        //     ts, &mut g, cfg, depth,
        //     0, &mut stop_counter, (alpha, beta),
        //     &mut stats,
        //     // &mut prev_ms,
        //     VecDeque::new(),
        //     &mut history,
        //     // &tt_r, tt_w.clone(),true,true);
        //     &tt_r, tt_w.clone());

        // // XXX: ??
        // match tx.send((depth,res,stats)) {
        // // match tx.try_send((depth,res,stats)) {
        //     Ok(_) => {},
        //     // Err(_) => panic!("tx send error: depth {}", depth),
        //     Err(_) => trace!("tx send error: depth {}", depth),
        //     // Err(_) => {},
        // }
        // drop(tx);

        unimplemented!()
    }

    fn _lazy_smp_single_negamax2(
        &self,
        ts:               &Tables,
        depth:            Depth,
        tx:               Sender<(Depth,ABResults,SearchStats)>,
        // tt_r:             TTRead,
        // tt_w:             TTWrite,
    ) {

        let mut stats = SearchStats::default();

        // let helper = self.build_exhelper(ts, 1, depth);
        // let res = helper.ab_search_negamax(ts, &mut stats, depth);

        // match tx.send((depth,res,stats)) {
        //     // match tx.try_send((depth,res,stats)) {
        //     Ok(_) => {},
        //     // Err(_) => panic!("tx send error: depth {}", depth),
        //     Err(_) => trace!("tx send error: depth {}", depth),
        //     // Err(_) => {},
        // }
        // drop(tx);

        unimplemented!()
    }

    #[allow(unused_doc_comments)]
    pub fn lazy_smp_negamax(&self, ts: &Tables, print: bool, strict_depth: bool)
                            -> ((ABResult, Vec<ABResult>),SearchStats,(TTRead,TTWrite)) {

        let out: Arc<RwLock<(Depth,ABResults,SearchStats)>> =
            Arc::new(RwLock::new((0, ABResults::ABNone, SearchStats::default())));

        let tt_w = self.tt_w.clone();
        let tt_r = self.tt_rf.handle();

        let sleep_time = Duration::from_millis(1);

        self.stop.store(false, SeqCst);
        {
            let mut w = self.best_mate.write();
            *w = None;
        }

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
                                ABResults::ABList(bestres, _) | ABResults::ABSingle(bestres)
                                    if depth > best_depth2.load(SeqCst) => {
                                        best_depth2.store(depth,SeqCst);

                                        debug!("new best move d({}): {:.3}s: {}: {:?}",
                                            depth, t0.elapsed().as_secs_f64(),
                                            // bestres.score, bestres.moves.front());
                                            bestres.score, bestres.mv);

                                        if bestres.score > 100_000_000 - 50 {
                                            let k = 100_000_000 - bestres.score.abs();
                                            debug!("Found mate in {}: d({}), {:?}",
                                                // k, depth, bestres.moves.front());
                                                k, depth, bestres.mv);
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

                                // ABResults::ABSingle(bestres) if depth > best_depth2.load(SeqCst) => {
                                //     best_depth2.store(depth,SeqCst);

                                //     debug!("new best move d({}): {:.3}s: {}: {:?}",
                                //         depth, t0.elapsed().as_secs_f64(),
                                //         bestres.score, bestres.moves.front());

                                //     if bestres.score > 100_000_000 - 50 {
                                //         let k = 100_000_000 - bestres.score.abs();
                                //         debug!("Found mate in {}: d({}), {:?}",
                                //                k, depth, bestres.moves.front());
                                //         let mut best = self.best_mate.write();
                                //         *best = Some(k as u8);
                                //         let mut w = out1.write();
                                //         *w = (depth, abres, w.2 + stats0);
                                //         // *w = (depth, scores, None);
                                //         break;
                                //     } else {
                                //         let mut w = out1.write();
                                //         *w = (depth, abres, w.2 + stats0);
                                //     }

                                // },

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
                    let tt_w2  = tt_w.clone();
                    let tt_r2  = self.tt_rf.handle();

                    trace!("spawning thread: (sid: {}) = cur_depth {:?}", sid, cur_depth);
                    s.builder()
                        // .stack_size(4 * 1024 * 1024) // 4 MiB
                        // .stack_size(1 * 512 * 1024) // 0.5 MiB
                        // .stack_size(64 * 1024) // 64 KiB
                        .spawn(move |_| {
                            self._lazy_smp_single_negamax(
                                // self._lazy_smp_single_aspiration(
                                // ts, cur_depth, tx2, tt_r2, tt_w2.clone());
                                ts, cur_depth, tx2);
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

                // debug!("finished lazy_smp_negamax: moves {:?}", best.moves);
                debug!("finished lazy_smp_negamax: move {:?}", best.mv);

                ((best,ress),stats,(tt_r,tt_w))
            },
            ABResults::ABSingle(best) => {
                // panic!("single result only?");
                debug!("finished lazy_smp_negamax: single move {:?}", best.mv);
                ((best,vec![]),stats,(tt_r,tt_w))
            }
            r => {
                // ((ABResults::ABNone, vec![]), stats, (tt_r,tt_w))
                // unimplemented!()
                panic!("ABResults: {:?}", r);
            },
        }
    }

}

/// Misc
impl Explorer {
}

