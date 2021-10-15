
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
pub use crate::timer::*;

// use std::sync::RwLock;
use parking_lot::RwLock;

use rayon::Scope;
use rayon::prelude::*;

use rustc_hash::FxHashMap;

#[derive(Debug,Clone)]
pub struct Explorer {
    pub side:          Color,
    pub game:          Game,
    // pub stats:      ABStats
    pub depth:         u32,
    // pub should_stop:   
    pub timer:         Timer,
    pub parallel:      bool,
    pub trans_table:   TransTable,
}

// #[derive(Debug,Default,Clone)]
// pub struct TransTable {
//     pub map:      FxHashMap<GameState, SearchInfo>,
// }
pub type TransTable = FxHashMap<GameState, SearchInfo>;

#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub struct SearchInfo {
    // pub best_move:          Move,
    // pub refutation_move:    Move,
    pub pv:                 Move,
    pub depth_searched:     u32,
    pub score:              Score,

    // /// https://www.chessprogramming.org/Integrated_Bounds_and_Values
    // pub node_type:          i32,
    // pub node_type:          NodeType,
}

#[derive(Debug,Eq,PartialEq,Hash,Clone,Copy)]
pub enum NodeType {
    NodePV,
    NodeAll, // Score = upper bound
    NodeCut, // Score = lower bound
}

impl Explorer {
    pub fn new(side:          Color,
               game:          Game,
               depth:         u32,
               should_stop:   Arc<AtomicBool>,
               settings:      TimeSettings,
    ) -> Self {
        Self {
            side,
            game,
            // stats: ABStats::new(),
            depth,
            timer: Timer::new(should_stop, settings),
            parallel: true,
            trans_table: TransTable::default(),
        }
    }
}

impl SearchInfo {
    // pub fn new(depth_searched: u32, evaluation: Score, node_type: i32) -> Self {
    pub fn new(pv: Move, depth_searched: u32, score: Score) -> Self {
        Self {
            pv,
            depth_searched,
            score,
            // node_type,
        }
    }
}

// impl Ord for SearchInfo {
// }

#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub struct ABStats {
    // /// Min score for Self
    // pub alpha:      Option<(Eval, i32)>,
    // /// Max score for Other
    // pub beta:       Option<(Eval, i32)>,
    /// Min score for Self
    pub alpha:      i32,
    /// Max score for Other
    pub beta:       i32,
    pub leaves:     u64,
}

impl ABStats {
    pub fn new() -> Self {
        Self {
            alpha:      i32::MIN,
            beta:       i32::MAX,
            // alpha:      None,
            // beta:       None,
            leaves:     0,
        }
    }
}

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

        let tt = RwLock::new(self.trans_table.clone());

        let mut out: Vec<(Move,i32)> = moves.into_par_iter()
        // let mut out: Vec<(Move,i32)> = moves.into_iter()
                .map(|m| {
                    let g2 = self.game.make_move_unchecked(&ts, &m).unwrap();
                    let alpha = (None,i32::MIN);
                    let beta  = (None,i32::MAX);
                    let (_,score) = self._ab_search(&ts, &tt, g2, self.depth, 1, Some(m), alpha, beta);
                    (m,score)
                })
                .collect();

        // let mut out: Vec<(Move,i32)> = if par {
        //     moves.into_par_iter()
        //     .map(|m| {
        //         let g2 = self.game.make_move_unchecked(&ts, &m).unwrap();
        //         let alpha = (None,i32::MIN);
        //         let beta  = (None,i32::MAX);
        //         let (_,score) = self._ab_search(&ts, &tt, g2, self.depth, 1, Some(m), alpha, beta);
        //         (m,score)
        //     })
        //     .collect()
        // } else {
        //     moves.into_iter()
        //         .map(|m| {
        //             let g2 = self.game.make_move_unchecked(&ts, &m).unwrap();
        //             let alpha = (None,i32::MIN);
        //             let beta  = (None,i32::MAX);
        //             let (_,score) = self._ab_search(&ts, &tt, g2, self.depth, 1, Some(m), alpha, beta);
        //             (m,score)
        //         })
        //         .collect()
        // };

        out.sort_by(|a,b| a.1.cmp(&b.1));
        out.reverse();
        if print {
            for (m,s) in out.iter() {
                eprintln!("{:?}: {:?}", s, m);
            }
        }
        out
    }

    pub fn rank_moves(&self, ts: &Tables, print: bool, par: bool) -> Vec<(Move,i32)> {
        let moves = self.game.search_all(&ts, self.game.state.side_to_move);

        if moves.is_end() {
            return vec![];
        }
        let moves = moves.get_moves_unsafe();

        // self.rank_moves_list(&ts, print, moves, par)
        self.rank_moves_list(&ts, print, moves)
    }

}

/// Misc
impl Explorer {
    // pub fn order_moves<'a>(&self, ts: &Tables, mut moves: &'a mut [(Move,Score,Game)]) -> &'a [(Move,Score,Game)] {
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

/// AB search
impl Explorer {

    pub fn ab_search(&self, ts: &Tables, m0: Move) -> i32 {
        let g = self.game.clone();

        let tt = RwLock::new(self.trans_table.clone());

        match g.make_move_unchecked(&ts, &m0) {
            Ok(g2) => {
                let alpha = (None,i32::MIN);
                let beta  = (None,i32::MAX);
                let (m2,score) = self._ab_search(&ts, &tt, g2, self.depth, 1, Some(m0), alpha, beta);
                score
            },
            Err(win) => {
                panic!("bad move? {:?}", &win);
            }
        }
    }

    pub fn _ab_search(
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

        let moves = g.search_all(&ts, g.state.side_to_move);

        match moves {
            Outcome::Checkmate(c) => return (m0, 100_000_000 - k),
            Outcome::Stalemate    => return (m0, -100_000_000 + k),
            Outcome::Moves(_)     => {},
        }

        if depth == 0 {
            let score = g.evaluate(&ts).sum(self.side);
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
                    let score = g2.evaluate(&ts);
                    let score = score.sum(self.side);
                    let score = if maximizing { -score } else { score };

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

            if maximizing {
                // maximize self
                let val2 = self._ab_search(
                    // &ts, &trans_table, g2, depth - 1, k + 1, Some(m), alpha, beta);
                    &ts, &trans_table, g2, depth - 1, k + 1, m0, alpha, beta);
                val = val.max(val2.1);

                if val >= beta.1 {
                    return beta;
                }

                if alpha.0.is_none() {
                    alpha = (m0, val);
                } else if val > alpha.1 {
                    alpha = (m0, val);
                }

            } else {
                // minimize other
                let val2 = self._ab_search(
                    // &ts, &trans_table, g2, depth - 1, k + 1, Some(m), alpha, beta);
                    &ts, &trans_table, g2, depth - 1, k + 1, m0, alpha, beta);
                val = val.min(val2.1);

                if val <= alpha.1 {
                    return alpha;
                }

                if beta.0.is_none() {
                    beta = (m0, val);
                } else if val < beta.1 {
                    beta = (m0, val);
                }
            }

        }


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

        if maximizing {
            alpha
        } else {
            beta
        }
    }

    // pub fn negamax(&self, ts: &Tables, max_depth: u32, col: Color) -> Vec<(Move,Eval)> {

    //     let moves = self.game.search_all(&ts, self.game.state.side_to_move);
    //     // let moves = vec![Move::Capture { from: "B7".into(), to: "D8".into()}];

    //     // let mut out = vec![];
    //     // for m in moves.into_iter() {
    //     //     if let MoveResult::Legal(g) = self.game.make_move_unchecked(&ts, &m) {
    //     //         if let Some(s) = self._negamax(&ts, g, max_depth) {
    //     //             out.push((m,s));
    //     //         } else {
    //     //             panic!("negamax panic 1: {:?}", m);
    //     //         }
    //     //     } else {
    //     //         panic!("negamax panic 0: {:?}", m);
    //     //     }
    //     // }

    //     // out
    //     unimplemented!()
    // }

    // fn _negamax(&self, ts: &Tables, g: Game, depth: u32) -> Option<Eval> {

    //     if depth == 0 { return Some(g.evaluate(&ts)); }

    //     let moves = g.search_all(&ts, g.state.side_to_move);

    //     // let out = moves.into_iter()
    //     //     .map(|m| {
    //     //         if let MoveResult::Legal(g) = g.make_move_unchecked(&ts, &m) {
    //     //             // let s = self._negamax(&ts, g, depth - 1);
    //     //             // (m,s)
    //     //             if let Some(s) = self._negamax(&ts, g, depth - 1) {
    //     //                 Some((m,s))
    //     //             } else { None }
    //     //         } else {
    //     //             panic!("negamax panic: {:?}", m)
    //     //         }
    //     //     })
    //     //     .flatten()
    //     //     ;

    //     // let ms = out.clone();

    //     // let out = out.map(|x| x.1);

    //     // for (m,e) in ms.into_iter() {
    //     //     eprintln!("{} = {:?}", e.score_material.diff(), m);
    //     // }

    //     // XXX: ! ?

    //     // let out = if g.state.side_to_move == White {
    //     //     Eval::max(g.state.side_to_move, out)
    //     // } else {
    //     //     Eval::min(g.state.side_to_move, out)
    //     // };

    //     // for (m,e) in ms.into_iter() {
    //     //     eprintln!("1: {} = {:?}", e.score_material.diff(), m);
    //     // }

    //     // if g.state.side_to_move == White {
    //     //     println!("wat 0");
    //     // } else {
    //     //     println!("wat 1");
    //     // }

    //     // let out = Eval::best(self.side, !g.state.side_to_move, out, |x| x.1);
    //     // eprintln!("out = {:?}", out);

    //     // let out = Eval::max(!g.state.side_to_move, out);
    //     // eprintln!("out = {:?}", out.unwrap().score_material.diff());

    //     // out.map(|x| x.1)
    //     unimplemented!()
    // }

}

/// Quiescence
impl Explorer {

    fn quiescence(&self,
                  ts: &Tables,
                  g: Game,
                  moves: Option<Outcome>,
                  k: i32,
                  mut alpha: i32,
                  mut beta: i32
    ) -> i32 {
        // println!("quiescence 0");
        let stand_pat = g.evaluate(&ts).sum(self.side);

        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let moves = match moves {
            Some(ms) => ms,
            None     => g.search_all(&ts, g.state.side_to_move),
        };
        match moves {
            Outcome::Checkmate(c) => return 100_000_000 - k,
            Outcome::Stalemate    => return -100_000_000 + k,
            Outcome::Moves(_)     => {},
        }

        let moves = moves.into_iter().filter(|m| m.filter_all_captures());

        for m in moves {
            if let Ok(g2) = g.make_move_unchecked(&ts, &m) {
                let score = -self.quiescence(&ts, g2, None, k+1, -alpha, -beta);

                if score >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }

        alpha
        // unimplemented!()
    }

}

// impl PartialOrd for NodeType {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         use std::cmp::Ordering;
//         use NodeType::*;
//         let out = match (self, other) {
//             (x, y) if x == y => Ordering::Equal,
//             (NodePV, _)      => Ordering::Greater,
//             (_, NodePV)      => Ordering::Less,
//             _                => unimplemented!()
//         };
//         Some(out)
//     }
// }

// impl Ord for NodeType {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.partial_cmp(&other).unwrap()
//     }
// }

