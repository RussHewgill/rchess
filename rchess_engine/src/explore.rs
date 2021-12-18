
use crate::evmap_tables::*;
use crate::lockless_map::*;
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;
use crate::alphabeta::*;
use crate::opening_book::*;
use crate::pawn_hash_table::*;
use crate::killer_moves::*;

#[cfg(feature = "syzygy")]
use crate::syzygy::SyzygyTB;
use crate::sf_compat::NNUE4;

pub use crate::move_ordering::*;
pub use crate::timer::*;
pub use crate::trans_table::*;
pub use crate::searchstats::*;

use std::cell::RefCell;
use std::path::Path;
use std::collections::{VecDeque,HashMap,HashSet};
use std::hash::BuildHasher;
use std::sync::atomic::{Ordering,Ordering::SeqCst,AtomicU8};
use std::time::{Instant,Duration};

use arrayvec::ArrayVec;
use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use crossbeam::thread::ScopedJoinHandle;

use itertools::Itertools;
use parking_lot::{Mutex,RwLock};

use rand::prelude::{SliceRandom,thread_rng};
use rayon::prelude::*;

use evmap::{ReadHandle,ReadHandleFactory,WriteHandle};

#[derive(Debug)]
// #[derive(Debug,Clone)]
pub struct Explorer {
    pub side:          Color,
    pub game:          Game,
    pub current_ply:   Option<Depth>,
    pub timer:         Timer,
    pub stop:          Arc<AtomicBool>,
    pub best_mate:     Arc<RwLock<Option<Depth>>>,

    pub cfg:           ExConfig,

    #[cfg(feature = "syzygy")]
    pub syzygy:        Option<Arc<SyzygyTB>>,
    pub opening_book:  Option<Arc<OpeningBook>>,

    pub nnue:          Option<NNUE4>,

    #[cfg(feature = "lockless_hashmap")]
    pub ptr_tt:        Arc<TransTable>,

    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_rf:         TTReadFactory,
    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_w:          TTWrite,

    // pub ph_rw:         (PHReadFactory,PHWrite),
    pub ph_rw:         PHTableFactory,

    // pub move_history:  Vec<(Zobrist, Move)>,
    // pub pos_history:   HashMap<Zobrist,u8>,
}

#[derive(Debug,Clone)]
pub struct ExConfig {
    pub max_depth:         Depth,
    pub num_threads:       Option<u8>,
    pub blocked_moves:     HashSet<Move>,
    pub only_moves:        Option<HashSet<Move>>,
    pub return_moves:      bool,
    pub clear_table:       bool,
    pub eval_params_mid:   EvalParams,
    pub eval_params_end:   EvalParams,
}

impl Default for ExConfig {
    fn default() -> Self {
        Self {
            max_depth:        10,
            num_threads:      None,
            blocked_moves:    HashSet::default(),
            only_moves:       None,
            return_moves:     false,
            clear_table:      true,
            eval_params_mid:  EvalParams::default(),
            eval_params_end:  EvalParams::default(),
        }
    }
}

#[derive(Debug,Clone)]
pub struct ExHelper {

    pub id:              usize,

    pub side:            Color,
    pub game:            Game,

    pub stop:            Arc<AtomicBool>,
    pub best_mate:       Arc<RwLock<Option<Depth>>>,

    #[cfg(feature = "syzygy")]
    pub syzygy:          Option<Arc<SyzygyTB>>,
    pub nnue:            Option<RefCell<NNUE4>>,

    pub cfg:             ExConfig,

    pub best_depth:      Arc<AtomicU8>,
    pub tx:              ExSender,
    // pub thread_dec:      Sender<usize>,

    #[cfg(feature = "lockless_hashmap")]
    pub ptr_tt:          Arc<TransTable>,

    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_r:            TTRead,
    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_w:            TTWrite,

    // pub ph_rw:         (PHRead,PHWrite),
    pub ph_rw:           PHTable,
}

/// Load EvalParams
impl ExHelper {
    pub fn load_evparams<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let (ev_mid,ev_end) = EvalParams::read_evparams(path)?;
        self.cfg.eval_params_mid = ev_mid;
        self.cfg.eval_params_end = ev_end;
        Ok(())
    }
}

#[derive(Debug,Clone)]
pub struct ABStack {
    pub history:      [[[Score; 64]; 64]; 2],
    pub killers:      KillerMoves,
    pub pvs:          Vec<Move>,
}

impl ABStack {
    fn new() -> Self {
        Self {
            history:      [[[0; 64]; 64]; 2],
            killers:      KillerMoves::default(),
            pvs:          Vec::with_capacity(64),
        }
    }
}

/// build_exhelper
impl Explorer {
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

            stop:            self.stop.clone(),
            best_mate:       self.best_mate.clone(),

            cfg:             self.cfg.clone(),

            #[cfg(feature = "syzygy")]
            syzygy:          self.syzygy.clone(),

            nnue:            self.nnue.clone().map(|x| RefCell::new(x)),

            best_depth,
            tx,
            // thread_dec,

            #[cfg(feature = "lockless_hashmap")]
            ptr_tt:          self.ptr_tt.clone(),

            #[cfg(not(feature = "lockless_hashmap"))]
            tt_r:            self.tt_rf.handle(),
            #[cfg(not(feature = "lockless_hashmap"))]
            tt_w:            self.tt_w.clone(),

            // ph_rw:           (self.ph_rw.0.handle(),self.ph_rw.1.clone()),
            ph_rw:           self.ph_rw.handle(),
        }
    }

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
    // pub root:             bool,
    pub do_null:          bool,
    pub inside_null:      bool,
    // pub use_ob:           bool,
}

impl ABConfig {
    pub fn new_depth(max_depth: Depth) -> Self {
        Self {
            max_depth,
            // root:         false,
            do_null:      true,
            inside_null:  false,
            // use_ob:       false,
        }
    }
}

/// New, misc
impl Explorer {
    pub fn new(
        side:          Color,
        game:          Game,
        max_depth:     Depth,
        settings:      TimeSettings,
    ) -> Self {

        let stop = Arc::new(AtomicBool::new(false));

        #[cfg(not(feature = "lockless_hashmap"))]
        let (tt_rf, tt_w) = new_hash_table();

        let ph_rw = PHTableFactory::new();

        let mut cfg = ExConfig::default();
        cfg.max_depth = max_depth;

        Self {
            side,
            game,
            // timer:          Timer::new(side, stop.clone(), settings),
            current_ply:    None,
            timer:          Timer::new(settings),
            stop,
            best_mate:      Arc::new(RwLock::new(None)),

            cfg,

            // move_history:   VecDeque::default(),
            // syzygy:         Arc::new(None),
            #[cfg(feature = "syzygy")]
            syzygy:         None,
            opening_book:   None,

            nnue:           None,

            #[cfg(feature = "lockless_hashmap")]
            ptr_tt:         Arc::new(TransTable::new_mb(DEFAULT_TT_SIZE_MB)),

            #[cfg(not(feature = "lockless_hashmap"))]
            tt_rf,
            #[cfg(not(feature = "lockless_hashmap"))]
            tt_w,

            // ph_rw:          (ph_rf,ph_w),
            ph_rw,

            // move_history:   vec![],
            // pos_history:    HashMap::default(),
        }
    }

    // pub fn add_move_to_history(&mut self, zb: Zobrist, mv: Move) {
    //     self.move_history.push((zb,mv));
    // }

    pub fn clear_tt(&self) {
        #[cfg(not(feature = "lockless_hashmap"))]
        {
            let mut w = self.tt_w.lock();
            w.purge();
            w.refresh();
        }
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

    pub fn load_nnue<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        #[cfg(feature = "nnue")]
        {
            let mut nn = NNUE4::read_nnue(path)?;
            self.nnue = Some(nn);
        }
        Ok(())
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
        #[cfg(feature = "lockless_hashmap")]
        {
            Self::_get_pv_lockless(ts, g, self.ptr_tt.clone())
        }
        #[cfg(not(feature = "lockless_hashmap"))]
        {
            let tt_r = self.tt_rf.handle();
            Self::_get_pv(ts, g, &tt_r)
        }
    }

    pub fn _get_pv_lockless(ts: &Tables, g: &Game, tt: Arc<TransTable>) -> Vec<Move> {
        let mut moves = vec![];

        let mut g2 = g.clone();
        let mut zb = g2.zobrist;

        let mut hashes = HashSet::<Zobrist>::default();
        hashes.insert(zb);

        while let Some(si) = tt.probe(zb) {
            hashes.insert(zb);

            // eprintln!("si.node_type {:>3} = {:?}", k, si.node_type);
            // eprintln!("si.best_move {:>3} = {:?}", k, si.best_move);
            // eprintln!();

            let mv = si.best_move;

            let mv = [mv.0, mv.1];
            let mv = PackedMove::unpack(&mv).unwrap().convert_to_move(ts, &g2);

            moves.push(mv);

            g2 = g2.make_move_unchecked(&ts, mv).unwrap();
            zb = g2.zobrist;

            if hashes.contains(&zb) {
                trace!("_get_pv, duplicate hash: {:?}\n{:?}", zb, g);
                break;
            }
        }

        moves
    }

    pub fn _get_pv(ts: &Tables, g: &Game, tt_r: &TTRead) -> Vec<Move> {
        let mut moves = vec![];

        let mut g2 = g.clone();
        let mut zb = g2.zobrist;

        let mut hashes = HashSet::<Zobrist>::default();
        hashes.insert(zb);

        while let Some(si) = tt_r.get_one(&zb) {
            hashes.insert(zb);

            // eprintln!("si.node_type {:>3} = {:?}", k, si.node_type);
            // eprintln!("si.best_move {:>3} = {:?}", k, si.best_move);
            // eprintln!();

            let mv = si.best_move;

            let mv = [mv.0, mv.1];
            let mv = PackedMove::unpack(&mv).unwrap().convert_to_move(ts, &g2);

            moves.push(mv);

            g2 = g2.make_move_unchecked(&ts, mv).unwrap();
            zb = g2.zobrist;

            if hashes.contains(&zb) {
                trace!("_get_pv, duplicate hash: {:?}\n{:?}", zb, g);
                break;
            }

        }

        moves
    }

}

/// Entry points
impl Explorer {

    // pub fn explore_mult(&self, ts: &Tables)
    //                     -> ((ABResult,Vec<ABResult>),SearchStats) {
    //     let ((best, mut scores),stats,(tt_r,tt_w)) = self.lazy_smp_negamax(ts, false, false);
    //     scores.sort_by_key(|x| x.score);
    //     scores.reverse();
    //     ((best,scores), stats)
    // }

    // pub fn explore(&self, ts: &Tables, _: Option<Depth>)
    pub fn explore(&self, ts: &Tables)
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

        // let ((best, scores),stats,(tt_r,tt_w)) = self.lazy_smp_negamax(ts, false, false);

        let (ress,moves,stats) = self.lazy_smp_2(ts);

        // debug!("finished lazy_smp_2, ress = {:?}", ress);

        // match ress {
        //     ABResults::ABList(res,_) | ABResults::ABSingle(res) 
        // }

        if let Some(best) = ress.get_result() {
            debug!("explore: best move = {:?}", best.mv);
            (Some((best.mv,best)),stats)
        } else {
            debug!("explore: no best move? = {:?}", ress);
            // panic!();
            (None,stats)
        }

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

        let max_threads = if let Some(x) = self.cfg.num_threads {
            x
        } else {
            #[cfg(feature = "one_thread")]
            let max_threads = 1;
            #[cfg(not(feature = "one_thread"))]
            let max_threads = 6;
            max_threads
        };

        debug!("lazy_smp_2 max_threads = {:?}", max_threads);

        if self.cfg.clear_table {
            debug!("clearing tt");
            self.clear_tt();
        }

        self.reset_stop();

        let (tx,rx): (ExSender,ExReceiver) = crossbeam::channel::unbounded();
        // let (dec_tx,dec_rx): (Sender<usize>, Receiver<usize>) = crossbeam::channel::unbounded();

        let out: Arc<RwLock<(Depth,ABResults,Vec<Move>, SearchStats)>> =
            Arc::new(RwLock::new((0, ABResults::ABNone, vec![], SearchStats::default())));

        let thread_counter = Arc::new(AtomicU8::new(0));
        let best_depth     = Arc::new(AtomicU8::new(0));

        let t0 = Instant::now();
        // std::thread::sleep(Duration::from_micros(100));

        #[cfg(feature = "basic_time")]
        let t_max = Duration::from_secs_f64(self.timer.settings.increment[self.side]);

        #[cfg(not(feature = "basic_time"))]
        let cur_ply = self.current_ply.unwrap_or(1);
        #[cfg(not(feature = "basic_time"))]
        let (t_opt,t_max) = self.timer.allocate_time(self.game.state.side_to_move, cur_ply);
        // debug!("searching with (t_opt,t_max) = ({:?},{:?})", t_opt, t_max);

        debug!("searching with t_max = {:?}", t_max);

        // let t_max = self.timer.allocate_time()

        crossbeam::scope(|s| {

            s.spawn(|_| {
                self.lazy_smp_listener(
                    rx,
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

                let d = best_depth.load(SeqCst);

                /// Found mate, halt
                if self.best_mate.read().is_some() {
                    debug!("breaking loop (Mate),  d: {}, t0: {:.3}",
                            d, t1.as_secs_f64());
                    self.stop.store(true, SeqCst);
                    break 'outer;
                }

                // TODO: 
                // /// passed optimum time, maybe halt?
                // if t1 > t_opt {
                // }

                /// Out of time, halt
                if t1 > t_max {
                    debug!("breaking loop (Time),  d: {}, t0: {:.3}", d, t1.as_secs_f64());
                    // XXX: Only force threads to stop if out of time ?
                    self.stop.store(true, SeqCst);
                    // drop(tx);
                    break 'outer;
                }

                /// Max depth reached, halt
                if d >= self.cfg.max_depth {
                    debug!("max depth reached, breaking");
                    self.stop.store(true, SeqCst);
                    drop(tx);
                    break 'outer;
                }

                if thread_counter.load(SeqCst) < max_threads {

                    trace!("Spawning thread, id = {}", thread_id);

                    let helper = self.build_exhelper(
                        thread_id, self.cfg.max_depth, best_depth.clone(), tx.clone());

                    s.builder()
                        // .stack_size(size)
                        .spawn(move |_| {
                            helper.lazy_smp_single(ts);
                        }).unwrap();

                    thread_id += 1;
                    thread_counter.fetch_add(1, SeqCst);
                    trace!("Spawned thread, count = {}", thread_counter.load(SeqCst));
                    // std::thread::sleep(Duration::from_millis(1));
                }

                if self.stop.load(SeqCst) {
                    break 'outer;
                }

                std::thread::sleep(Duration::from_millis(1));
            }
            trace!("exiting lazy_smp_2 loop");

        }).unwrap();
        trace!("exiting lazy_smp_2 scoped");

        let (d,mut out,moves,mut stats) = {
            let r = out.read();
            r.clone()
        };
        stats.max_depth = d;

        stats.ph_hits   = self.ph_rw.hits.load(Ordering::Relaxed);
        stats.ph_misses = self.ph_rw.misses.load(Ordering::Relaxed);

        if let Some(res) = out.get_result() {
            let out = if self.game.move_is_legal(ts, res.mv, self.game.state.side_to_move) {
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

    fn lazy_smp_listener(
        &self,
        rx:               ExReceiver,
        best_depth:       Arc<AtomicU8>,
        thread_counter:   Arc<AtomicU8>,
        t0:               Instant,
        out:              Arc<RwLock<(Depth,ABResults,Vec<Move>,SearchStats)>>,
    ) {
        loop {
            // match rx.try_recv() {
            match rx.recv() {
                Ok(ExMessage::End(id)) => {
                    thread_counter.fetch_sub(1, SeqCst);
                    trace!("decrementing thread counter id = {}, new val = {}",
                            id, thread_counter.load(SeqCst));
                    // break;
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

                                if bestres.score.abs() == CHECKMATE_VALUE {
                                    self.stop.store(true, SeqCst);
                                    debug!("in mate, nothing to do");
                                    break;
                                }

                                if bestres.score > CHECKMATE_VALUE - 50 {
                                    let k = CHECKMATE_VALUE - bestres.score.abs();
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

        // let mut history = [[[0; 64]; 64]; 2];
        let mut tracking = ABStack::new();
        let mut stats = SearchStats::default();

        let skip_size = Self::SKIP_SIZE[self.id % Self::SKIP_LEN];
        let start_ply = Self::START_PLY[self.id % Self::SKIP_LEN];
        let mut depth = start_ply + 1;

        // trace!("self.max_depth = {:?}", self.max_depth);
        // trace!("iterative skip_size {}", skip_size);
        // trace!("iterative start_ply {}", start_ply);

        /// Iterative deepening
        while !self.stop.load(SeqCst)
            && depth <= self.cfg.max_depth
            && self.best_mate.read().is_none()
        {
            // trace!("iterative depth {}", depth);

            let res = self.ab_search_single(ts, &mut stats, &mut tracking, depth);
            // debug!("res = {:?}", res);
            // trace!("finished res, id = {}, depth = {}", self.id, depth);

            if !self.stop.load(SeqCst) && depth >= self.best_depth.load(SeqCst) {
                let moves = if self.cfg.return_moves {
                    #[cfg(feature = "lockless_hashmap")]
                    {
                        Explorer::_get_pv_lockless(ts, &self.game, self.ptr_tt.clone())
                    }
                    #[cfg(not(feature = "lockless_hashmap"))]
                    {
                        Explorer::_get_pv(ts, &self.game, &self.tt_r)
                    }
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

        match self.tx.try_send(ExMessage::End(self.id)) {
            Ok(_)  => {},
            Err(_) => {
                trace!("tx send error 1: id: {}, depth {}", self.id, depth);
            },
        }

        trace!("exiting lazy_smp_single, id = {}", self.id);
    }

}

