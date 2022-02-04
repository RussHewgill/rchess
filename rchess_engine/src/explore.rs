
use crate::evmap_tables::*;
use crate::lockless_map::*;
use crate::material::{MaterialTable,PawnTable};
use crate::movegen::MoveGen;
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;
use crate::alphabeta::*;
use crate::opening_book::*;
// use crate::pawn_hash_table::*;
// use crate::heuristics::*;
pub use crate::stack::{ABStack,ABStackPly};

#[cfg(feature = "syzygy")]
use crate::syzygy::SyzygyTB;
use crate::sf_compat::NNUE4;

pub use crate::move_ordering::*;
pub use crate::timer::*;
pub use crate::trans_table::*;
pub use crate::searchstats::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::path::Path;
use std::collections::{VecDeque,HashMap,HashSet};
use std::hash::BuildHasher;
use std::sync::atomic::{Ordering,Ordering::SeqCst,Ordering::Relaxed,AtomicU8,AtomicI8,AtomicI16};
use std::time::{Instant,Duration};

use arrayvec::ArrayVec;
use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use crossbeam::thread::ScopedJoinHandle;
use crossbeam::utils::CachePadded;

use itertools::Itertools;
use parking_lot::{Mutex,RwLock};

use rand::prelude::{SliceRandom,thread_rng};
use rayon::prelude::*;
use lazy_static::lazy_static;

use derive_new::new;

// use evmap::{ReadHandle,ReadHandleFactory,WriteHandle};

lazy_static! { /// DEBUG_ABSTACK
    pub static ref DEBUG_ABSTACK: Mutex<ABStack> = Mutex::new(ABStack::new());
}

/// used for persistent data between runs
#[derive(Debug,Default,Clone,new)]
pub struct PerThreadData {
    pub mat_table:       MaterialTable,
    pub pawn_table:      PawnTable,
}

// #[derive(Debug)]
#[derive(Debug,Clone)]
pub struct Explorer {
    pub side:              Color,
    pub game:              Game,
    // pub current_ply:       Option<Depth>,

    #[cfg(feature = "basic_time")]
    pub timer:             Timer,
    #[cfg(not(feature = "basic_time"))]
    pub time_settings:     TimeSettings,

    // pub stop:              Arc<AtomicBool>,
    pub stop:              Arc<CachePadded<AtomicBool>>,
    // pub best_mate:         Arc<RwLock<Option<Depth>>>,
    pub best_mate:         Arc<CachePadded<AtomicI16>>,
    pub best_depth:        Arc<CachePadded<AtomicI16>>,

    pub tx:                ExSender,
    pub rx:                ExReceiver,

    pub cfg:               ExConfig,

    pub search_params:     SParams,

    #[cfg(feature = "syzygy")]
    pub syzygy:            Option<Arc<SyzygyTB>>,
    pub opening_book:      Option<Arc<OpeningBook>>,

    pub nnue:              Option<NNUE4>,

    #[cfg(feature = "lockless_hashmap")]
    pub ptr_tt:            Arc<TransTable>,

    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_rf:             TTReadFactory,
    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_w:              TTWrite,

    // pub eval_hashmap:      (EVReadFactory<Score>,EVWrite<Score>),

    // pub ph_rw:             (PHReadFactory,PHWrite),
    // pub ph_rw:             PHTableFactory,

    // pub mat_rw:            

    pub move_history:      Vec<(Zobrist, Move)>,
    // pub pos_history:       HashMap<Zobrist,u8>,


    pub per_thread_data:   Vec<Option<PerThreadData>>,

}

/// New
impl Explorer {
    pub fn new(
        side:          Color,
        game:          Game,
        max_depth:     Depth,
        time_settings: TimeSettings,
    ) -> Self {

        // let stop = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(CachePadded::new(AtomicBool::new(false)));

        #[cfg(not(feature = "lockless_hashmap"))]
        let (tt_rf, tt_w) = new_hash_table();

        // let ph_rw = PHTableFactory::new();

        let mut cfg = ExConfig::default();
        cfg.max_depth = max_depth;

        let (tx,rx): (ExSender,ExReceiver) = crossbeam::channel::unbounded();

        Self {
            side,
            game,
            // timer:          Timer::new(side, stop.clone(), settings),
            // current_ply:    None,

            #[cfg(feature = "basic_time")]
            timer:          Timer::new(time_settings),
            #[cfg(not(feature = "basic_time"))]
            time_settings,

            stop,
            // best_mate:      Arc::new(RwLock::new(None)),
            best_mate:     Arc::new(CachePadded::new(AtomicI16::new(-1))),
            best_depth:     Arc::new(CachePadded::new(AtomicI16::new(0))),

            tx,
            rx,

            cfg,
            search_params:  SParams::default(),

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
            // ph_rw,

            move_history:   vec![],
            // pos_history:    HashMap::default(),

            per_thread_data: vec![Some(PerThreadData::default())],
        }
    }

}

#[derive(Debug,Clone)]
pub struct ExConfig {
    pub max_depth:             Depth,
    pub num_threads:           Option<u8>,

    pub blocked_moves:         HashSet<Move>,
    pub only_moves:            Option<HashSet<Move>>,

    pub late_move_reductions:  bool,

    pub return_moves:          bool,

    pub clear_table:           bool,
    pub hash_size_mb:          Option<usize>,

    // pub eval_params_mid:       EvalParams,
    // pub eval_params_end:       EvalParams,
}

impl Default for ExConfig {
    fn default() -> Self {
        Self {
            max_depth:             10,
            num_threads:           None,

            blocked_moves:         HashSet::default(),
            only_moves:            None,

            late_move_reductions:  cfg!(feature = "late_move_reduction"),

            return_moves:          false,

            // clear_table:           true,
            clear_table:           false,
            hash_size_mb:          None,

            // eval_params_mid:       EvalParams::default(),
            // eval_params_end:       EvalParams::default(),
        }
    }
}

#[derive(Debug,Clone)]
pub struct ExHelper {

    pub id:              usize,

    pub side:            Color,
    pub game:            Game,

    pub root_moves:      Vec<Move>,
    // pub root_moves:      RefCell<Vec<Move>>,

    pub stop:            Arc<CachePadded<AtomicBool>>,
    pub best_mate:       Arc<CachePadded<AtomicI16>>,

    #[cfg(feature = "syzygy")]
    pub syzygy:          Option<Arc<SyzygyTB>>,
    // pub nnue:            Option<RefCell<NNUE4>>,
    pub nnue:            Option<NNUE4>,

    pub cfg:             ExConfig,
    pub params:          SParams,

    pub best_depth:      Arc<CachePadded<AtomicI16>>,
    pub tx:              ExSender,
    // pub thread_dec:      Sender<usize>,

    #[cfg(feature = "lockless_hashmap")]
    pub ptr_tt:          Arc<TransTable>,

    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_r:            TTRead,
    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_w:            TTWrite,

    // pub ph_rw:         (PHRead,PHWrite),
    // pub ph_rw:           PHTable,

    pub move_history:    Vec<(Zobrist, Move)>,
    // pub stack:           Cell<ABStack>,

    // pub prev_best_move:  Option<Move>,

    pub material_table:  MaterialTable,
    pub pawn_table:      PawnTable,

}

// /// Load EvalParams
// impl ExHelper {
//     pub fn load_evparams<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
//         let (ev_mid,ev_end) = EvalParams::read_evparams(path)?;
//         self.cfg.eval_params_mid = ev_mid;
//         self.cfg.eval_params_end = ev_end;
//         Ok(())
//     }
// }

/// build_exhelper
impl Explorer {
    pub fn build_exhelper(
        &self,
        id:               usize,
        max_depth:        Depth,
        best_depth:       Arc<CachePadded<AtomicI16>>,
        root_moves:       Vec<Move>,
        tx:               ExSender,
        thread_data:      PerThreadData,
    ) -> ExHelper {
        ExHelper {
            id,

            side:            self.side,
            game:            self.game.clone(),

            // root_moves:      RefCell::new(root_moves),
            root_moves:      root_moves,

            stop:            self.stop.clone(),
            best_mate:       self.best_mate.clone(),

            cfg:             self.cfg.clone(),
            params:          self.search_params.clone(),

            #[cfg(feature = "syzygy")]
            syzygy:          self.syzygy.clone(),

            // nnue:            self.nnue.clone().map(|x| RefCell::new(x)),
            nnue:            self.nnue.clone(),

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
            // ph_rw:           self.ph_rw.handle(),

            move_history:    self.move_history.clone(),

            // prev_best_move:  None,

            material_table:  thread_data.mat_table,
            pawn_table:      thread_data.pawn_table,

        }
    }

}

#[derive(Debug,Clone)]
pub enum ExMessage {
    Message(Depth,ABResults,Vec<Move>,Box<SearchStats>),
    End(usize),
    Stop,
}

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

/// new_game, misc
impl Explorer {
    // pub fn add_move_to_history(&mut self, zb: Zobrist, mv: Move) {
    //     self.move_history.push((zb,mv));
    // }

    pub fn new_game(&mut self) {
        self.clear_tt();

        for pt in self.per_thread_data.iter_mut() {
            if let Some(pt) = pt {
                *pt = PerThreadData::default();
            }
        }

    }

    pub fn clear_tt(&self) {
        #[cfg(feature = "lockless_hashmap")]
        {
            // debug!("clearing table, unsafe");
            // unsafe {
            //     self.ptr_tt.clear_table();
            // }
            self.ptr_tt.clear_table();
        }
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

            #[cfg(feature = "prev_accum")]
            // nnue.ft.accum.undo_stack_copies.clear();
            nnue.ft.accum.stack_copies.clear();
            #[cfg(feature = "prev_accum")]
            // nnue.ft.accum.undo_stack_delta.clear();
            nnue.ft.accum.stack_delta.clear();

            #[cfg(feature = "prev_accum")]
            nnue.ft.reset_accum(&g);
            #[cfg(not(feature = "prev_accum"))]
            nnue.ft.reset_feature_trans(&g);
        }
        self.side = g.state.side_to_move;
        self.game = g;
    }

    pub fn _update_game_movelist<'a>(
        &mut self,
        ts:          &Tables,
        moves:       &[Move],
    ) {
        let mut g = self.game;
        for &mv in moves.iter() {
            g = g.make_move_unchecked(&ts, mv).unwrap();
            self.move_history.push((g.zobrist,mv));
        }
        self.update_game(g);
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

/// Load nnue, syzygy, openings
impl Explorer {

    pub fn add_nnue(&mut self, mut nn: NNUE4) {
        #[cfg(feature = "nnue")]
        {
            #[cfg(feature = "prev_accum")]
            nn.ft.reset_accum(&self.game);
            #[cfg(not(feature = "prev_accum"))]
            nn.ft.reset_feature_trans(&self.game);

            self.nnue = Some(nn);
        }
    }

    pub fn load_nnue<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        #[cfg(feature = "nnue")]
        {
            let mut nn = NNUE4::read_nnue(path)?;

            #[cfg(feature = "prev_accum")]
            nn.ft.reset_accum(&self.game);
            #[cfg(not(feature = "prev_accum"))]
            nn.ft.reset_feature_trans(&self.game);

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
#[cfg(feature = "nope")]
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

/// Get PV
impl ExHelper {
    // pub fn get_pv(&self, ts: &'static Tables, st: &ABStack) -> Vec<Move> {
    pub fn get_pv(&self, ts: &Tables, st: &ABStack) -> Vec<Move> {
        st.pvs.to_vec()
    }
}

/// Entry points
impl Explorer {

    // pub fn explore(&self, ts: &'static Tables) -> (Option<(Move,ABResult)>,SearchStats) {
    pub fn explore(&mut self, ts: &Tables) -> (Option<(Move,ABResult)>,SearchStats) {
        let (ress,moves,stats) = self.lazy_smp_2(ts);
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
impl Explorer {

    pub fn reset_stop(&self) {
        self.stop.store(false, SeqCst);
        // let mut w = self.best_mate.write();
        // *w = None;
        self.best_mate.store(-1, SeqCst);
    }

    #[allow(unused_labels,unused_doc_comments)]
    // pub fn lazy_smp_2(&self, ts: &'static Tables) -> (ABResults,Vec<Move>,SearchStats) {
    pub fn lazy_smp_2(&mut self, ts: &Tables) -> (ABResults,Vec<Move>,SearchStats) {

        #[cfg(feature = "one_thread")]
        let max_threads = 1;
        #[cfg(not(feature = "one_thread"))]
        let max_threads = if let Some(x) = self.cfg.num_threads {
            x as i8
        } else {
            let max_threads = num_cpus::get_physical();
            max_threads as i8
        };

        while self.per_thread_data.len() < max_threads as usize {
            self.per_thread_data.push(Some(PerThreadData::default()));
        }

        debug!("lazy_smp_2 max_threads = {:?}", max_threads);

        if self.cfg.clear_table {
            debug!("clearing tt");
            self.clear_tt();
            #[cfg(feature = "lockless_hashmap")]
            self.ptr_tt.increment_cycle();
        } else {
            #[cfg(feature = "lockless_hashmap")]
            self.ptr_tt.increment_cycle();
        }

        self.reset_stop();

        let (tx,rx): (ExSender,ExReceiver) = crossbeam::channel::unbounded();

        let out: Arc<RwLock<(Depth,ABResults,Vec<Move>, SearchStats)>> =
            Arc::new(RwLock::new((0, ABResults::ABUninit, vec![], SearchStats::default())));

        // let thread_counter = Arc::new(AtomicI8::new(0));
        // let best_depth     = Arc::new(AtomicI16::new(0));
        let thread_counter = Arc::new(CachePadded::new(AtomicI8::new(0)));
        let best_depth     = Arc::new(CachePadded::new(AtomicI16::new(0)));

        let t0 = Instant::now();
        // std::thread::sleep(Duration::from_micros(100));

        #[cfg(feature = "basic_time")]
        let t_max = Duration::from_secs_f64(self.timer.settings.increment[self.side]);

        #[cfg(feature = "basic_time")]
        debug!("searching with t_max = {:?}", t_max);

        // #[cfg(not(feature = "basic_time"))]
        // let cur_ply = self.current_ply.unwrap_or(1);
        // #[cfg(not(feature = "basic_time"))]
        // let (t_opt,t_max) = self.timer.allocate_time(self.game.state.side_to_move, cur_ply);
        // // debug!("searching with (t_opt,t_max) = ({:?},{:?})", t_opt, t_max);

        #[cfg(not(feature = "basic_time"))]
        let mut timer = TimeManager::new(self.time_settings);
        // let mut timer = TimeManager::new(self.time_settings);
        #[cfg(not(feature = "basic_time"))]
        debug!("searching with time limit (soft,hard) = ({:.3},{:.3})",
               timer.limit_soft as f64 / 1000.0,
               timer.limit_hard as f64 / 1000.0);

        // let t_max = self.timer.allocate_time()

        // let root_moves = MoveGen::gen_all(ts, &self.game);
        let root_moves: Vec<Move> = vec![];
        let mut per_thread_data = vec![None; self.per_thread_data.len()];

        // #[cfg(feature = "nope")]
        // crossbeam::scope(|s| {
        //     trace!("spawning listener");
        //     /// Dispatch listener
        //     let handle_listener = s.spawn(|_| {
        //         self.lazy_smp_listener(
        //             rx,
        //             best_depth.clone(),
        //             thread_counter.clone(),
        //             t0,
        //             out.clone(),
        //         );
        //     });
        //     trace!("sending stop message to listener");
        //     tx.send(ExMessage::Stop).unwrap();
        //     handle_listener.join().unwrap();
        // }).unwrap();
        // trace!("done");

        // #[cfg(feature = "nope")]
        crossbeam::scope(|s| {

            // let ord = SeqCst;

            let mut handles = vec![];
            /// Dispatch threads
            for thread_id in 0..max_threads as usize {
                trace!("Spawning thread, id = {}", thread_id);

                let thread_data = self.per_thread_data[thread_id].take()
                    .unwrap_or_default();

                let mut helper = self.build_exhelper(
                    thread_id,
                    self.cfg.max_depth,
                    best_depth.clone(),
                    root_moves.clone(),
                    tx.clone(),
                    thread_data,
                );

                /// 4 MB is needed to prevent stack overflow
                let size = 1024 * 1024 * 4;
                let handle: ScopedJoinHandle<PerThreadData> = s.builder()
                    .stack_size(size)
                    .spawn(move |_| {
                        helper.lazy_smp_single(ts);
                        PerThreadData::new(helper.material_table, helper.pawn_table)
                    }).unwrap();

                handles.push((thread_id,handle));

                thread_counter.fetch_add(1, SeqCst);
                trace!("Spawned thread, count = {}", thread_counter.load(SeqCst));
            }

            let (tx_stop,rx_stop) = crossbeam::channel::unbounded();

            /// Dispatch listener
            let handle_listener = s.spawn(|_| {
                self.lazy_smp_listener(
                    rx,
                    rx_stop,
                    best_depth.clone(),
                    thread_counter.clone(),
                    t0,
                    out.clone(),
                );
            });

            /// stoppage checking loop
            'outer: loop {

                // let best_move_instability = 1 + 2 * best_move_changes / max_threads as u32;

                /// Check for out of time stop
                if timer.should_stop() {
                    debug!("breaking loop (Time),  d: {}", best_depth.load(Relaxed));
                    self.stop.store(true, SeqCst);
                    // drop(tx);
                    break 'outer;
                }

                let d = best_depth.load(Relaxed);
                /// Max depth reached, halt
                if d >= self.cfg.max_depth {
                    debug!("breaking loop (Max Depth)");
                    self.stop.store(true, SeqCst);
                    break 'outer;
                }

                /// Found mate, halt
                // if self.best_mate.read().is_some() {
                if self.best_mate.load(Relaxed) != -1 {
                    #[cfg(not(feature = "basic_time"))]
                    // let t1 = Instant::now().checked_duration_since(t0).unwrap();
                    debug!("breaking loop (Mate),  d: {}", d);
                    self.stop.store(true, SeqCst);
                    break 'outer;
                }

                if self.stop.load(Relaxed) {
                    debug!("breaking loop (External stop),  d: {}", d);
                    break 'outer;
                }

                std::thread::sleep(Duration::from_millis(1));
                // std::thread::sleep(Duration::from_millis(10));
            }

            trace!("sending stop message to listener");
            tx_stop.send(()).unwrap_or(());
            tx.send(ExMessage::Stop).unwrap_or(());
            handle_listener.join().unwrap();

            trace!("joining threads");
            for (thread_id,handle) in handles.into_iter() {
                trace!("joining thread: {}", thread_id);
                let thread_data = handle.join().unwrap();
                per_thread_data[thread_id] = Some(thread_data);
            }

            trace!("exiting lazy_smp_2 loop");

        }).unwrap();
        trace!("exiting lazy_smp_2 scoped");

        for (thread_id,thread_data) in per_thread_data.into_iter().enumerate() {
            self.per_thread_data[thread_id] = thread_data;
        }

        let (d,mut out,moves,mut stats) = {
            let r = out.read();
            r.clone()
        };
        stats.max_depth = Max(d as u32);

        // stats.ph_hits   = self.ph_rw.hits.load(Ordering::Relaxed);
        // stats.ph_misses = self.ph_rw.misses.load(Ordering::Relaxed);

        if let Some(res) = out.get_result() {

            // let out = if self.game.move_is_legal(ts, res.mv.unwrap(), self.game.state.side_to_move) {
            let out = if MoveGen::new_move_is_legal(ts, &self.game, res.mv.unwrap()) {
                out
            } else {
                debug!("best move wasn't legal? {:?}\n{:?}\n{:?}", self.game, self.game.to_fen(), res);
                // ABResults::ABNone
                panic!();
            };
            (out,moves,stats)
        } else {
            (out,moves,stats)
        }
    }

}

/// Lazy SMP Listener
impl Explorer {

    fn lazy_smp_listener(
        &self,
        rx:               ExReceiver,
        rx_stop:          Receiver<()>,
        best_depth:       Arc<CachePadded<AtomicI16>>,
        thread_counter:   Arc<CachePadded<AtomicI8>>,
        t0:               Instant,
        out:              Arc<RwLock<(Depth,ABResults,Vec<Move>,SearchStats)>>,
    ) {
        let mut any_move_stored = false;
        loop {

            crossbeam::select! {
                recv(rx_stop) -> _ => {
                    trace!("lazy_smp_listener Stop 0");
                    break;
                },
                recv(rx) -> msg => match msg {
                    Ok(ExMessage::Stop) => {
                        trace!("lazy_smp_listener Stop 1");
                        break;
                    },
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
                                    if depth > best_depth.load(SeqCst)
                                        // || bestres.score > CHECKMATE_VALUE - MAX_SEARCH_PLY as Score - 1
                                        || bestres.score > CHECKMATE_VALUE - (MAX_SEARCH_PLY as Score * 2)
                                        || !any_move_stored
                                    {
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

                                        if bestres.score > CHECKMATE_VALUE - (MAX_SEARCH_PLY as Score * 2) {
                                        // if bestres.score > CHECKMATE_VALUE - MAX_SEARCH_PLY as Score - 1 {
                                            let k = CHECKMATE_VALUE - bestres.score.abs();
                                            debug!("Found mate in {}: d({}), {:?}",
                                                // bestres.score, bestres.moves.front());
                                                k, depth, bestres.mv);

                                            // let mut best = self.best_mate.write();
                                            // *best = Some(k as Depth);

                                            self.best_mate.store(k as Depth, SeqCst);

                                            self.stop.store(true, SeqCst);

                                            let mut w = out.write();
                                            *w = (depth, res, moves, w.3 + *stats);
                                            // *w = (depth, scores, None);
                                            break;
                                        } else {
                                            let mut w = out.write();
                                            *w = (depth, res, moves, w.3 + *stats);
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
                                w.3 = w.3 + *stats;
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

            // match rx.try_recv() {

            #[cfg(feature = "nope")]
            match rx.recv() {
                Ok(ExMessage::Stop) => {
                    trace!("lazy_smp_listener Stop");
                    break;
                },
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
                                if depth > best_depth.load(SeqCst)
                                    // || bestres.score > CHECKMATE_VALUE - MAX_SEARCH_PLY as Score - 1
                                    || bestres.score > CHECKMATE_VALUE - (MAX_SEARCH_PLY as Score * 2)
                                    || !any_move_stored
                                {
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

                                    if bestres.score > CHECKMATE_VALUE - (MAX_SEARCH_PLY as Score * 2) {
                                    // if bestres.score > CHECKMATE_VALUE - MAX_SEARCH_PLY as Score - 1 {
                                        let k = CHECKMATE_VALUE - bestres.score.abs();
                                        debug!("Found mate in {}: d({}), {:?}",
                                            // bestres.score, bestres.moves.front());
                                            k, depth, bestres.mv);

                                        // let mut best = self.best_mate.write();
                                        // *best = Some(k as Depth);

                                        self.best_mate.store(k as Depth, SeqCst);

                                        self.stop.store(true, SeqCst);

                                        let mut w = out.write();
                                        *w = (depth, res, moves, w.3 + stats);
                                        // *w = (depth, scores, None);
                                        break;
                                    } else {
                                        let mut w = out.write();
                                        *w = (depth, res, moves, w.3 + stats);
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
                    trace!("Breaking thread counter loop (Disconnect)");
                    break;
                },
            }
        }
        trace!("exiting listener");
    }
}

/// Lazy SMP Iterative Deepening with Aspiration window
#[cfg(feature = "nope")]
impl ExHelper {

    fn lazy_smp_single(
        &self,
        // ts:               &'static Tables,
        ts:               &Tables,
    ) {

        let mut stack = ABStack::new_with_moves(&self.move_history);
        let mut stats = SearchStats::default();

        let skip_size = Self::SKIP_SIZE[self.id % Self::SKIP_LEN];
        let start_ply = Self::START_PLY[self.id % Self::SKIP_LEN];

        let mut cur_depth = start_ply + 1;

        let mut best_value;
        let mut delta = -CHECKMATE_VALUE;
        let mut alpha = -CHECKMATE_VALUE;
        let mut beta  = CHECKMATE_VALUE;

        /// Iterative deepening
        while !self.stop.load(SeqCst)
            && cur_depth <= self.cfg.max_depth
            && self.best_mate.read().is_none()
        {
            if cur_depth >= 4 {
            }

            let mut res;

            let mut failed_high = 0;
            loop {
                stack.pvs.fill(Move::NullMove);

                let res2 = self.ab_search_single(ts, &mut stats, &mut stack, Some((alpha,beta)), cur_depth);

                let res3 = match res2 {
                    ABResults::ABList(r,_) => r.clone(),
                    _                      => unimplemented!()
                };
                res = Some(res2);

                best_value = res3.score;

                {
                    let mut mvs = self.root_moves.borrow_mut();
                    let pv_mv  = stack.pvs[0];
                    let pv_idx = mvs.iter().position(|&mv| mv == pv_mv).unwrap();
                    mvs.swap(0, pv_idx);
                }

                if self.stop.load(SeqCst) { break; }

                if best_value <= alpha {

                    // beta = (alpha + beta) / 2;
                    // alpha = (best_value - delta).max(-CHECKMATE_VALUE);

                    beta = alpha.checked_add(beta).unwrap() / 2;
                    alpha = best_value.checked_sub(delta).unwrap().max(-CHECKMATE_VALUE);

                } else if best_value >= beta {
                    // beta = (best_value + delta).min(CHECKMATE_VALUE);
                    beta = best_value.checked_add(delta).unwrap().min(CHECKMATE_VALUE);
                    failed_high += 1;
                } else {
                    break;
                }

                delta += delta / 4 + 5;

                assert!(alpha >= -CHECKMATE_VALUE);
                assert!(beta <= CHECKMATE_VALUE);
            }


            // let depth2 = 

            /// Send result to listener
            if !self.stop.load(SeqCst) && cur_depth >= self.best_depth.load(SeqCst) {
                let moves = if self.cfg.return_moves {
                    let mut v = stack.pvs.to_vec();
                    v.retain(|&mv| mv != Move::NullMove);
                    v
                } else { vec![] };

                if let Some(res) = res {
                    match self.tx.try_send(ExMessage::Message(cur_depth, res, moves, stats)) {
                        Ok(_)  => {
                            stats = SearchStats::default();
                        },
                        Err(_) => {
                            trace!("tx send error 0: id: {}, depth {}", self.id, cur_depth);
                            break;
                        },
                    }
                }
            }

            // cur_depth += skip_size;
            cur_depth += 1;
        }

        match self.tx.try_send(ExMessage::End(self.id)) {
            Ok(_)  => {},
            Err(_) => {
                trace!("tx send error 1: id: {}, depth {}", self.id, cur_depth);
            },
        }

        trace!("exiting lazy_smp_single, id = {}", self.id);
    }

}

/// Lazy SMP Iterative Deepening loop
impl ExHelper {

    const SKIP_LEN: usize = 20;
    const SKIP_SIZE: [Depth; Self::SKIP_LEN] =
        [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
    const START_PLY: [Depth; Self::SKIP_LEN] =
        [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];

    // #[cfg(feature = "nope")]
    fn lazy_smp_single(
        &mut self,
        // ts:               &'static Tables,
        ts:               &Tables,
        // max_threads:      i8,
    // ) -> PerThreadData {
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
            // && self.best_mate.read().is_none()
            && self.best_mate.load(Relaxed) == -1
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
                match self.tx.try_send(ExMessage::Message(depth, res, moves, Box::new(stats))) {
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

        // if let Some(nn) = &self.nnue {
        //     let stats = nn.ft.stats;
        //     stats.print_stats();
        // }

        trace!("exiting lazy_smp_single, id = {}", self.id);

        // /// XXX: this might be slow? only clones once per thread per search
        // // PerThreadData::new(self.material_table.clone(), self.pawn_table.clone())
        // PerThreadData::new(self.material_table, self.pawn_table)

    }

}

