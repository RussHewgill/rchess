
use crate::pawn_hash_table::PHTable;
use crate::stack::ABStack;
use crate::tuning::SParams;
use crate::types::*;
use crate::explore::*;
use crate::searchstats::SearchStats;
use crate::lockless_map::TransTable;
use crate::pawn_hash_table::*;

#[cfg(feature = "syzygy")]
use crate::syzygy::SyzygyTB;
use crate::sf_compat::NNUE4;

use std::time::{Instant,Duration};
use std::sync::Arc;
use std::sync::atomic::{Ordering,Ordering::SeqCst,Ordering::Relaxed,AtomicI16,AtomicBool};
use std::cell::{Cell,RefCell};
use std::thread::JoinHandle;

use itertools::Itertools;
use parking_lot::{Mutex,RwLock,Condvar};
use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use crossbeam::thread::ScopedJoinHandle;
use crossbeam::utils::CachePadded;

#[derive(Debug,Clone)]
pub struct Explorer2 {
    pub side:          Color,
    pub game:          Game,
    pub current_ply:   Option<Depth>,

    #[cfg(feature = "basic_time")]
    pub timer:         Timer,
    #[cfg(not(feature = "basic_time"))]
    pub time_settings: TimeSettings,

    pub stop:          Arc<CachePadded<AtomicBool>>,
    pub best_mate:     Arc<RwLock<Option<Depth>>>,
    pub best_depth:    Arc<CachePadded<AtomicI16>>,

    pub tx:            ExSender,
    pub rx:            ExReceiver,

    pub cfg:           ExConfig,

    pub search_params: SParams,

    #[cfg(feature = "syzygy")]
    pub syzygy:        Option<Arc<SyzygyTB>>,
    // pub opening_book:  Option<Arc<OpeningBook>>,

    pub nnue:          Option<NNUE4>,

    #[cfg(feature = "lockless_hashmap")]
    pub ptr_tt:        Arc<TransTable>,

    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_rf:         TTReadFactory,
    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_w:          TTWrite,

    // pub eval_hashmap:  (EVReadFactory<Score>,EVWrite<Score>),

    // pub ph_rw:         (PHReadFactory,PHWrite),
    pub ph_rw:         PHTableFactory,

    pub move_history:  Vec<(Zobrist, Move)>,
    // pub pos_history:   HashMap<Zobrist,u8>,

}

#[derive(Debug)]
pub struct ThreadPool {
    pub handles:      Vec<JoinHandle<()>>,

    pub waits:        Vec<Arc<(Mutex<bool>,Condvar)>>,
    // pub wait:         (Sender<()>,Arc<Receiver<()>>),

}

impl ThreadPool {
    pub fn new() -> Self {
        // let wait = crossbeam_channel::unbounded();
        // let wait = (wait.0,Arc::new(wait.1));
        // let wait = Arc::new((Mutex::new(false), Condvar::new()));
        Self {
            handles: vec![],
            // wait,
            waits: vec![],
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
    ) -> (Thread, Arc<(Mutex<bool>,Condvar)>) {
    // ) -> Thread {

        let wait = Arc::new((Mutex::new(false), Condvar::new()));

        let thread = Thread {
            id,

            side:            self.side,
            game:            self.game.clone(),

            stop:            self.stop.clone(),
            best_mate:       self.best_mate.clone(),
            best_depth:      self.best_depth.clone(),

            cfg:             self.cfg.clone(),
            params:          self.search_params.clone(),

            wait:            wait.clone(),

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

            stats:           RefCell::new(SearchStats::default()),
            stack:           RefCell::new(ABStack::new()),

        };

        (thread, wait)
        // thread
    }
}

/// wakeup, sleep
impl ThreadPool {
    pub fn wakeup_threads(&self) {
        for wait in self.waits.iter() {
            let mut lock = wait.0.lock();
            let cv = &wait.1;
            *lock = true;
            cv.notify_all();
        }
    }

    pub fn sleep_threads(&self) {
        unimplemented!()
    }
}

/// spawn_threads
impl Explorer2 {

    pub fn spawn_threads(&mut self, threadpool: &mut ThreadPool) {

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

            let (thread,wait) = self.build_thread(thread_id, self.tx.clone());
            // let thread = self.build_thread(thread_id, threadpool.wait.1.clone());

            let handle = std::thread::spawn(move || {
                thread.idle();
            });

            threadpool.handles.push(handle);
            threadpool.waits.push(wait);

            thread_id += 1;
        }

    }

}

#[derive(Debug,Clone)]
pub struct Thread {

    pub id:              usize,

    pub side:            Color,
    pub game:            Game,

    #[cfg(feature = "syzygy")]
    pub syzygy:          Option<Arc<SyzygyTB>>,
    pub nnue:            Option<RefCell<NNUE4>>,

    pub cfg:             ExConfig,
    pub params:          SParams,

    pub wait:            Arc<(Mutex<bool>,Condvar)>,

    pub stop:            Arc<CachePadded<AtomicBool>>,
    pub best_mate:       Arc<RwLock<Option<Depth>>>,
    pub best_depth:      Arc<CachePadded<AtomicI16>>,
    pub tx:              ExSender,

    #[cfg(feature = "lockless_hashmap")]
    pub ptr_tt:          Arc<TransTable>,

    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_r:            TTRead,
    #[cfg(not(feature = "lockless_hashmap"))]
    pub tt_w:            TTWrite,

    pub ph_rw:           PHTable,

    pub move_history:    Vec<(Zobrist, Move)>,

    pub stats:           RefCell<SearchStats>,
    pub stack:           RefCell<ABStack>,

}

impl Thread {
}

impl Thread {

    pub fn idle(&self) {

        let (lock,cvar) = &*self.wait;
        // cvar.wait(lock.load(Relaxed));
        let mut lock = lock.lock();
        cvar.wait(&mut lock);

        // match self.wait.recv() {
        //     Ok(()) => {},
        //     Err(e) => panic!("wat"),
        // }

        println!("thread (id: {:>2}) waking up", self.id);

    }

    pub fn clear(&self, g: Game) {
        let mut stack = self.stack.borrow_mut();
        stack.clear_history();
        // unimplemented!()
    }

}







