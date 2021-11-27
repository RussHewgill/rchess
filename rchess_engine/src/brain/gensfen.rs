
use crate::explore::*;
use crate::opening_book::*;
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
use crate::alphabeta::*;

pub use self::packed_move::*;
pub use self::td_tree::*;
pub use self::td_builder::*;

use std::collections::HashMap;
use std::path::Path;
use std::time::{Instant,Duration};

use serde::{Serialize,Deserialize};

use rand::{prelude::{StdRng,SliceRandom},Rng,SeedableRng};
use rand::distributions::{Uniform,uniform::SampleUniform};

use crossbeam::channel::{Sender,Receiver,RecvError,TryRecvError};
use std::sync::atomic::{Ordering,AtomicU64};

mod packed_move {
    use super::*;
    use packed_struct::prelude::*;
    pub use packed_struct::PackedStruct;

    // #[derive(Debug,Eq,PartialEq,Clone,Copy,Serialize,Deserialize)]
    // pub struct PackedMove2 {
    //     #[serde(serialize_with = "PackedMove2::ser")]
    //     #[serde(deserialize_with = "PackedMove2::de")]
    //     mv:   PackedMove,
    // }

    // impl PackedMove2 {
    //     pub fn ser<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    //         unimplemented!()
    //     }
    //     pub fn de<'de, D>(d: D) -> Result<PackedMove, D::Error>
    //     where D: serde::Deserializer<'de>
    //     {
    //         unimplemented!()
    //     }
    // }

    #[derive(Debug,Eq,PartialEq,Clone,Copy,PackedStruct,Serialize,Deserialize)]
    // #[derive(Debug,Eq,PartialEq,Clone,Copy,PackedStruct)]
    #[packed_struct(bit_numbering = "msb0")]
    pub struct PackedMove {
        #[packed_field(bits = "0..6")]
        _from:   Integer<u8, packed_bits::Bits::<6>>,
        #[packed_field(bits = "6..12")]
        _to:     Integer<u8, packed_bits::Bits::<6>>,
        #[packed_field(bits = "13..15")]
        _prom:   Integer<u8, packed_bits::Bits::<3>>,
    }

    impl PackedMove {

        pub fn convert(mv: Move) -> Self {
            match mv {
                Move::Promotion { new_piece, .. } | Move::PromotionCapture { new_piece, .. } =>
                    Self::new(mv.sq_from().into(), mv.sq_to().into(), Some(new_piece)),
                _ => Self::new(mv.sq_from().into(), mv.sq_to().into(), None),
            }
        }

        pub fn from(&self) -> u8 {
            u8::from(self._from)
        }
        pub fn to(&self) -> u8 {
            u8::from(self._to)
        }
        pub fn prom(&self) -> Option<Piece> {
            Self::convert_to_piece(u8::from(self._prom))
        }

        pub fn new(from: u8, to: u8, prom: Option<Piece>) -> Self {
            Self {
                _from:  from.into(),
                _to:    to.into(),
                _prom:  Self::convert_from_piece(prom).into(),
            }
        }

        fn convert_from_piece(pc: Option<Piece>) -> u8 {
            match pc {
                None         => 0,
                Some(Knight) => 1,
                Some(Bishop) => 2,
                Some(Rook)   => 3,
                Some(Queen)  => 4,
                _            => panic!("PackedMove: bad promotion: {:?}", pc),
            }
        }

        pub fn convert_to_piece(pc: u8) -> Option<Piece> {
            match pc {
                0 => None,
                1 => Some(Knight),
                2 => Some(Bishop),
                3 => Some(Rook),
                4 => Some(Queen),
                _ => unimplemented!(),
            }
        }

    }

}

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
        num_threads:     u8,
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
        builder_field!(num_threads, u8);
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
                eprintln!("{:>6} sfens, {:>6} moves, {:.1}s, avg {:.1} sfens / sec, {:.1} moves / sec",
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
            ts:         &Tables,
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
                s.spawn(|_| Self::save_listener(save_bin, path, rx.clone()));

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
            }).unwrap();

            Ok(())
        }

        fn save_listener<P: AsRef<Path>>(
            save_bin:   bool,
            path:       P,
            rx:         Receiver<TrainingData>,
        ) {
            let mut out = vec![];
            loop {
                match rx.recv() {
                    Ok(td) => {
                        out.push(td);
                        TrainingData::save_all(save_bin, &path, &out).unwrap();
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
            ts:         &Tables,
            ob:         &OpeningBook,
            count:      u64,
            mut rng:    StdRng,
            sfen_n:     Arc<(AtomicU64,AtomicU64)>,
            tx:         Sender<TrainingData>,
        ) {

            let opening_ply = 16;

            let mut g = Game::from_fen(ts, STARTPOS).unwrap();
            let timesettings = TimeSettings::new_f64(0.0, self.time);
            let mut ex = Explorer::new(g.state.side_to_move, g.clone(), self.max_depth, timesettings);
            ex.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();

            ex.cfg.num_threads = Some(1);
            ex.cfg.clear_table = false;

            let mut ex_white = ex.clone();
            let mut ex_black = ex.clone();
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
                sfen_n.0.fetch_add(n as u64, Ordering::SeqCst);

                if let Some(result) = result {
                    let opening = opening.iter().map(|&mv| PackedMove::convert(mv)).collect();
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

                let n_fens  = sfen_n.0.load(Ordering::SeqCst);
                if n_fens > count { break 'outer; }
                let n_moves = sfen_n.1.load(Ordering::SeqCst);
                if let Some(n) = self.num_positions {
                    if n_moves > n { break 'outer; }
                }

                // eprintln!("_do_explore: breaking outer loop");
                // break 'outer;
            }
        }

        pub fn _do_explore2(
            &self,
            ts:         &Tables,
            ob:         &OpeningBook,
            count:      usize,
            mut rng:    StdRng,
            sfen_n:     Arc<(AtomicU64,AtomicU64)>,
            tx:         Sender<TrainingData>,
        ) {

            let ex_min_nodes = 5000;
            let ex_max_nodes = 15000;

            let ex_max_ply = 200;

            let opening_ply = 16;

            let mut g = Game::from_fen(ts, STARTPOS).unwrap();

            let timesettings = TimeSettings::new_f64(0.0, self.time);
            let mut ex = Explorer::new(g.state.side_to_move, g.clone(), self.max_depth, timesettings);

            ex.cfg.num_threads = Some(1);
            ex.cfg.clear_table = false;

            // let mut s = if let Some(o) = self.opening { o } else { OBSelection::BestN(0) };
            let mut s = OBSelection::Random(rng);

            let mut n_games = 0;

            'outer: for gk in 0..count {
                // eprintln!("starting do_explore #{}", gk);

                let (mut g,opening) = ob.start_game(&ts, Some(opening_ply), &mut s).unwrap();

                // let fen = "7k/8/8/8/8/8/4Q3/7K w - - 0 1"; // Queen endgame, #7
                // let opening = vec![];
                // let mut g = Game::from_fen(ts, fen).unwrap();

                // eprintln!("g.to_fen() = {:?}", g.to_fen());
                // eprintln!("g 0 = {:?}", g);
                // for mv in opening.iter() {
                //     eprintln!("    mv = {:?}", mv);
                // }

                ex.update_game(g.clone());
                ex.clear_tt();
                let mut ply = opening.len();

                'game: loop {

                    if let Some(k) = self.num_positions {
                        if sfen_n.0.load(Ordering::Relaxed) > k {
                            return;
                        }
                    }

                    let mut best                       = None;
                    let mut last_res: Option<ABResult> = None;
                    let mut moves: Vec<TDEntry>        = vec![];

                    let t0 = std::time::Instant::now();
                    for depth in 1..self.max_depth {

                        // search, with multiPV: self.max_depth, nodes

                        let mut stats = SearchStats::default();
                        let (res,moves,stats) = ex.lazy_smp_2(ts);

                        // match res {
                        //     ABResults::ABList(res, _) => res,
                        //     ABResults::ABSingle(res)  => res,
                        //     ABResults::ABSyzygy(res)  => res,
                        //     _                         => {
                        //         println!("game ended, res = {:?}", res);
                        //         // panic!("game ended, res = {:?}", res);
                        //     },
                        // }

                        match res.get_result() {
                            Some(res) => {
                                last_res = Some(res.clone());
                                best = Some((res.mv, res.score));
                            },
                            None => {},
                        }

                        // // if let Some(mv) = res.moves.get(0) {
                        // if let Some(mv) = res.mv {
                        //     best = Some((mv, res.score));
                        // }
                        // unimplemented!()

                    }

                    if let Some((mv,score)) = best.take() {
                        g = g.make_move_unchecked(ts, mv).unwrap();
                        trace!("fen = {:?}", g.to_fen());
                        trace!("mv = {:?}", mv);
                        trace!("did move in {:.3} seconds", t0.elapsed().as_secs_f64());

                        // eprintln!("g = {:?}", g);

                        ply += 1;
                        // trace!("ply = {:?}", ply);

                        ex.game = g.clone();
                        ex.side = g.state.side_to_move;

                        let skip = false;

                        if !skip {
                            sfen_n.1.fetch_add(1, Ordering::SeqCst);
                        }

                        let e = TDEntry::new(mv, score, skip);
                        moves.push(e);
                    } else {

                        match last_res {
                            Some(score) => {

                                let result = if score.score > CHECKMATE_VALUE - 100 {
                                    TDOutcome::Win(!g.state.side_to_move)
                                } else if score.score.abs() > CHECKMATE_VALUE - 100 {
                                    TDOutcome::Win(g.state.side_to_move)
                                } else {
                                    println!("wat");
                                    panic!();
                                };

                                debug!("Finished game: {:?}", result);
                                let k = sfen_n.0.load(Ordering::SeqCst);
                                // eprintln!("sfen_n = {:?}", k);

                                let n = moves.iter().filter(|x| !x.skip).count();
                                sfen_n.0.fetch_add(n as u64, Ordering::SeqCst);

                                let opening = opening.iter().map(|&mv| PackedMove::convert(mv)).collect();
                                let mut td = TrainingData {
                                    result,
                                    opening,
                                    moves,
                                };
                                // out.push(td);
                                // TrainingData::save_all(&path, &out)?;

                                n_games += 1;
                                if n_games >= count {
                                    break 'outer;
                                } else {
                                    break 'game;
                                }
                            }
                            None                          => unimplemented!(),
                        }

                        // eprintln!("g        = {:?}", g);
                        // eprintln!("last_res = {:?}", last_res);
                        // panic!("game ended maybe?");
                    }

                }

            }

            // Ok(out)
        }


        // pub fn generate_single(&self, ts: &Tables) -> Option<TrainingData> {

        //     let mut g = Game::from_fen(ts, STARTPOS).unwrap();
        //     // let mut moves = vec![];

        //     // for &mv in self.opening.iter() {
        //     //     g = g.clone().make_move_unchecked(ts, mv).unwrap();
        //     //     // g.make_move(ts, mv);
        //     // }

        //     // // let fen = "6k1/4Q3/8/8/8/5K2/8/8 w - - 6 4"; // Queen endgame, #4
        //     // // let fen = "7k/4Q3/8/4K3/8/8/8/8 w - - 8 5"; // Queen endgame, #2
        //     // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // position 2
        //     // // let fen = "r4rk1/4npp1/1p1q2b1/1B2p3/1B1P2Q1/P3P3/5PP1/R3K2R b KQ - 1 1"; // Q cap d6b4
        //     // let mut g = Game::from_fen(ts, fen).unwrap();
        //     // eprintln!("g = {:?}", g);

        //     // let mut g = g.flip_sides(ts);

        //     let mut max_depth = self.max_depth;
        //     let mut t = self.time;

        //     let timesettings = TimeSettings::new_f64(0.0,t);
        //     let mut ex = Explorer::new(g.state.side_to_move, g.clone(), max_depth, timesettings);
        //     ex.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();

        //     let mut prevs: HashMap<Zobrist, u8> = HashMap::default();

        //     // let mut prev_idx = None;
        //     // let mut tree = TDTree::empty();
        //     let mut moves = vec![];

        //     debug!("generate_single starting...");
        //     let result = loop {
        //         ex.blocked_moves.clear();

        //         if let (Some((mv,score)),stats) = ex.explore(&ts, None) {
        //             g = g.make_move_unchecked(ts, mv).unwrap();

        //             if self.print {
        //                 eprintln!("{:?}\n{:?}\n{:?}", mv, g, g.to_fen());
        //                 eprintln!("score.score = {:?}", score.score);
        //             }

        //             if score.score > CHECKMATE_VALUE - 100 {
        //                 break TDOutcome::Win(!g.state.side_to_move);
        //             } else if score.score.abs() > CHECKMATE_VALUE - 100 {
        //                 // break GameEnd::Checkmate { win: g.state.side_to_move };
        //                 break TDOutcome::Win(g.state.side_to_move);
        //             }

        //             ex.game = g.clone();
        //             ex.side = g.state.side_to_move;

        //             let e = TDEntry::new(mv, score.score);
        //             // prev_idx = Some(tree.insert(prev_idx, e));
        //             moves.push(e);

        //         } else {
        //             panic!("wat");
        //             // break GameEnd::Error;
        //         }

        //     };

        //     Some(TrainingData {
        //         result,
        //         opening: self.opening.iter().map(|&mv| PackedMove::convert(mv)).collect(),
        //         // moves: tree,
        //         moves,
        //     })

        // }

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
    pub opening:      Vec<PackedMove>,
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

#[derive(Debug,Eq,PartialEq,Clone,Copy,Serialize,Deserialize)]
pub struct TDEntry {
    mv:       PackedMove,
    // eval:     i8,
    eval:     Score,
    skip:     bool,
}

impl TDEntry {
    pub fn new(mv: Move, eval: Score, skip: bool) -> Self {
        Self {
            mv: PackedMove::convert(mv),
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

    pub fn load_all<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<Self>> {
        let mut b = std::fs::read(path)?;
        let out: Vec<Self> = bincode::deserialize(&b).unwrap();
        Ok(out)
    }

    pub fn save_all<P: AsRef<Path>>(save_bin: bool, path: P, xs: &Vec<Self>) -> std::io::Result<()> {
        use std::io::Write;
        // let mut buf: Vec<u8> = vec![];

        let mut file = std::fs::File::create(path)?;

        if save_bin {
            let buf: Vec<u8> = bincode::serialize(&xs).unwrap();
            file.write_all(&buf)
        } else {
            // file.write_all(&buf)
            unimplemented!()
        }

        // for x in xs.iter() {
        //     let b: Vec<u8> = bincode::serialize(&x).unwrap();
        //     buf.extend(b.into_iter());
        // }

    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        use std::io::Write;
        let b: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut file = std::fs::File::create(path)?;
        file.write_all(&b)
    }

    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        use std::io::Write;
        let mut b = std::fs::read(path)?;
        let out: Self = bincode::deserialize(&b).unwrap();
        Ok(out)
    }

}





