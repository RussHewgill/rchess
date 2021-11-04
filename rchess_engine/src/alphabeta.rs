
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;
use crate::explore::*;

use crate::stats;

pub use ABResults::*;

use std::collections::VecDeque;

use std::sync::atomic::Ordering::SeqCst;

#[derive(Debug,Default,PartialEq,PartialOrd,Clone)]
pub struct ABResult {
    pub moves:    VecDeque<Move>,
    pub score:    Score,
}

impl ABResult {
    pub fn new_empty(score: Score) -> Self {
        Self {
            moves: VecDeque::default(),
            score,
        }
    }

    pub fn new_with(moves: VecDeque<Move>, score: Score) -> Self {
        Self {
            moves,
            score,
        }
    }

    pub fn neg_score(&mut self) {
        self.score = -self.score;
    }
}

#[derive(Debug,PartialEq,PartialOrd,Clone)]
pub enum ABResults {
    ABSingle(ABResult),
    ABList(ABResult, Vec<ABResult>),
    // ABPrune,
    ABNone,
}

/// Negamax AB
impl Explorer {

    /// returns (can_use, SearchInfo)
    /// true: when score can be used instead of searching
    /// false: score can be used ONLY for move ordering
    pub fn check_tt_negamax(&self,
                ts:             &Tables,
                g:              &Game,
                depth:          Depth,
                // maximizing:     bool,
                tt_r:           &TTRead,
                mut stats:      &mut SearchStats,
    ) -> Option<(SICanUse,SearchInfo)> {
        if let Some(si) = tt_r.get_one(&g.zobrist) {
            if si.depth_searched >= depth {
                stats!(stats.tt_hits += 1);
                Some((SICanUse::UseScore,si.clone()))
            } else {
                stats!(stats.tt_halfmiss += 1);
                Some((SICanUse::UseOrdering,si.clone()))
            }
        } else {
            // if g.zobrist == Zobrist(0x1eebfbac03c62e9d) { println!("wat wat 3"); }
            stats!(stats.tt_misses += 1);
            // let score = self._ab_search(&ts, &g, depth - 1, k + 1, alpha, beta, !maximizing);
            // score
            None
        }
    }

    #[allow(unused_doc_comments,unused_labels)]
    /// alpha: the MIN score that the maximizing player is assured of
    /// beta:  the MAX score that the minimizing player is assured of
    pub fn _ab_search_negamax(
        &self,
        ts:                      &Tables,
        g:                       &Game,
        max_depth:               Depth,
        depth:                   Depth,
        ply:                     i16,
        (mut alpha, mut beta):   (i32,i32),
        mut stats:               &mut SearchStats,
        prev_mvs:                VecDeque<(Zobrist,Move)>,
        mut history:             &mut [[[Score; 64]; 64]; 2],
        tt_r:                    &TTRead,
        tt_w:                    TTWrite,
        root:                    bool,
        do_null:                 bool,
    ) -> ABResults {

        // trace!("negamax entry, ply {}, a/b = {:>10}/{:>10}", k, alpha, beta);

        if self.stop.load(SeqCst) {
            return ABNone;
        }

        {
            let r = self.best_mate.read();
            if let Some(best) = *r {
                drop(r);
                if best <= max_depth {
                    trace!("halting search of depth {}, faster mate found", max_depth);
                    return ABNone;
                }
            }
        }

        let moves = g.search_all(&ts);

        let mut moves: Vec<Move> = match moves {
            Outcome::Checkmate(c) => {
                let score = 100_000_000 - ply as Score;
                if !tt_r.contains_key(&g.zobrist) {
                    stats!(stats.leaves += 1);
                    stats!(stats.checkmates += 1);
                }

                return ABSingle(ABResult::new_empty(-score));

                // // XXX: backwards, but gets negated 1 level above
                // if self.side == g.state.side_to_move {
                //     return ABSingle(ABResult::new_empty(score));
                // } else {
                //     return ABSingle(ABResult::new_empty(-score));
                // }

            },
            Outcome::Stalemate    => {
                let score = -200_000_000 + ply as Score;
                if !tt_r.contains_key(&g.zobrist) {
                    stats!(stats.leaves += 1);
                    stats!(stats.stalemates += 1);
                }
                // TODO: adjust stalemate value when winning/losing
                return ABSingle(ABResult::new_empty(-score));
            },
            Outcome::Moves(ms)    => ms,
        };

        // /// XXX: stat padding by including nodes found in TT
        // stats!(stats.inc_nodes_arr(ply));
        // stats!(stats.nodes += 1);

        // if !tt_r.contains_key(&g.zobrist) {
        //     // /// XXX: stat padding by including nodes found in TT
        //     stats!(stats.inc_nodes_arr(k));
        //     stats!(stats.nodes += 1);
        //     trace!("adding node: {}, {:?}", k, g.zobrist);
        // } else {
        //     trace!("skipped node: {}, {:?}", k, g.zobrist);
        // }

        // let mvs = self.move_history.clone();

        if depth == 0 {
            if !tt_r.contains_key(&g.zobrist) {
                stats!(stats.leaves += 1);
            }

            #[cfg(feature = "qsearch")]
            let score = {
                // trace!("    beginning qsearch, {:?}, a/b: {:?},{:?}",
                //        prev_mvs.front().unwrap().1, alpha, beta);
                let score = self.qsearch(&ts, &g, (ply,0), alpha, beta, &mut stats);
                // trace!("    returned from qsearch, score = {}", score);
                score
                // if g.state.side_to_move == Black { score } else { -score }
            };

            #[cfg(not(feature = "qsearch"))]
            let score = if g.state.side_to_move == Black {
                -g.evaluate(&ts).sum()
            } else {
                g.evaluate(&ts).sum()
            };

            return ABSingle(ABResult::new_empty(score));
        }

        #[cfg(feature = "pvs_search")]
        let mut is_pv_node = beta != alpha + 1;
        #[cfg(not(feature = "pvs_search"))]
        let is_pv_node = false;

        // if is_pv_node {
        //     // trace!("is_pv_node, ply {}: g = {:?}", ply, g);
        //     trace!("is pv_node, ply {}: {}, {}", ply, alpha, beta);
        // } else {
        //     trace!("not pv_node, ply {}: {}, {}", ply, alpha, beta);
        // }

        /// Null Move pruning
        #[cfg(feature = "null_pruning")]
        if g.state.checkers.is_empty()
            && depth >= 2
            && !is_pv_node
            && g.state.phase < 200
            && do_null {
                if self.prune_null_move_negamax(
                    ts, g, max_depth, depth, ply, alpha, beta, &mut stats,
                    prev_mvs.clone(), &mut history, tt_r, tt_w.clone()) {

                    // return ABNone;
                    return ABSingle(ABResult::new_empty(beta));
                }
        }

        /// MVV LVA move ordering
        order_mvv_lva(&mut moves);

        // moves.sort_by_cached_key(|m| g.static_exchange(&ts, *m));
        // moves.reverse();

        /// History Heuristic ordering
        #[cfg(feature = "history_heuristic")]
        order_moves_history(&history[g.state.side_to_move], &mut moves);

        /// Make move, Lookup games in Trans Table
        let mut gs: Vec<(Move,Game,Option<(SICanUse,SearchInfo)>)> = {
            let mut gs0 = moves.into_iter()
                .flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, m) {
                    // let mut ss = SearchStats::default();
                    let tt = self.check_tt_negamax(
                        &ts, &g2, depth, &tt_r, &mut stats);
                    // *stats = *stats + ss;
                    Some((m,g2,tt))
            } else {
                None
            });
            gs0.collect()
        };

        /// Move Ordering
        order_searchinfo(false, &mut gs[..]);

        let mut node_type = Node::All;

        let mut search_pv = true;
        let mut skip_pv   = false;

        let mut moves_searched = 0;
        let mut val = i32::MIN + 200;
        let mut val: (Option<(Zobrist,Move,ABResult)>,i32) = (None,val);
        let mut list = vec![];

        'outer: for (mv,g2,tt) in gs.iter() {
            let zb = g2.zobrist;

            #[cfg(feature = "pvs_search")]
            if depth < 3 {
                skip_pv = true;
            }

            let (can_use,res) = match tt {
                Some((SICanUse::UseScore,si)) => {
                    let mut si = si.clone();
                    match si.node_type {
                        Node::PV  => {},
                        Node::All => if si.score <= alpha {
                            // trace!("Node::All, using alpha {}", alpha);
                            si.score = alpha;
                        },
                        Node::Cut => if si.score >= beta {
                            // trace!("Node::Cut, using beta {}", beta);
                            si.score = beta;
                        },
                        _         => unimplemented!(),
                    }
                    (true, ABResult::new_with(si.moves.into(),si.score))
                },
                _ => 'search: {
                    let mut pms = prev_mvs.clone();
                    pms.push_back((g.zobrist,*mv));

                    let mut lmr = true;
                    let mut depth2 = depth - 1;

                    // #[cfg(not(feature = "late_move_reduction"))]
                    // {
                    //     if g2.state.checkers.is_not_empty() || g.state.checkers.is_not_empty() {
                    //         trace!("check extension: = {:?}, {:?}", mv, pms.front().unwrap().1);
                    //         depth2 = depth;
                    //     }
                    // }

                    /// not reducing when in check replaces check extension
                    #[cfg(feature = "late_move_reduction")]
                    if lmr
                        && !is_pv_node
                        && moves_searched >= 4
                        && ply >= 3
                        && depth > 2
                        // && depth > 3
                        && !mv.filter_all_captures()
                        && !mv.filter_promotion()
                        && g.state.checkers.is_empty()
                        && g2.state.checkers.is_empty()
                    {
                        let depth3 = depth - 3;
                        match self._ab_search_negamax(
                            &ts, &g2, max_depth, depth3, ply + 1,
                            (-beta, -alpha), &mut stats,
                            pms.clone(), &mut history, tt_r, tt_w.clone(),
                            false,
                            // XXX: No Null pruning inside reduced depth search ?
                            true) {
                            ABSingle(mut res) => {
                                res.neg_score();
                                if res.score <= alpha {
                                    stats!(stats.lmrs.0 += 1);
                                    res.moves.push_front(*mv);
                                    break 'search (false,res);
                                }
                            },
                            // ABNone => {},
                            _ => {},
                        }

                    }

                    #[cfg(feature = "pvs_search")]
                    let (a2,b2) = if skip_pv || search_pv {
                        (-beta, -alpha)
                    } else {
                        (-alpha - 1, -alpha)
                    };
                    #[cfg(not(feature = "pvs_search"))]
                    let (a2,b2) = (-beta, -alpha);

                    match self._ab_search_negamax(
                        &ts, &g2, max_depth, depth2, ply + 1,
                        (a2, b2), &mut stats,
                        pms.clone(), &mut history, tt_r, tt_w.clone(), false, true) {
                        ABSingle(mut res) => {
                            res.moves.push_front(*mv);
                            res.neg_score();

                            #[cfg(feature = "pvs_search")]
                            if !search_pv && res.score > alpha {
                                match self._ab_search_negamax(
                                    &ts, &g2, max_depth, depth2, ply + 1,
                                    (-beta, -alpha), &mut stats,
                                    pms, &mut history, tt_r, tt_w.clone(), false, true) {
                                    ABSingle(mut res2) => {
                                        res2.neg_score();
                                        res2.moves.push_front(*mv);
                                        res = res2;
                                    },
                                    _ => {
                                        break 'outer
                                    },
                                }
                            }

                            if root {
                                list.push(res.clone());
                            }
                            (false, res)
                        },
                        // ABPrune => {
                        //     unimplemented!()
                        // },
                        _ => {
                            // continue 'outer;
                            break 'outer;
                        },
                    }

                },
            };
            let mut b = false;

            if res.score > val.1 {
                val.1 = res.score;
                // if !can_use { mv_seq.push(*mv) };
                val.0 = Some((zb, *mv, res.clone()))
            }

            #[cfg(not(feature = "negamax_only"))]
            { if res.score >= beta { // Fail Soft
                b = true;
                // return beta;
            }

            if !b && val.1 > alpha {
                node_type = Node::PV;
                alpha = val.1;
                #[cfg(feature = "pvs_search")]
                if true { search_pv = false; }
            }

            if b {
                // node_type = Some(Node::Cut);
                node_type = Node::Cut;

                #[cfg(feature = "history_heuristic")]
                if !mv.filter_all_captures() {
                    history[g.state.side_to_move][mv.sq_from()][mv.sq_to()] += ply as Score * ply as Score;
                }

                if moves_searched == 0 {
                    stats!(stats.beta_cut_first.0 += 1);
                } else {
                    stats!(stats.beta_cut_first.1 += 1);
                }

                break;
            }}

            moves_searched += 1;
        }

        // if root && k > 5 {
        //     trace!("node_type = {:?}", node_type);
        // }


        /// XXX: stat padding by including nodes found in TT
        stats!(stats.inc_nodes_arr(ply));
        stats!(stats.nodes += 1);

        // if !tt_r.contains_key(&g.zobrist) {
        //     // /// XXX: stat padding by including nodes found in TT
        //     stats!(stats.inc_nodes_arr(ply));
        //     stats!(stats.nodes += 1);
        //     // trace!("adding node: {}, {:?}", k, g.zobrist);
        // } else {
        //     // trace!("skipped node: {}, {:?}", k, g.zobrist);
        // }

        match &val.0 {
            // Some((zb,mv,mv_seq)) => Some(((mv_seq.clone(),val.1),(alpha,beta))),
            Some((zb,mv,res)) => {

                // trace!("inserting TT, zb = {:?}", g.zobrist);
                Self::tt_insert_deepest(
                    &tt_r, tt_w, g.zobrist,
                    // &tt_r, tt_w, *zb,
                    SearchInfo::new(
                        *mv,
                        res.moves.clone().into(),
                        // res.moves.len() as u8,
                        // res.moves.len() as u8 - 1,
                        // res.moves.len() as u8,
                        depth - 1,
                        // depth,
                        node_type,
                        res.score,
                    ));

                // match node_type {
                //     None => {},
                //     Some(nt) => {
                //     }
                // }

                if root {
                    ABList(res.clone(), list)
                } else {
                    ABSingle(res.clone())
                }
            },
            _                    => ABNone,
        }
    }

    // fn _ab_score_negamax(
    //     &self,
    //     (mv,g2):                       (Move,&Game),
    //     (can_use,mut mv_seq,score):    (bool,Vec<Move>,Score),
    //     mut val:                       &mut (Option<(Zobrist,Move,Vec<Move>)>,i32),
    //     depth:                         Depth,
    //     mut alpha:                     &mut i32,
    //     mut beta:                      &mut i32,
    // ) -> bool {
    //     unimplemented!()
    // }

}

/// AB search
impl Explorer {

    /// returns (can_use, SearchInfo)
    /// true: when score can be used instead of searching
    /// false: score can be used ONLY for move ordering
    pub fn check_tt(&self,
                ts:             &Tables,
                g:              &Game,
                depth:          Depth,
                maximizing:     bool,
                tt_r:           &TTRead,
                mut stats:      &mut SearchStats,
    ) -> Option<(SICanUse,SearchInfo)> {
        if let Some(si) = tt_r.get_one(&g.zobrist) {
            if si.depth_searched >= depth {
                stats!(stats.tt_hits += 1);
                Some((SICanUse::UseScore,si.clone()))
            } else {
                stats!(stats.tt_misses += 1);
                Some((SICanUse::UseOrdering,si.clone()))
            }

        } else {
            // if g.zobrist == Zobrist(0x1eebfbac03c62e9d) { println!("wat wat 3"); }
            stats!(stats.tt_misses += 1);
            // let score = self._ab_search(&ts, &g, depth - 1, k + 1, alpha, beta, !maximizing);
            // score
            None
        }
    }

    #[allow(unused_doc_comments,unused_labels)]
    /// alpha: the min score that the maximizing player is assured of
    /// beta:  the max score that the minimizing player is assured of
    pub fn _ab_search(
        &self,
        ts:                 &Tables,
        g:                  &Game,
        max_depth:          Depth,
        depth:              Depth,
        k:                  i16,
        mut alpha:          i32,
        mut beta:           i32,
        maximizing:         bool,
        mut stats:          &mut SearchStats,
        // mv0:                Move,
        prev_mvs:           VecDeque<(Zobrist,Move)>,
        mut history:        &mut [[[Score; 64]; 64]; 2],
        tt_r:               &TTRead,
        tt_w:               TTWrite,
    // ) -> Option<(Vec<Move>, Score)> {
    ) -> Option<((Vec<Move>, Score), (i32, i32))> {

        if self.stop.load(SeqCst) {
            return None;
        }

        {
            let r = self.best_mate.read();
            if let Some(best) = *r {
                drop(r);
                if best <= max_depth {
                    trace!("halting search of depth {}, faster mate found", max_depth);
                    return None;
                }
            }
        }

        let moves = g.search_all(&ts);

        let mut moves: Vec<Move> = match moves {
            Outcome::Checkmate(c) => {
                let score = 100_000_000 - k as Score;
                // if !self.tt_contains(&g.zobrist) {
                if !tt_r.contains_key(&g.zobrist) {
                    stats!(stats.leaves += 1);
                    stats!(stats.checkmates += 1);
                }
                if maximizing {
                    // return Some((vec![mv0],-score));
                    return Some(((vec![], -score),(alpha,beta)));
                } else {
                    // return Some((vec![mv0],score));
                    return Some(((vec![], score),(alpha,beta)));
                }

            },
            Outcome::Stalemate    => {
                let score = -20_000_000 + k as Score;
                // if !self.tt_contains(&g.zobrist) {
                if !tt_r.contains_key(&g.zobrist) {
                    stats!(stats.leaves += 1);
                    stats!(stats.stalemates += 1);
                }
                // return Some((vec![],score));
                return Some(((vec![], score),(alpha,beta)));
            },
            Outcome::Moves(ms)    => ms,
        };

        // if !tt_r.contains_key(&g.zobrist) {}
        /// XXX: stat padding by including nodes found in TT
        stats!(stats.inc_nodes_arr(k));
        stats!(stats.nodes += 1);

        if depth == 0 {

            if !tt_r.contains_key(&g.zobrist) {
                stats!(stats.leaves += 1);
            }

            // let mv0 = prev_mvs.back().unwrap().1;

            // // if self.side == Black {
            // let score = if !maximizing {
            //     self.quiescence(
            //         ts, g, moves, k, -alpha, -beta, maximizing, &mut stats,
            //     )
            // } else {
            //     self.quiescence(
            //         ts, g, moves, k, alpha, beta, maximizing, &mut stats,
            //     )
            // };

            // let moves = moves.into_iter().filter(|x| x.filter_all_captures()).collect();

            // let score = if g.state.checkers.is_empty() {
            //     self.quiescence2(
            //         // ts, g, moves, k, alpha, beta, maximizing, &mut stats,
            //         ts, g, k, alpha, beta, maximizing, &mut stats,
            //     )
            // } else {
            //     let score = g.evaluate(&ts).sum();
            //     if self.side == Black { -score } else { score }
            // };

            let score = g.evaluate(&ts).sum();
            let score = if self.side == Black { -score } else { score };

            return Some(((vec![], score),(alpha,beta)));

            // let score = g.evaluate(&ts).sum();
            // if self.side == Black {
            //     return Some(((vec![], -score),(alpha,beta)));
            // } else {
            //     return Some(((vec![], score),(alpha,beta)));
            // }

        }

        /// Null Move pruning
        #[cfg(feature = "null_pruning")]
        if g.state.checkers.is_empty()
            && g.state.phase < 200
            && self.prune_null_move(
                ts, g, max_depth, depth, k, alpha, beta, maximizing, &mut stats,
                prev_mvs.clone(), &mut history, tt_r, tt_w.clone()) {
                return None;
        }

        /// MVV LVA move ordering
        order_mvv_lva(&mut moves);

        /// History Heuristic ordering
        #[cfg(feature = "history_heuristic")]
        order_moves_history(&history[g.state.side_to_move], &mut moves);

        /// Make move, Lookup games in Trans Table
        // #[cfg(feature = "par")]
        let mut gs: Vec<(Move,Game,Option<(SICanUse,SearchInfo)>)> = {
            let mut gs0 = moves.into_iter()
                .flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, m) {
                        let mut ss = SearchStats::default();
                        let tt = self.check_tt(&ts, &g2, depth, maximizing, &tt_r, &mut ss);
                        // Some(((m,g2,tt), ss))
                        *stats = *stats + ss;
                        Some((m,g2,tt))
            } else {
                None
            });
            gs0.collect()
        };

        /// Move Ordering
        order_searchinfo(maximizing, &mut gs[..]);

        // let mut node_type = Node::All;
        let mut node_type = Node::PV;
        // let mut parent_node_type = None;

        /// Get parent node type
        let moves = match tt_r.get_one(&g.zobrist) {
            None     => {
                // panic!("no parent node?");
            },
            Some(si) => {
                // parent_node_type = Some(si.node_type);
                match si.node_type {
                    Node::Cut => {
                        node_type = Node::All;
                        /// Cut nodes only need one child to be searched
                        gs.truncate(1);
                    },
                    /// Each child of an All node is a Cut nodes
                    Node::All => node_type = Node::Cut,
                    _         => {},
                }
            }
        };

        let mut val = if maximizing { i32::MIN } else { i32::MAX };
        let mut val: (Option<(Zobrist,Move,Vec<Move>)>,i32) = (None,val);

        // let (alpha0,beta0) = (alpha,beta);

        let mut moves_searched = 0;

        'outer: for (mv,g2,tt) in gs.iter() {

            let zb = g2.zobrist;

            /// Cycle prevention
            if self.cycle_prevention(&ts, (mv,g2), &prev_mvs) {
                panic!("wat: {:?}\n {:?}", mv, g2)
            }

            let (can_use,mut mv_seq,score) = match tt {
                Some((SICanUse::UseScore,si)) => {
                    // return (si.moves.clone(),si.score);
                    // (true,si.moves.clone(),si.score)
                    (true,si.moves.to_vec(),si.score)
                },
                _ => 'search: {
                    let mut pms = prev_mvs.clone();
                    pms.push_back((g.zobrist,*mv));

                    let mut lmr = true;

                    let mut depth2 = depth - 1;

                    /// not reducing when in check replaces check extension
                    #[cfg(feature = "late_move_reduction")]
                    if lmr
                        && moves_searched >= 4
                        && k >= 3
                        && depth > 2
                        // && depth > 3
                        && !mv.filter_all_captures()
                        && !mv.filter_promotion()
                        && g.state.checkers.is_empty()
                        && g2.state.checkers.is_empty()
                    {
                        let depth2 = depth - 2;
                        // trace!("Checking Late Move Reduction");
                        if let Some(((mv_seq,score),_)) = self._ab_search(
                            &ts, &g2, max_depth, depth2, k + 1,
                            alpha, beta, !maximizing, &mut stats, pms.clone(),
                            &mut history,
                            tt_r, tt_w.clone()) {
                            if maximizing {
                                if score <= alpha {
                                    // trace!("Late move reduction success 1");
                                    stats!(stats.lmrs.0 += 1);
                                    break 'search (false,mv_seq,score);
                                }
                            } else {
                                if score >= beta {
                                    // trace!("Late move reduction success -1");
                                    stats!(stats.lmrs.1 += 1);
                                    break 'search (false,mv_seq,score);
                                }
                            }
                        } else {
                            break 'outer;
                        }
                    }

                    if let Some(((mv_seq,score),_)) = self._ab_search(
                        &ts, &g2, max_depth, depth2, k + 1,
                        // alpha, beta, !maximizing, &mut stats, *mv,
                        alpha, beta, !maximizing, &mut stats, pms,
                        &mut history,
                        tt_r, tt_w.clone(),
                    ) {
                        (false,mv_seq,score)
                    } else {
                        break 'outer;
                    }

                },
            };

            // /// basic minimax
            // if maximizing {
            //     val.1 = i32::max(val.1, score);
            // } else {
            //     val.1 = i32::min(val.1, score);
            // }

            let b = self._ab_score(
                (*mv,&g2),
                (can_use,mv_seq,score),
                &mut val,
                depth,
                &mut alpha,
                &mut beta,
                maximizing,
                // mv0
            );
            if b {
                node_type = Node::Cut;

                if !mv.filter_all_captures() {
                    history[g.state.side_to_move][mv.sq_from()][mv.sq_to()] += k as Score * k as Score;
                }

                if moves_searched == 0 {
                    stats!(stats.beta_cut_first.0 += 1);
                } else {
                    stats!(stats.beta_cut_first.1 += 1);
                }

                break;
            }

            moves_searched += 1;
        }

        // XXX: depth or depth - 1? Update: Pretty sure depth - 1 is correct
        if let Some((zb,mv,mv_seq)) = &val.0 {
            let mut mv_seq = mv_seq.clone();
            Self::tt_insert_deepest(
                &tt_r, tt_w,
                *zb, SearchInfo::new(*mv, mv_seq, depth - 1, node_type, val.1));
        }

        stats!(stats.alpha = stats.alpha.max(alpha));
        stats!(stats.beta  = stats.beta.max(beta));

        match &val.0 {
            Some((zb,mv,mv_seq)) => Some(((mv_seq.clone(),val.1),(alpha,beta))),
            _                    => None,
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
        // mv0:                           Move,
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

            if val.1 > *alpha {
                *alpha = val.1;
            }

            // *alpha = i32::max(*alpha, val.1);
            if val.1 >= *beta { // Beta cutoff
                // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 2"); }
                // self.trans_table.insert(
                //     zb, SearchInfo::new(mv, mv_seq.clone(), depth, Node::Cut, val.1));
                return true;
                // return Some(Node::Cut);
            }

            // self.trans_table.insert_replace(
            //     zb, SearchInfo::new(mv, depth, Node::All, val.1));
        } else {
            // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 3"); }
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
                // return Some(Node::Cut);
            }

            // node_type = Node::All;
            // self.trans_table.insert_replace(
            //     zb, SearchInfo::new(mv, depth, Node::All, val.1));
        }
        false
        // None
    }

}


