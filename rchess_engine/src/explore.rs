
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
pub use crate::timer::*;
pub use crate::trans_table::*;
pub use crate::searchstats::*;

use std::hash::Hasher;
use std::iter::successors;
use std::sync::atomic::AtomicI32;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
// use std::sync::RwLock;
use parking_lot::RwLock;

use rayon::Scope;
use rayon::prelude::*;

use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct Explorer {
    pub side:          Color,
    pub game:          Game,
    // pub stats:      ABStats
    pub max_depth:     Depth,
    pub timer:         Timer,
    pub parallel:      bool,
    pub trans_table:   RwTransTable,
    // pub trans_table:   TransTable,
    // pub trans_table:   RwLock<TransTable>,
}

impl Explorer {
    pub fn new(side:          Color,
               game:          Game,
               max_depth:     Depth,
               should_stop:   Arc<AtomicBool>,
               settings:      TimeSettings,
    ) -> Self {
        let tt = RwTransTable::default();
        // let tt = TransTable::new();
        Self {
            side,
            game,
            // stats: ABStats::new(),
            max_depth,
            timer: Timer::new(side, should_stop, settings),
            parallel: true,
            // trans_table: TransTable::default(),
            trans_table: tt,
        }
    }
}

/// Entry points
impl Explorer {

    pub fn explore(&self, ts: &Tables, depth: Option<Depth>) -> (Option<Move>,SearchStats) {

        let (moves,stats) = self.iterative_deepening(&ts, false);
        (moves.get(0).map(|x| x.0),stats)

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

    // pub fn rank_moves_list(&self, ts: &Tables, print: bool, moves: Vec<Move>
    // ) -> (Vec<(Move,Score)>,SearchStats) {

    //     self.trans_table.clear();

    //     if print {
    //         eprintln!("moves.len() = {:?}", moves.len());
    //     }

    //     // eprintln!("Explorer: not parallel");
    //     // let (mut out,ss): (Vec<(Move,i32)>,Vec<SearchStats>) = moves.into_iter()
    //     let (mut out,ss): (Vec<(Move,i32)>,Vec<SearchStats>) = moves.into_par_iter()
    //             .map(|mv| {
    //                 let g2 = self.game.make_move_unchecked(&ts, &mv).unwrap();
    //                 // let alpha = i32::MIN;
    //                 // let beta  = i32::MAX;
    //                 let alpha = Arc::new(AtomicI32::new(i32::MIN));
    //                 let beta = Arc::new(AtomicI32::new(i32::MAX));
    //                 let mut stats = SearchStats::default();
    //                 let score = self._ab_search(
    //                     &ts, &g2, self.max_depth, 1, alpha, beta, false, &mut stats);
    //                 ((mv,score),stats)
    //             })
    //             // .collect();
    //             .unzip();

    //     let stats = ss.iter().sum();

    //     out.sort_by(|a,b| a.1.cmp(&b.1));

    //     if self.side == self.game.state.side_to_move {
    //         out.reverse();
    //     }

    //     if print {
    //         for (m,s) in out.iter() {
    //             eprintln!("{:>8} = {:?}", s, m);
    //         }
    //     }
    //     (out,stats)
    // }

    pub fn iterative_deepening(&self, ts: &Tables, print: bool) -> (Vec<(Move,Score)>,SearchStats) {

        let ms = self.game.search_all(&ts, None);
        let ms = ms.get_moves_unsafe();
        let (ms,stats) = self._iterative_deepening(&ts, print, ms);

        // (ms.get(0).map(|x| x.0),stats)
        // (ms.get(0).copied(),stats)
        (ms,stats)
    }

    pub fn _iterative_deepening(&self, ts: &Tables, print: bool, moves: Vec<Move>
    ) -> (Vec<(Move,Score)>,SearchStats) {

        self.trans_table.clear();

        let mut timer = self.timer.clone();
        timer.reset();

        let mut out: Vec<(Move,i32)> = vec![];
        let mut ss:  Vec<SearchStats>;
        let mut stats = SearchStats::default();
        let mut depth = 0;

        let gs: Vec<(Move,Game)> = moves.par_iter().flat_map(|mv| {
        // let gs: Vec<(Move,Game)> = moves.iter().flat_map(|mv| {
            if let Ok(g2) = self.game.make_move_unchecked(&ts, &mv) {
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

        while timer.should_search(self.side, depth) && (depth <= self.max_depth) {

            // eprintln!("Explorer: not parallel");
            // (out,ss) = gs.iter().map(|(mv,g2)| {
            (out,ss) = gs.par_iter().map(|(mv,g2)| {
                let alpha = Arc::new(AtomicI32::new(i32::MIN));
                let beta  = Arc::new(AtomicI32::new(i32::MAX));
                // let alpha = i32::MIN;
                // let beta  = i32::MAX;
                let mut stats = SearchStats::default();
                let score = self._ab_search(
                    // &ts, &g2, depth, 1, alpha.clone(), beta.clone(), false, &mut stats);
                    &ts, &g2, depth, 1, alpha, beta, false, &mut stats);
                ((*mv,score),stats)
                })
                .unzip();

            stats = ss.iter().sum();

            out.sort_by(|a,b| a.1.cmp(&b.1));

            if self.side == self.game.state.side_to_move {
                out.reverse();
            }

            depth += 1;
            timer.update_times(self.side, stats.nodes);
            if print {
                eprintln!("depth, time = {:?}, {:.2}", depth-1, timer.time_left[self.side]);
            }
        }
        if print {
            print!("\n");
            for (m,s) in out.iter() {
                eprintln!("{:>8} = {:?}", s, m);
            }
        }
        (out,stats)
    }

}

impl Explorer {

}

/// AB search
impl Explorer {

    fn check_tt(&self,
                ts:             &Tables,
                g:              &Game,
                depth:          Depth,
                // k:              i32,
                // alpha:          i32,
                // beta:           i32,
                maximizing:     bool,
                mut stats:      &mut SearchStats,
    ) -> Option<Score> {
        if let Some(si) = self.trans_table.get(&g.zobrist) {

            if si.depth_searched == depth {
                stats.tt_hits += 1;
                Some(si.score)
            } else {
                stats.tt_misses += 1;
                None
            }
        } else {
            stats.tt_misses += 1;
            // let score = self._ab_search(&ts, &g, depth - 1, k + 1, alpha, beta, !maximizing);
            // score
            None
        }
    }

    pub fn _ab_search(
        &self,
        ts:                 &Tables,
        g:                  &Game,
        depth:              Depth,
        k:                  i16,
        // mut alpha:          i32,
        // mut beta:           i32,
        alpha:              Arc<AtomicI32>,
        beta:               Arc<AtomicI32>,
        maximizing:         bool,
        mut stats:          &mut SearchStats,
    ) -> Score {

        let moves = g.search_all(&ts, None);

        let moves: Vec<Move> = match moves {
            Outcome::Checkmate(c) => {
                let score = 100_000_000 - k as Score;
                stats.leaves += 1;
                stats.checkmates += 1;
                return score;
            },
            Outcome::Stalemate    => {
                let score = -100_000_000 + k as Score;
                stats.leaves += 1;
                stats.stalemates += 1;
                return score;
            },
            Outcome::Moves(ms)    => ms,
        };

        if depth == 0 {
            let score = g.evaluate(&ts).sum();
            // let score = self.quiescence(&ts, &g, moves, k, alpha, beta);

            stats.leaves += 1;
            if self.side == Black {
                return -score;
            } else {
                return score;
            }
        }

        let gs = moves.into_iter().flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, &m) {
        // let gs = moves.into_par_iter().flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, &m) {
            let tt = self.check_tt(&ts, &g2, depth, maximizing, &mut stats);
            Some((m,g2,tt))
        } else {
            None
        });

        let mut gs: Vec<(Move,Game,Option<Score>)> = gs.collect();
        gs.sort_unstable_by(|a,b| a.2.partial_cmp(&b.2).unwrap());
        if !maximizing {
            gs.reverse();
        }

        use crossbeam_channel::unbounded;
        let (chan_tx,chan_rx) = unbounded();
        // let (chan_tx,chan_rx) = std::sync::mpsc::sync_channel(4);

        let mut i_max = gs.len() as i16;
        // eprintln!("i_max = {:?}", i_max);
        // let (gs2,ss): (Vec<(Move,Game,Option<Score>,Score)>,Vec<SearchStats>) = gs.into_par_iter()
        // let (gs2,ss): (Vec<(Move,Game,Option<Score>,Score)>,Vec<SearchStats>) = gs.into_iter()
        let gs2 = gs.into_par_iter()
        // let gs2 = gs.into_iter()
            .enumerate()
            .for_each(|(i,(m,g2,tt))| {
                // println!("wat 0: {}", i);
                let mut ss = SearchStats::default();
                // let score = self._ab_search(&ts, &g2, depth - 1, k + 1, alpha, beta, !maximizing, &mut stats);
                // let score = self._ab_search(&ts, &g2, depth - 1, k + 1, alpha, beta, !maximizing, &mut ss);
                let alpha2 = Arc::new(AtomicI32::new(alpha.load(Ordering::Relaxed)));
                let beta2  = Arc::new(AtomicI32::new(beta.load(Ordering::Relaxed)));
                let score = self._ab_search(&ts, &g2, depth - 1, k + 1, alpha2, beta2, !maximizing, &mut ss);
                // let score = self.check_tt(&ts, &g2, depth, k, alpha, beta, maximizing);
                // let score = match tt {
                //     None    => self._ab_search(&ts, &g2, depth - 1, k + 1, alpha, beta, !maximizing),
                //     Some(s) => s,
                // };
                chan_tx.send((m,g2,tt,score,ss)).unwrap();
            });
        // *stats += ss.into_iter().sum();

        // let gs2 = gs.into_iter()
        //     .map(|(m,g2,tt)| {
        //         let mut ss = SearchStats::default();
        //         let score = self._ab_search(&ts, &g2, depth - 1, k + 1, alpha, beta, !maximizing, &mut ss);
        //         (m,g2,tt,score)
        //     });

        let mut val = if maximizing { i32::MIN } else { i32::MAX };
        let mut val: (Option<(Zobrist,Move)>,i32) = (None,val);

        let mut node_type = Node::PV;

        // for (mv,g2,tt) in gs.into_iter() {
        // for (mv,g2,tt,score) in gs2.into_iter() {
        // for (mv,g2,tt,score,ss) in gs2 {
        // for (mv,g2,tt,score) in gs2 {
        loop {

            if i_max == 0 { break; } else { i_max -= 1; }
            let (mv,g2,tt,score,ss) = match chan_rx.recv() {
                Ok(x)  => x,
                Err(_) => break,
            };

            let zb = g2.zobrist;

            // *stats += ss;
            stats.nodes += 1;

            let alpha2 = Arc::new(AtomicI32::new(alpha.load(Ordering::Relaxed)));
            let beta2  = Arc::new(AtomicI32::new(beta.load(Ordering::Relaxed)));
            let score = self._ab_search(
                // &ts, &g2, depth - 1, k + 1, alpha.clone(), beta.clone(), !maximizing, &mut stats);
                &ts, &g2, depth - 1, k + 1, alpha2, beta2, !maximizing, &mut stats);
                // &ts, &g2, depth - 1, k + 1, alpha, beta, !maximizing, &mut stats);

            // if maximizing {
            //     val.1 = i32::max(val.1, score);
            // } else {
            //     val.1 = i32::min(val.1, score);
            // }

            if maximizing {
                if score > val.1 {
                    val = (Some((zb,mv)),score);
                }
                alpha.fetch_max(val.1, Ordering::SeqCst);
                if val.1 >= beta.load(Ordering::SeqCst) { // Beta cutoff
                    // stats.
                    // self.trans_table.insert_replace(
                    //     zb, SearchInfo::new(mv, depth, Node::LowerBound, val.1));
                    break;
                }
            } else {
                if score < val.1 {
                    val = (Some((zb,mv)),score);
                }
                beta.fetch_min(val.1, Ordering::SeqCst);
                if val.1 <= alpha.load(Ordering::SeqCst) { // Alpha cutoff
                    // self.trans_table.insert_replace(
                    //     zb, SearchInfo::new(mv, depth, Node::UpperBound, val.1));
                    break;
                }
            }

            // if maximizing {
            //     if score > val.1 {
            //         val = (Some((zb,mv)),score);
            //     }
            //     if val.1 > alpha {
            //         alpha = val.1;
            //         // self.trans_table.insert_replace(
            //         //     zb, SearchInfo::new(mv, depth, val.1));
            //     }
            //     if val.1 >= beta { // Beta cutoff
            //         // self.trans_table.insert(zb, SearchInfo::new(depth, NodeType::NodeLowerBound(beta)));
            //         self.trans_table.insert_replace(
            //             zb, SearchInfo::new(mv, depth, Node::LowerBound(val.1)));
            //         break;
            //     }
            // } else {
            //     if score < val.1 {
            //         val = (Some((zb,mv)),score);
            //     }
            //     if val.1 < beta {
            //         beta = val.1;
            //         // self.trans_table.insert_replace(
            //         //     zb, SearchInfo::new(mv, depth, val.1));
            //     }
            //     if val.1 <= alpha { // Alpha cutoff
            //         self.trans_table.insert_replace(
            //             zb, SearchInfo::new(mv, depth, Node::UpperBound(val.1)));
            //         break;
            //     }
            // }

            /*
            if maximizing {
                val = i32::max(val, score);
                alpha = i32::max(val, alpha); // fail soft
                if val >= beta {
                    // maximum score that the minimizing player is assured of
                    break;
                }
                // alpha = i32::max(val, alpha); // fail hard
            } else {
                val = i32::min(val, score);
                beta = i32::min(val, beta); // fail soft
                if val <= alpha {
                    // score for this node is less than the least bad move for self
                    // the minimum score that the maximizing player is assured of
                    break
                }
                // beta = i32::min(val, beta); // fail hard
            }
            */

        }

        match node_type {
            Node::PV         => {
                if let Some((zb,mv)) = val.0 {
                    self.trans_table.insert_replace(
                        zb, SearchInfo::new(mv, depth, Node::PV, val.1));
                }
            },
            Node::UpperBound => {},
            Node::LowerBound => {},
        }

        // stats.alpha = stats.alpha.max(alpha);
        // stats.beta = stats.beta.max(beta);
        stats.alpha = stats.alpha.max(alpha.load(Ordering::SeqCst));
        stats.beta = stats.beta.max(beta.load(Ordering::SeqCst));

        val.1
        // val
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

    pub fn quiescence(&self, ts: &Tables, g: &Game, ms: Vec<Move>, k: i16, mut alpha: i32, mut beta: i32) -> i32 {
        println!("quiescence {}", k);

        let stand_pat = g.evaluate(&ts).sum();
        // return stand_pat;

        if stand_pat >= beta {
            // return score; // fail soft
            return beta; // fail hard
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let captures = ms.into_iter().filter(|m| m.filter_all_captures()).collect::<Vec<_>>();

        for m in captures.into_iter() {
            if let Ok(g2) = g.make_move_unchecked(&ts, &m) {
                if let Outcome::Moves(ms2) = g2.search_all(&ts, None) {
                    let score = -self.quiescence(&ts, &g2, ms2, k + 1, alpha, beta);

                    if score >= beta {
                        return beta;
                    }
                    if score > alpha {
                        alpha = score;
                    }
                }
            }
        }

        alpha
    }

    pub fn quiescence2(&self, ts: &Tables, g: &Game, ms: Vec<Move>, k: i16, mut alpha: i32, mut beta: i32
    ) -> i32 {
        // println!("quiescence");

        let score = g.evaluate(&ts).sum();
        if score >= beta {
            // return score; // fail soft
            return beta; // fail hard
        }
        if alpha < score {
            alpha = score;
        }

        let ms = ms.into_iter().filter(|m| m.filter_all_captures()).collect::<Vec<_>>();

        if ms.len() == 0 {
            println!("wat 0");
            return alpha;
        }

        let ms = ms.into_par_iter()
            .flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, &m) {
                if let Outcome::Moves(ms2) = g2.search_all(&ts, None) {
                    let score = -self.quiescence2(&ts, &g2, ms2, k + 1, -alpha, -beta);
                    Some((m,g2,score))
                } else {
                    None
                }
            } else { None }).collect::<Vec<_>>();

        for (m,g2,score) in ms.into_iter() {
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        println!("wat 1");
        alpha
    }

    // fn quiescence2(&self,
    //               ts: &Tables,
    //               g: Game,
    //               moves: Option<Outcome>,
    //               k: i32,
    //               mut alpha: i32,
    //               mut beta: i32
    // ) -> i32 {
    //     // println!("quiescence 0");
    //     let stand_pat = g.evaluate(&ts).sum();

    //     if stand_pat >= beta {
    //         return beta;
    //     }
    //     if alpha < stand_pat {
    //         alpha = stand_pat;
    //     }

    //     let moves = match moves {
    //         Some(ms) => ms,
    //         None     => g.search_all(&ts, None),
    //     };
    //     match moves {
    //         Outcome::Checkmate(c) => return 100_000_000 - k,
    //         Outcome::Stalemate    => return -100_000_000 + k,
    //         Outcome::Moves(_)     => {},
    //     }

    //     let moves = moves.into_iter().filter(|m| m.filter_all_captures());

    //     for m in moves {
    //         if let Ok(g2) = g.make_move_unchecked(&ts, &m) {
    //             let score = -self.quiescence2(&ts, g2, None, k+1, -alpha, -beta);

    //             if score >= beta {
    //                 return beta;
    //             }
    //             if score > alpha {
    //                 alpha = score;
    //             }
    //         }
    //     }

    //     alpha
    //     // unimplemented!()
    // }

}

/// Pruning
impl Explorer {

    pub fn prune_delta(&self, ts: &Tables, g: Game) -> bool {
        unimplemented!()
    }


}

/// Misc
impl Explorer {

    pub fn order_moves(&self, ts: &Tables, mut moves: &mut [(Move,Score,Game)]) {
        use std::cmp::Ordering;
        moves.sort_unstable_by(|a,b| {
            let a_cap = a.0.filter_all_captures();
            let b_cap = b.0.filter_all_captures();
            let a_castle = a.0.filter_castle();
            let b_castle = b.0.filter_castle();
            if a_cap & !b_cap {
                Ordering::Greater
            } else if !a_cap & b_cap {
                Ordering::Less
            } else if a_castle & !b_castle {
                Ordering::Greater
            } else if !a_castle & b_castle {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
    }

}

/// Old
impl Explorer {

    pub fn _ab_search2(
        &self,
        ts:                 &Tables,
        trans_table:        &RwLock<TransTable>,
        g:                  Game,
        depth:              u32,
        k:                  i32,
        m0:                 Option<Move>,
        mut alpha:          (Option<Move>,i32),
        mut beta:           (Option<Move>,i32),
    ) -> (Option<Move>,i32) {
        // eprintln!("m0 = {:?}", m0);

        let maximizing = self.side == g.state.side_to_move;

        let mut val = if maximizing {
            i32::MIN
        } else {
            i32::MAX
        };

        let moves = g.search_all(&ts, None);

        match moves {
            Outcome::Checkmate(c) => return (m0, 100_000_000 - k),
            Outcome::Stalemate    => return (m0, -100_000_000 + k),
            Outcome::Moves(_)     => {},
        }

        if depth == 0 {
            // println!("wat 0");
            let score = g.evaluate(&ts).sum();
            let score = if !maximizing { -score } else { score };
            // trans_table.write().insert(g.state, SearchInfo::new(m0.unwrap(), k as u32, s));
            return (m0,score);
            // if maximizing {
            //     return (m0,self.quiescence(&ts, g, Some(moves), k, alpha.1, beta.1));
            // } else {
            //     return (m0,-self.quiescence(&ts, g, Some(moves), k, alpha.1, beta.1));
            // }
            // return (m0,self.quiescence(&ts, g, Some(moves), k, alpha.1, beta.1));
        }

        let mut moves = moves.get_moves_unsafe();
        // let ms = moves.clone();

        let mut gs = moves.into_iter().flat_map(|m| {
        // let mut gs = moves.iter_mut().flat_map(|m| {
            match g.make_move_unchecked(&ts, &m) {
                Ok(g2) => {
                    // let score = g2.evaluate(&ts);
                    // let score = score.sum(self.side);
                    // let score = if maximizing { -score } else { score };
                    let score = 0;

                    // if let Some(prev) = trans_table.read().get(&g2.state) {
                    //     Some((m,score,Some(*prev),g2))
                    // } else {
                    //     Some((m,score,None,g2))
                    // }

                    Some((m,score,g2))

                },
                Err(end) => {
                    panic!()
                },
            }
        })
            // .collect::<Vec<(Move,Option<SearchInfo>, Game)>>();
            // .collect::<Vec<(Move, Score, Option<SearchInfo>, Game)>>();
            // .collect::<Vec<(Move, Score, Game)>>();
            ;

        let gs0 = gs.clone().filter(|(m,p,g2)| m.filter_all_captures());
        let gs1 = gs.filter(|(m,p,g2)| !m.filter_all_captures());
        let gs = gs0.chain(gs1);

        // let gs2 = gs1.clone().filter(|(m,p,g2)| !m.filter_castle());
        // let gs1 = gs1.filter(|(m,p,g2)| m.filter_castle());
        // let gs = gs0.chain(gs1).chain(gs2);

        // gs.sort_unstable_by(|a,b| {
        //     a.1.partial_cmp(&b.1).unwrap()
        //     // a.1.cmp(&b.1)
        // });

        // self.order_moves(&ts, &mut gs);

        // if maximizing {
        //     gs.reverse();
        // }

        // println!("wat 0");
        // for (m,score,p,g2) in gs {
        for (m,score,g2) in gs {
            // eprintln!("score = {:?}", score);

            // match p {
            //     Some(prev) => {
            //         if prev.depth_searched > (self.depth / 2) {
            //             return (m0,prev.score);
            //         }
            //     },
            //     None => {},
            // }

            // let score = g2.evaluate(&ts);
            // let score = score.sum(self.side);

            let a = (alpha.0,-alpha.1);
            let b = (beta.0,-beta.1);

            let score = -self._ab_search2(
                &ts, &trans_table, g2, depth - 1, k + 1, m0, a, b).1;

            if score >= beta.1 {
                return beta;
            }
            if score > alpha.1 {
                alpha.1 = score;
            }

            // if maximizing {
            // }

            // if maximizing {
            //     // maximize self
            //     let val2 = self._ab_search(
            //         // &ts, &trans_table, g2, depth - 1, k + 1, Some(m), alpha, beta);
            //         &ts, &trans_table, g2, depth - 1, k + 1, m0, alpha, beta);
            //     val = val.max(val2.1);

            //     if val >= beta.1 {
            //         return beta;
            //     }

            //     if alpha.0.is_none() {
            //         alpha = (m0, val);
            //     } else if val > alpha.1 {
            //         alpha = (m0, val);
            //     }

            // } else {
            //     // minimize other
            //     let val2 = self._ab_search(
            //         // &ts, &trans_table, g2, depth - 1, k + 1, Some(m), alpha, beta);
            //         &ts, &trans_table, g2, depth - 1, k + 1, m0, alpha, beta);
            //     val = val.min(val2.1);

            //     if val <= alpha.1 {
            //         return alpha;
            //     }

            //     if beta.0.is_none() {
            //         beta = (m0, val);
            //     } else if val < beta.1 {
            //         beta = (m0, val);
            //     }
            // }

        }

        alpha


        // for m in moves.into_iter() {
        //     match g.make_move_unchecked(&ts, &m) {
        //         Ok(g2)   => {
        //             let score = g2.evaluate(&ts);
        //             let score = score.sum(self.side);

        //             if maximizing {
        //                 // maximize self
        //                 let val2 = self._ab_search(
        //                     &ts, &trans_table, g2, depth - 1, k + 1, Some(m), alpha, beta);
        //                 val = val.max(val2.1);

        //                 if val >= beta.1 {
        //                     return beta;
        //                 }

        //                 if alpha.0.is_none() {
        //                     alpha = (Some(m), val);
        //                 } else if val > alpha.1 {
        //                     alpha = (Some(m), val);
        //                 }

        //             } else {
        //                 // minimize other
        //                 let val2 = self._ab_search(
        //                     &ts, &trans_table, g2, depth - 1, k + 1, Some(m), alpha, beta);
        //                 val = val.min(val2.1);

        //                 if val <= alpha.1 {
        //                     return alpha;
        //                 }

        //                 if beta.0.is_none() {
        //                     beta = (Some(m), val);
        //                 } else if val < beta.1 {
        //                     beta = (Some(m), val);
        //                 }
        //             }
        //         },
        //         Err(end) => {
        //             panic!()
        //         },
        //     }
        // }

        // if maximizing {
        //     alpha
        // } else {
        //     beta
        // }

    }

}

