
use crate::evmap_tables;
use crate::explore::*;
use crate::opening_book::*;
use crate::pawn_hash_table::PHTableFactory;
use crate::qsearch::exhelper_once;
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
use crate::alphabeta::*;

pub use self::td_tree::*;
pub use self::td_builder::*;
pub use self::sprt::*;

use std::collections::HashMap;
use std::path::Path;
use std::time::{Instant,Duration};

use serde::{Serialize,Deserialize};

use rand::{prelude::{StdRng,SliceRandom},Rng,SeedableRng};
use rand::distributions::{Uniform,uniform::SampleUniform};

use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use std::sync::atomic::{Ordering,AtomicU64};
use rayon::prelude::*;

mod td_tree {
    use super::*;

    #[derive(Debug,Eq,PartialEq,Clone,Serialize,Deserialize)]
    pub struct TDTree<T: PartialEq> {
        arena:  Vec<TDNode<T>>,
    }

    impl<T: PartialEq> TDTree<T> {
        pub fn empty() -> Self {
            Self {
                arena: vec![],
            }
        }

        pub fn insert(&mut self, parent: Option<usize>, val: T) -> usize {
            if let Some(parent) = parent {
                let idx = self.node(Some(parent), val);
                self.arena[parent].children.push(idx);
                idx
            } else {
                let idx = self.node(None, val);
                idx
            }
        }

        fn node(&mut self, parent: Option<usize>, val: T) -> usize {
            for node in &self.arena {
                if node.val == val {
                    return node.idx;
                }
            }
            let idx = self.arena.len();
            self.arena.push(TDNode::new(idx, val, parent));
            idx
        }

    }

    #[derive(Debug,Eq,PartialEq,Clone,Serialize,Deserialize)]
    pub struct TDNode<T> {
        idx:       usize,
        val:       T,
        parent:    Option<usize>,
        children:  Vec<usize>
    }

    impl<T> TDNode<T> {
        pub fn new(idx: usize, val: T, parent: Option<usize>) -> Self {
            Self {
                idx,
                val,
                parent,
                children: vec![],
            }
        }
    }

}

mod td_builder {
    use std::collections::VecDeque;

    use super::*;
    use crate::builder_field;

    // use derive_builder::Builder;
    // #[derive(Debug,Builder,PartialEq,Clone)]
    // #[builder(pattern = "owned")]
    // #[builder(setter(prefix = "with"))]

    #[derive(Debug,Clone)]
    pub struct TDBuilder {
        // opening:         Option<OBSelection>,
        min_depth:       Depth,
        max_depth:       Depth,
        nodes_per_pos:   Option<u64>,
        num_positions:   Option<u64>,
        num_threads:     usize,
        time:            f64,
        print:           bool,
    }

    impl TDBuilder {
        pub fn new() -> Self {
            Self {
                // opening:        None,
                min_depth:      2,
                max_depth:      5,
                nodes_per_pos:  None,
                num_positions:  None,
                num_threads:    1,
                time:           0.5,
                print:          true,
            }
        }
        // builder_field!(opening, Option<OBSelection>);
        builder_field!(min_depth, Depth);
        builder_field!(max_depth, Depth);
        builder_field!(nodes_per_pos, Option<u64>);
        builder_field!(num_positions, Option<u64>);
        builder_field!(num_threads, usize);
        builder_field!(time, f64);
        builder_field!(print, bool);
    }

    /// Generate single
    impl TDBuilder {

        // fn recurse(&self, ts: &Tables, mut ex: &mut Explorer, )

        fn watch_sfen(
            t0:       Instant,
            sfen_n:   Arc<(AtomicU64,AtomicU64)>,
            rx:       Receiver<TrainingData>,
        ) {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1000));

                let sfens = sfen_n.0.load(Ordering::Relaxed);
                let moves = sfen_n.1.load(Ordering::Relaxed);

                let t1 = t0.elapsed().as_secs_f64();
                eprintln!("{:>6} games, {:>6} sfens, {:.1}s, avg {:.1} sfens / sec, {:.1} moves / sec",
                          sfens, moves, t1,
                          sfens as f64 / t1,
                          moves as f64 / t1,
                );

                match rx.try_recv() {
                    Err(TryRecvError::Disconnected) => break,
                    _                               => {},
                }
            }
        }

        pub fn do_explore<P: AsRef<Path> + Send>(
            &self,
            ts:         &'static Tables,
            ob:         &OpeningBook,
            count:      u64,
            print:      bool,
            mut rng:    StdRng,
            save_bin:   bool,
            path:       P,
        ) -> std::io::Result<()> {

            let (tx,rx): (Sender<TrainingData>,
                          Receiver<TrainingData>) =
                crossbeam::channel::unbounded();

            let sfen_n = Arc::new((AtomicU64::new(0), AtomicU64::new(0)));

            let t0 = Instant::now();

            crossbeam::scope(|s| {
                s.spawn(|_| Self::save_listener(save_bin, count, path, rx.clone()));

                if print {
                    s.spawn(|_| Self::watch_sfen(t0, sfen_n.clone(), rx.clone()));
                }

                for id in 0..self.num_threads {
                    let rng2: u64 = rng.gen();
                    let mut rng2: StdRng = SeedableRng::seed_from_u64(1234u64);
                    let tx2 = tx.clone();
                    s.builder()
                        .spawn(|_| self._do_explore(ts, ob, count, rng2, sfen_n.clone(), tx2))
                        .unwrap();
                }
                drop(tx);
            }).unwrap_or_else(|_| ());

            Ok(())
        }

        fn save_listener<P: AsRef<Path>>(
            save_bin:   bool,
            count:      u64,
            path:       P,
            rx:         Receiver<TrainingData>,
        ) {
            // let mut out = vec![];
            let mut file = std::fs::File::create(path).unwrap();
            let mut n = 0;

            loop {
                match rx.recv() {
                    Ok(td) => {
                        // out.push(td);
                        // eprintln!("rx, len out = {:?}", out.len());
                        // eprintln!("rx, len out = {:?}", n);
                        n += 1;
                        match TrainingData::save_into(save_bin, &mut file, &td) {
                            Ok(_)  => {},
                            Err(e) => {
                                eprintln!("save_all error = {:?}", e);
                            },
                        }
                        if n >= count {
                            break;
                        }
                    },
                    // Err(TryRecvError::Empty)    => {
                    //     std::thread::sleep(std::time::Duration::from_millis(1));
                    // },
                    // Err(TryRecvError::Disconnected)    => {
                    Err(_)    => {
                        trace!("Breaking save listener loop (Disconnect)");
                        break;
                    },
                }
            }
        }

        pub fn _do_explore(
            &self,
            ts:         &'static Tables,
            ob:         &OpeningBook,
            count:      u64,
            mut rng:    StdRng,
            sfen_n:     Arc<(AtomicU64,AtomicU64)>,
            tx:         Sender<TrainingData>,
        ) {
            unimplemented!()
        }

        #[cfg(feature = "nope")]
        pub fn _do_explore(
            &self,
            ts:         &'static Tables,
            ob:         &OpeningBook,
            count:      u64,
            mut rng:    StdRng,
            sfen_n:     Arc<(AtomicU64,AtomicU64)>,
            tx:         Sender<TrainingData>,
        ) {

            let opening_ply = 16;

            let mut g = Game::from_fen(ts, STARTPOS).unwrap();
            let timesettings = TimeSettings::new_f64(0.0, self.time);
            // let mut ex = Explorer::new(g.state.side_to_move, g.clone(), self.max_depth, timesettings);
            let mut ex_white = Explorer::new(g.state.side_to_move, g.clone(), self.max_depth, timesettings);
            let mut ex_black = Explorer::new(g.state.side_to_move, g.clone(), self.max_depth, timesettings);
            // ex.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();
            ex_white.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();
            ex_black.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();

            // ex.cfg.num_threads = Some(1);
            ex_white.cfg.num_threads = Some(1);
            ex_black.cfg.num_threads = Some(1);
            // ex.cfg.clear_table = false;
            ex_white.cfg.clear_table = false;
            ex_black.cfg.clear_table = false;

            // let mut ex_white = ex.clone();
            // let mut ex_black = ex.clone();
            ex_black.side = Black;

            let mut exs = [ex_white, ex_black];

            let mut s = OBSelection::Random(rng);

            'outer: for gk in 0..count {

                let (mut g,opening) = ob.start_game(&ts, Some(opening_ply), &mut s).unwrap();

                let mut ex;

                // let fen = "7k/8/8/8/8/8/4Q3/7K w - - 0 1"; // Queen endgame, #7
                // let mut g = Game::from_fen(ts, fen).unwrap();

                let mut ply = opening.len();

                let mut moves = vec![];
                // let mut result: Option<TDOutcome> = None;

                let result: Option<TDOutcome> = 'game: loop {

                    ex = &mut exs[g.state.side_to_move];

                    assert_eq!(ex.side, g.state.side_to_move);

                    ex.game = g.clone();
                    // ex.update_game(g.clone());
                    ex.clear_tt();

                    let (res,_,stats) = ex.lazy_smp_2(ts);

                    match res.get_result() {
                        Some(res) => {

                            let skip = false;

                            if !skip { sfen_n.1.fetch_add(1, Ordering::SeqCst); }
                            let e = TDEntry::new(res.mv, res.score, skip);

                            moves.push(e);

                            match g.make_move_unchecked(ts, res.mv) {
                                Ok(g2) => {

                                    if res.score > CHECKMATE_VALUE - 100 {
                                        trace!("score > mate value, res = {:?}", res);
                                        break 'game Some(TDOutcome::Win(!g.state.side_to_move));
                                    } else if res.score.abs() > CHECKMATE_VALUE - 100 {
                                        trace!("score < mate value, res = {:?}", res);
                                        break 'game Some(TDOutcome::Win(g.state.side_to_move));
                                    }

                                    g = g2;
                                },
                                Err(e) => {
                                    let e = match e {
                                        GameEnd::Checkmate { win } => TDOutcome::Win(win),
                                        GameEnd::Stalemate         => TDOutcome::Stalemate,
                                        _                          => panic!("GameEnd other ?? {:?}", e),
                                    };
                                    break 'game Some(e);
                                },
                            }
                        },
                        None => {
                            trace!("None res = {:?}", res);
                            break 'game None;
                        },
                    }

                };

                debug!("game done, result = {:?}", result);

                let n = moves.iter().filter(|x| !x.skip).count();
                sfen_n.0.fetch_add(1, Ordering::SeqCst);

                if let Some(result) = result {
                    // let opening = opening.iter().map(|&mv| PackedMove::convert(mv)).collect();
                    let mut td = TrainingData {
                        result,
                        opening,
                        moves,
                    };

                    match tx.send(td) {
                        Ok(_)  => {},
                        // Err(e) => eprintln!("tx.send error = {:?}", e),
                        Err(e) => {},
                    }
                } else {
                    trace!("result = None ??");
                }

                let n_fens = sfen_n.0.load(Ordering::SeqCst);
                if n_fens > count { break 'outer; }

                if let Some(n) = self.num_positions {
                    let n_moves = sfen_n.1.load(Ordering::SeqCst);
                    if n_moves > n { break 'outer; }
                }

                // eprintln!("_do_explore: breaking outer loop");
                // break 'outer;
            }
        }

    }


}

mod sprt {
    use super::*;

    fn log_likelyhood(x: f64) -> f64 {
        1.0 / (1.0 + 10.0f64.powf(-x / 400.0))
    }

    fn ll_ratio((win,draw,loss): (u32,u32,u32), elo0: f64, elo1: f64) -> f64 {
        if win == 0 || draw == 0 || loss == 0 {
            return 0.0;
        }
        let (w,d,l) = (win as f64, draw as f64, loss as f64);

        let n = w + d + l;

        let w = w / n;
        let d = d / n;
        let l = l / n;

        let s     = w + d / 2.0;
        let m2    = w + d / 4.0;
        let var   = m2 - s.powi(2);
        let var_s = var / n;

        let s0 = log_likelyhood(elo0);
        let s1 = log_likelyhood(elo1);

        (s1 - s0) * (2.0 * s - s0 - s1) / var_s / 2.0
    }

    pub fn sprt(
        (win,draw,loss): (u32,u32,u32),
        (elo0,elo1): (f64,f64),
        alpha: f64,
        beta:  f64,
    ) -> Option<bool> {

        let llr = ll_ratio((win,draw,loss), elo0, elo1);

        let la = f64::ln(beta / (1.0 - alpha));
        let lb = f64::ln((1.0 - beta) / alpha);

        if llr > lb {
            return Some(true);
        } else if llr < la {
            return Some(false);
        } else {
            None
        }
    }

}

#[derive(Debug,Eq,PartialEq,Clone,Copy,Serialize,Deserialize)]
pub enum TDOutcome {
    Win(Color),
    Draw,
    Stalemate,
}

#[derive(Debug,Eq,PartialEq,Clone,Serialize,Deserialize)]
// #[derive(Debug,Eq,PartialEq,Clone)]
pub struct TrainingData {
    pub result:       TDOutcome,
    // pub opening:      Vec<PackedMove>,
    pub opening:      Vec<Move>,
    // pub moves:        TDTree<TDEntry>,
    pub moves:        Vec<TDEntry>,
}

/// Generate data set
impl TrainingData {

    // pub fn generate_training_data<P: AsRef<Path>>(
    //     ts:       &Tables,
    //     ob:       &OpeningBook,
    //     open_ply: usize,
    //     n:        usize,
    //     path:     P,
    // ) -> std::io::Result<()> {
    //     use std::io::Write;

    //     // let mut s = OBSelection::BestN(0);
    //     let mut s = OBSelection::new_random_seeded(1234);

    //     let mut out: Vec<TrainingData> = vec![];

    //     let mut fens = 0;

    //     // loop {
    //     for _ in 0..n {

    //         // let (_,opening) = ob.start_game(&ts, Some(open_ply), &mut s).unwrap();
    //         // eprintln!("opening = {:?}", opening);

    //         // let k0: TrainingData = TDBuilder::new()
    //         //     .opening(opening)
    //         //     .max_depth(5)
    //         //     .time(0.2)
    //         //     .generate_single(&ts)
    //         //     .unwrap();

    //         // fens += k0.moves.len();
    //         // eprintln!("fens = {:?}", fens);
    //         // if fens >= n { break; }
    //         // eprintln!("result = {:?}", k0.result);

    //         // out.push(k0);

    //         Self::save_all(&path, &out)?;
    //     }

    //     Ok(())
    // }

}

/// Init opening
impl TrainingData {
    pub fn init_opening(&self, ts: &Tables) -> Option<Game> {
        let mut g = Game::from_fen(&ts, STARTPOS).unwrap();
        for mv in self.opening.iter() {
            if let Ok(g2) = g.make_move_unchecked(&ts, *mv) {
                g = g2;
            } else {
                return None;
            }
        }
        Some(g)
    }
}

/// Filter only quiet positions
impl TrainingData {

    pub fn filter_quiet(
        ts:               &'static Tables,
        (ev_mid,ev_end):  &(EvalParams,EvalParams),
        tds:              Vec<TrainingData>,
    ) -> Vec<TrainingData> {

        let ncpus = num_cpus::get();
        let g = Game::start_pos(ts);

        let ph_factory = PHTableFactory::new();

        let tds = tds.chunks(tds.len() / ncpus).map(|xs| {
            let ph_rw = ph_factory.handle();
            let exhelper = exhelper_once(ts, &g, White, ev_mid, ev_end, Some(&ph_rw), None);
            (exhelper, xs)
        }).collect::<Vec<(ExHelper, &[TrainingData])>>();

        let out = tds.into_par_iter().map(|(exhelper,xs)| {
            // let out = xs.iter().filter(|td| td._filter_quiet(ts, &exhelper, &mut stats));
            let mut xs: Vec<TrainingData> = xs.to_vec();
            xs.iter_mut().for_each(|td| {
                td._filter_quiet(ts, &exhelper)
            });
            xs
        }).flatten().collect::<Vec<_>>();

        out
    }

    pub fn _filter_quiet(
        &mut self,
        ts:               &Tables,
        exhelper:         &ExHelper,
    ) {
        let mut stats = SearchStats::default();

        let mut g = if let Some(g) = self.init_opening(ts) { g } else { return; };

        let (ev_mid,ev_end) = (&exhelper.cfg.eval_params_mid,&exhelper.cfg.eval_params_end);

        let mut done = false;
        for mut te in self.moves.iter_mut() {

            if done {
                te.skip = true;
                continue;
            }

            if let Ok(g2) = g.make_move_unchecked(&ts, te.mv) {
                g = g2;

                if te.skip
                    || te.mv.filter_all_captures()
                    || !g.state.checkers.is_empty() {
                        te.skip = true;
                    } else {

                        let score   = g.sum_evaluate(&ts, &ev_mid, &ev_end, None);
                        let q_score = exhelper.qsearch_once(&ts, &g, &mut stats);
                        let q_score = g.state.side_to_move.fold(q_score, -q_score);

                        if score != q_score {
                            te.skip = true;
                        } else if score.abs() > DRAW_VALUE - 100 {
                            te.skip = true;
                            done = true;
                        }
                    }


            } else {
                // panic!("bad move: {:?}\n{:?},{:?}", g.to_fen(), g, te.mv);
                break;
            }
        }

    }
}

#[derive(Debug,Eq,PartialEq,Clone,Copy,Serialize,Deserialize)]
pub struct TDEntry {
    // mv:       PackedMove,
    pub mv:       Move,
    // eval:     i8,
    pub eval:     Score,
    pub skip:     bool,
}

impl TDEntry {
    pub fn new(mv: Move, eval: Score, skip: bool) -> Self {
        Self {
            mv,
            // mv: PackedMove::convert(mv),
            // eval: Self::convert_from_score(score),
            eval,
            skip,
        }
    }

    // pub fn convert_to_score(s: i8) -> Score {
    //     unimplemented!()
    // }
    // pub fn convert_from_score(s: Score) -> i8 {
    //     unimplemented!()
    // }

}

/// Load, Save
impl TrainingData {

    pub fn load_all<P: AsRef<Path>>(path: P, count: Option<usize>) -> std::io::Result<Vec<Self>> {
        // let mut b = std::fs::read(path)?;
        // let out: Vec<Self> = bincode::deserialize(&b).unwrap();
        let mut f = std::fs::File::open(path)?;
        let mut out: Vec<Self> = vec![];
        let mut n = 0;
        while let Ok(td) = bincode::deserialize_from(&mut f) {
            out.push(td);
            n += 1;
            if count.map_or(false, |c| n >= c) { break; }
        }
        // let out: Vec<Self> = bincode::deserialize_from(&mut f).unwrap();
        Ok(out)
    }

    // pub fn _save_all(save_bin: bool, mut file: &mut std::fs::File, xs: &Vec<Self>) -> std::io::Result<()> {
    pub fn save_into(save_bin: bool, mut file: &mut std::fs::File, xs: &Self) -> std::io::Result<()> {
        use std::io::Write;

        if save_bin {

            match bincode::serialize_into(&mut file, &xs) {
                Ok(_)  => Ok(()),
                Err(_) => {
                    Ok(())
                },
            }

            // match bincode::serialize(&xs) {
            //     Ok(buf) => {
            //         file.write_all(&buf)
            //     },
            //     Err(e) => {
            //         eprintln!("save_all: bincode = {:?}", e);
            //         Ok(())
            //     }
            // }

        } else {
            // file.write_all(&buf)
            unimplemented!()
        }

    }

    // #[cfg(feature = "nope")]
    pub fn save_all<P: AsRef<Path>>(save_bin: bool, path: P, xs: &Vec<Self>) -> std::io::Result<()> {
        use std::io::Write;
        // let mut buf: Vec<u8> = vec![];

        let mut file = std::fs::File::create(path)?;

        if save_bin {
            // let buf: Vec<u8> = bincode::serialize(&xs).unwrap();
            match bincode::serialize(&xs) {
                Ok(buf) => {
                    file.write_all(&buf)
                    // Ok(file)
                },
                Err(e) => {
                    eprintln!("save_all: bincode = {:?}", e);
                    // Ok(file)
                    Ok(())
                }
            }
        } else {
            // file.write_all(&buf)
            unimplemented!()
        }

        // for x in xs.iter() {
        //     let b: Vec<u8> = bincode::serialize(&x).unwrap();
        //     buf.extend(b.into_iter());
        // }

    }

    #[cfg(feature = "nope")]
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        use std::io::Write;
        let b: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut file = std::fs::File::create(path)?;
        file.write_all(&b)
    }

    #[cfg(feature = "nope")]
    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        use std::io::Write;
        let mut b = std::fs::read(path)?;
        let out: Self = bincode::deserialize(&b).unwrap();
        Ok(out)
    }

}





