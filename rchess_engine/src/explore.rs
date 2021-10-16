
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
pub use crate::timer::*;
pub use crate::trans_table::*;

use std::time::Duration;

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
    pub depth:         u32,
    // pub should_stop:   
    pub timer:         Timer,
    pub parallel:      bool,
    // pub trans_table:   TransTable,
    pub trans_table:   RwLock<TransTable>,
}

impl Explorer {
    pub fn new(side:          Color,
               game:          Game,
               depth:         u32,
               should_stop:   Arc<AtomicBool>,
               settings:      TimeSettings,
    ) -> Self {
        let tt = RwLock::new(TransTable::default());
        Self {
            side,
            game,
            // stats: ABStats::new(),
            depth,
            timer: Timer::new(should_stop, settings),
            parallel: true,
            // trans_table: TransTable::default(),
            trans_table: tt,
        }
    }
}

// #[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
// pub struct ABStats {
//     // /// Min score for Self
//     // pub alpha:      Option<(Eval, i32)>,
//     // /// Max score for Other
//     // pub beta:       Option<(Eval, i32)>,
//     /// Min score for Self
//     pub alpha:      i32,
//     /// Max score for Other
//     pub beta:       i32,
//     pub leaves:     u64,
// }

// impl ABStats {
//     pub fn new() -> Self {
//         Self {
//             alpha:      i32::MIN,
//             beta:       i32::MAX,
//             // alpha:      None,
//             // beta:       None,
//             leaves:     0,
//         }
//     }
// }

/// Entry points
impl Explorer {

    pub fn explore(&self, ts: &Tables, depth: u32) -> Option<Move> {

        // let e = self.game.evaluate(&ts, self.side);
        // eprintln!("e = {:?}", e.diff());

        // let mut ms = self.negamax(&ts, depth, self.side);

        // let ms = self.ab_search(&ts, depth, None);
        let mut moves = self.rank_moves(&ts, false, true);

        // eprintln!("ms = {:?}", ms);

        moves.get(0).map(|x| x.0)

        // ms.map(|x| x.0).flatten()
        // ms.0
        // unimplemented!()

        // let ms = Eval::sort_rev(self.side, self.game.state.side_to_move, ms, |x| x.1);
        // for (m,e) in ms.into_iter() {
        //     eprintln!("0: {} = {:?}", e.score_material.diff(), m);
        // }

        // unimplemented!()
        // None
    }

    pub fn rank_moves_list(&self, ts: &Tables, print: bool, moves: Vec<Move>) -> Vec<(Move,i32)> {

        // let tt = RwLock::new(self.trans_table.clone());

        let mut out: Vec<(Move,i32)> = moves.into_par_iter()
        // let mut out: Vec<(Move,i32)> = moves.into_iter()
                .map(|m| {
                    let g2 = self.game.make_move_unchecked(&ts, &m).unwrap();
                    // let alpha = (None,i32::MIN);
                    // let beta  = (None,i32::MAX);
                    let alpha = i32::MIN;
                    let beta  = i32::MAX;
                    // let (_,score) = self._ab_search(&ts, &tt, g2, self.depth, 1, Some(m), alpha, beta);
                    let score = self._ab_search(
                        &ts, g2, self.depth, 1, alpha, beta, false);
                    (m,score)
                })
                .collect();

        out.sort_by(|a,b| a.1.cmp(&b.1));

        if self.side == self.game.state.side_to_move {
            out.reverse();
        }

        if print {
            for (m,s) in out.iter() {
                eprintln!("{:?}: {:?}", s, m);
            }
        }
        out
    }

    pub fn rank_moves(&self, ts: &Tables, print: bool, par: bool) -> Vec<(Move,i32)> {
        let moves = self.game.search_all(&ts, None);

        if moves.is_end() {
            return vec![];
        }
        let moves = moves.get_moves_unsafe();

        // self.rank_moves_list(&ts, print, moves, par)
        self.rank_moves_list(&ts, print, moves)
    }

}

/// AB search
impl Explorer {

    // pub fn ab_search(&self, ts: &Tables, m0: Move) -> i32 {
    //     let g = self.game.clone();

    //     let tt = RwLock::new(self.trans_table.clone());

    //     match g.make_move_unchecked(&ts, &m0) {
    //         Ok(g2) => {
    //             let alpha = (None,i32::MIN);
    //             let beta  = (None,i32::MAX);
    //             let (m2,score) = self._ab_search(&ts, &tt, g2, self.depth, 1, Some(m0), alpha, beta);
    //             score
    //         },
    //         Err(win) => {
    //             panic!("bad move? {:?}", &win);
    //         }
    //     }
    // }

    pub fn _ab_search(
        &self,
        ts:                 &Tables,
        // trans_table:        &RwLock<TransTable>,
        g:                  Game,
        depth:              u32,
        k:                  i32,
        mut alpha:          i32,
        mut beta:           i32,
        maximizing:         bool,
        // moves:              Option<Vec>,
    ) -> i32 {

        let moves = g.search_all(&ts, None);

        let moves: Vec<Move> = match moves {
            Outcome::Checkmate(c) => {
                let score = 100_000_000 - k;
                if self.side == Black { return -score; } else { return score; }
            },
            Outcome::Stalemate    => {
                let score = -100_000_000 + k;
                if self.side == Black { return -score; } else { return score; }
            },
            Outcome::Moves(ms)    => ms,
        };

        if depth == 0 {
            let score = g.evaluate(&ts).sum();
            // let score = self.quiescence(&ts, g, moves, k, alpha, beta);

            if self.side == Black {
                return -score;
            } else {
                return score;
            }
        }

        let gs = moves.into_iter().flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, &m) {
            Some((m,g2))
        } else {
            None
        });

        let mut val = if maximizing { i32::MIN } else { i32::MAX };
        // let mut val: (Option<Move>,i32) = (None,val);
        for (m,g2) in gs {
            let score = self._ab_search(&ts, g2, depth - 1, k + 1, alpha, beta, !maximizing);

            // if maximizing {
            //     if score > val.1 {
            //         val = (Some(m),score);
            //     }
            //     if val.1 > alpha {
            //         alpha = val.1;
            //     }
            //     if val.1 >= beta {
            //         break;
            //     }
            // } else {
            //     if score < val.1 {
            //         val = (Some(m),score);
            //     }
            //     if val.1 < beta {
            //         beta = val.1;
            //     }
            //     if val.1 <= alpha {
            //         break;
            //     }
            // }


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

        // let mut w = self.trans_table.write();
        // w.insert(g2.zobrist, SearchInfo::new(k, NodeType::NodePV()))

        // val.1
        val
    }

}

/// Quiescence
impl Explorer {

    fn quiescence(&self, ts: &Tables, g: Game, ms: Vec<Move>, k: i32, mut alpha: i32, mut beta: i32) -> i32 {

        let score = g.evaluate(&ts).sum();
        if score >= beta {
            return score; // fail soft
            // return beta; // fail hard
        }
        if alpha < score {
            alpha = score;
        }

        let ms = ms.into_iter()
            .filter(|m| m.filter_all_captures())
            .flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, &m) {
                Some((m,g2))
            } else {
                None
        });

        for (m,g2) in ms {
            if let Outcome::Moves(ms2) = g2.search_all(&ts, None) {
                let val = -self.quiescence(&ts, g2, ms2, k + 1, -alpha, -beta);
                if val >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }

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



