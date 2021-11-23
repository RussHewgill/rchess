
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;
use crate::explore::*;
#[cfg(feature = "syzygy")]
use crate::syzygy::{SyzygyTB, Wdl, Dtz};

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

    pub fn new_single(mv: Move, score: Score) -> Self {
        let mut moves = VecDeque::default();
        moves.push_back(mv);
        Self {
            moves,
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
    ABPrune(Score, Prune),
    ABSyzygy(ABResult),
    ABNone,
}

#[derive(Debug,PartialEq,PartialOrd,Clone)]
pub enum Prune {
    NullMove,
}

impl Explorer {

    /// returns (can_use, SearchInfo)
    /// true: when score can be used instead of searching
    /// false: score can be used ONLY for move ordering
    pub fn check_tt_negamax(
        &self,
        ts:             &Tables,
        // g:              &Game,
        zb:             &Zobrist,
        depth:          Depth,
        tt_r:           &TTRead,
        mut stats:      &mut SearchStats,
    ) -> Option<(SICanUse,SearchInfo)> {
        // if let Some(si) = tt_r.get_one(&g.zobrist) {
        if let Some(si) = tt_r.get_one(zb) {
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

    pub fn ab_search_negamax(
        &self,
        ts:             &Tables,
        mut stats:      &mut SearchStats,
        depth:          Depth,
    ) -> ABResults {

        let mut history = [[[0; 64]; 64]; 2];

        let mut stop_counter = 0;

        let mut cfg = ABConfig::new_depth(depth);
        cfg.root = true;

        let (alpha,beta) = (i32::MIN,i32::MAX);
        let (alpha,beta) = (alpha + 200,beta - 200);

        let mut g = self.game.clone();

        let tt_r = self.tt_rf.handle();
        let tt_w = self.tt_w.clone();

        let res = self._ab_search_negamax(
            ts, &mut g, cfg, depth,
            0, &mut stop_counter, (alpha, beta),
            &mut stats,
            VecDeque::new(),
            &mut history,
            &tt_r, tt_w.clone());

        res
    }

    #[allow(unused_doc_comments,unused_labels)]
    /// alpha: the MIN score that the maximizing player is assured of
    /// beta:  the MAX score that the minimizing player is assured of
    pub fn _ab_search_negamax(
        &self,
        ts:                      &Tables,
        // g:                       &Game,
        g:                       &Game,
        mut cfg:                 ABConfig,
        depth:                   Depth,
        ply:                     Depth,
        mut stop_counter:        &mut u16,
        (mut alpha, mut beta):   (i32,i32),
        mut stats:               &mut SearchStats,
        prev_mvs:                VecDeque<(Zobrist,Move)>,
        mut history:             &mut [[[Score; 64]; 64]; 2],
        tt_r:                    &TTRead,
        tt_w:                    TTWrite,
    ) -> ABResults {

        // trace!("negamax entry, ply {}, a/b = {:>10}/{:>10}", k, alpha, beta);

        /// Repetition checking
        {
            if let Some(k) = g.history.get(&g.zobrist) {
                if *k >= 2 {
                    let score = -STALEMATE_VALUE + ply as Score;
                    return ABSingle(ABResult::new_empty(-score));
                    // return ABSingle(ABResult::new_empty(0));
                }
            }
        }

        // TODO: bench this
        if *stop_counter > 2000 {
            if self.stop.load(SeqCst) {
                return ABNone;
            }
            {
                let r = self.best_mate.read();
                if let Some(best) = *r {
                    drop(r);
                    if best <= cfg.max_depth {
                        trace!("halting search of depth {}, faster mate found", cfg.max_depth);
                        return ABNone;
                    }
                }
            }
            *stop_counter = 0;
        } else {
            *stop_counter += 1;
        }

        let moves = g.search_all(&ts);

        let mut moves: Vec<Move> = match moves {
            Outcome::Checkmate(c) => {
                // let score = 100_000_000 - ply as Score;
                let score = CHECKMATE_VALUE - ply as Score;
                if !tt_r.contains_key(&g.zobrist) {
                    stats!(stats.leaves += 1);
                    stats!(stats.checkmates += 1);
                }

                return ABSingle(ABResult::new_empty(-score));

            },
            Outcome::Stalemate    => {
                let score = -STALEMATE_VALUE + ply as Score;
                if !tt_r.contains_key(&g.zobrist) {
                    stats!(stats.leaves += 1);
                    stats!(stats.stalemates += 1);
                }
                // TODO: adjust stalemate value when winning/losing
                return ABSingle(ABResult::new_empty(-score));
            },
            Outcome::Moves(ms)    => ms,
        };

        if cfg.root {
            moves.retain(|mv| !self.blocked_moves.contains(&mv));
        }

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
            };

            #[cfg(not(feature = "qsearch"))]
            let score = if g.state.side_to_move == Black {
                -g.evaluate(&ts).sum()
            } else {
                g.evaluate(&ts).sum()
            };

            return ABSingle(ABResult::new_empty(score));
        }

        /// Syzygy Probe
        #[cfg(feature = "syzygy")]
        if let Some(tb) = &self.syzygy {
            // let mv = Move::Quiet { from: "E5".into(), to: "F6".into(), pc: King };
            // let score = CHECKMATE_VALUE - (ply as Score + 4);
            // return ABResults::ABSingle(ABResult::new_single(mv, -score));

            match tb.probe_wdl(ts, g) {
                Ok(Wdl::Win) => {
                    // trace!("found WDL win: {:?}", Wdl::Win);
                    match tb.best_move(ts, g) {
                        Ok(Some((mv,dtz)))  => {
                            trace!("dtz,ply = {:?}, {:?}", dtz, ply);
                            // let score = CHECKMATE_VALUE - ply as Score - dtz.0 as Score;
                            let score = CHECKMATE_VALUE - dtz.add_plies(ply as i32).0.abs() as Score;

                            // XXX: wrong, but matches with other wrong mate in x count
                            let score = score + 1;
                            return ABResults::ABSingle(ABResult::new_single(mv, score));
                            // return ABResults::ABSyzygy(ABResult::new_single(mv, score));
                        },
                        Err(e) => {
                        },
                        _ => {
                        },
                    }
                },
                Ok(Wdl::Loss) => {
                    // return ABResults::ABSingle()
                },
                Ok(wdl) => {
                    trace!("found other WDL: {:?}", wdl);
                    // unimplemented!()
                },
                Err(e) => {
                    // unimplemented!()
                }
            }
        }


        #[cfg(feature = "pvs_search")]
        let mut is_pv_node = beta != alpha + 1;
        #[cfg(not(feature = "pvs_search"))]
        let is_pv_node = false;

        /// Null Move pruning
        #[cfg(feature = "null_pruning")]
        if g.state.checkers.is_empty()
            && depth >= 2
            && !is_pv_node
            && g.state.phase < 200
            && cfg.do_null {
                if self.prune_null_move_negamax(
                    ts, g, cfg, depth, ply, alpha, beta, &mut stats,
                    prev_mvs.clone(), &mut history, (tt_r, tt_w.clone())) {

                    // return ABNone;
                    // return ABSingle(ABResult::new_empty(beta));
                    return ABPrune(beta, Prune::NullMove);
                }
        }

        /// MVV LVA move ordering
        order_mvv_lva(&mut moves);

        // moves.sort_by_cached_key(|m| g.static_exchange(&ts, *m));
        // moves.reverse();

        /// History Heuristic ordering
        #[cfg(feature = "history_heuristic")]
        order_moves_history(&history[g.state.side_to_move], &mut moves);

        // /// Make move, Lookup games in Trans Table
        // let mut gs: Vec<(Move,Game,Option<(SICanUse,SearchInfo)>)> = {
        //     let mut gs0 = moves.into_iter()
        //         .flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, m) {
        //             let tt = self.check_tt_negamax(&ts, &g2.zobrist, depth, &tt_r, &mut stats);
        //             Some((m,g2,tt))
        //         } else {
        //             trace!("game not ok? {:?} {:?}", m, g);
        //             None
        //         });
        //     gs0.collect()
        // };

        let mut gs: Vec<(Move,Zobrist,Option<(SICanUse,SearchInfo)>)> = moves.into_iter()
            .map(|mv| {
                let zb = g.zobrist.update_move_unchecked(ts, g, mv);
                let tt = self.check_tt_negamax(&ts, &zb, depth, &tt_r, &mut stats);
                (mv,zb,tt)
            }).collect();

        /// Move Ordering
        order_searchinfo(&mut gs[..]);

        let mut node_type = Node::All;

        let mut search_pv = true;
        let mut skip_pv   = false;

        let mut moves_searched = 0;
        let mut val = i32::MIN + 200;
        let mut val: (Option<(Zobrist,Move,ABResult)>,i32) = (None,val);
        let mut list = vec![];

        'outer: for (mv,zb0,tt) in gs.iter() {
        // 'outer: for (mv,g2,tt) in gs.iter() {

            // let zb = g2.zobrist;

            let g2 = if let Ok(g2) = g.make_move_unchecked(ts, *mv) {
                g2
            } else { continue 'outer; };

            let zb = g2.zobrist;
            assert_eq!(zb, *zb0);
            // if zb != *zb0 {
            //     eprintln!("zb  = {:?}", zb);
            //     eprintln!("zb0 = {:?}", zb0);
            //     panic!("zb != zb0: {:?}\n{:?}", mv, g2);
            // }

            #[cfg(feature = "pvs_search")]
            if depth < 3 {
                skip_pv = true;
            }

            let (can_use,res) = match tt {

                // Some((SICanUse::UseScore,si)) => {
                //     let mut si = si.clone();
                //     // eprintln!("using si: = {:?}", si);
                //     (true, ABResult::new_with(si.moves.into(), si.score))
                // },

                // Some((SICanUse::UseScore,si)) => {
                //     let mut si = si.clone();
                //     match si.node_type {
                //         Node::PV  => {},
                //         Node::All => if si.score <= alpha {
                //             // trace!("Node::All, using alpha {}", alpha);
                //             si.score = alpha;
                //         },
                //         Node::Cut => if si.score >= beta {
                //             // trace!("Node::Cut, using beta {}", beta);
                //             si.score = beta;
                //         },
                //         _         => unimplemented!(),
                //     }
                //     (true, ABResult::new_with(si.moves.into(),si.score))
                // },
                _ => 'search: {
                    let mut pms = prev_mvs.clone();
                    pms.push_back((g.zobrist,*mv));

                    let mut cfg2 = cfg;
                    cfg2.do_null = true;
                    cfg2.root    = false;

                    let mut lmr = true;
                    let mut depth2 = depth - 1;

                    if mv.filter_all_captures() {
                        let see = g2.static_exchange(&ts, *mv).unwrap();
                        /// Capture with good SEE: do not reduce
                        if see > 0 {
                            lmr = false;
                        }
                    }

                    /// not reducing when in check replaces check extension
                    #[cfg(feature = "late_move_reduction")]
                    if lmr
                        && !is_pv_node
                        && moves_searched >= LMR_MIN_MOVES
                        // && ply >= LMR_MIN_PLY
                        && depth >= 3
                        && !mv.filter_promotion()
                        && g.state.checkers.is_empty()
                        && g2.state.checkers.is_empty()
                    {

                        // let depth3 = depth.checked_sub(LMR_REDUCTION).unwrap();
                        let depth3 = depth.checked_sub(3).unwrap();

                        // let depth3 = if moves_searched < 2 {
                        //     depth.checked_sub(1).unwrap()
                        // } else if moves_searched < 4 {
                        //     depth.checked_sub(2).unwrap()
                        // } else {
                        //     // let k = Depth::max(1, depth)
                        //     let k = 3;
                        //     depth.checked_sub(k).unwrap()
                        //     // depth.checked_sub(depth / 3).unwrap()
                        // };

                        // let depth3 = if ply < LMR_PLY_CONST {
                        //     // depth - LMR_REDUCTION
                        //     depth.checked_sub(1).unwrap()
                        // } else {
                        //     depth.checked_sub(depth / 3).unwrap()
                        // };

                        match self._ab_search_negamax(
                            ts, &g2, cfg2, depth3, ply + 1, &mut stop_counter,
                            (-beta, -alpha), &mut stats,
                            pms.clone(), &mut history, tt_r, tt_w.clone(),
                            // false,
                            // // XXX: No Null pruning inside reduced depth search ?
                            // true
                        ) {
                            ABSingle(mut res) | ABSyzygy(mut res) => {
                                res.neg_score();
                                if res.score <= alpha {
                                    stats!(stats.lmrs.0 += 1);
                                    res.moves.push_front(*mv);
                                    break 'search (false,res);
                                }
                            },
                            ABList(_, _) => panic!("found ABList when not root?"),
                            ABPrune(beta, prune) => {
                                // panic!("ABPrune 0");
                                // trace!("ABPrune 0: {:?} {:?}", beta, prune);
                            },
                            ABNone       => {},
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
                        ts, &g2, cfg2, depth2, ply + 1, &mut stop_counter,
                        (a2, b2), &mut stats,
                        pms.clone(), &mut history, tt_r, tt_w.clone()) {
                        ABSingle(mut res) | ABSyzygy(mut res) => {
                            res.moves.push_front(*mv);
                            res.neg_score();

                            #[cfg(feature = "pvs_search")]
                            if !search_pv && res.score > alpha {
                                match self._ab_search_negamax(
                                    ts, &g2, cfg2, depth2, ply + 1, &mut stop_counter,
                                    (-beta, -alpha), &mut stats,
                                    pms, &mut history, tt_r, tt_w.clone()) {
                                    ABSingle(mut res2) | ABSyzygy(mut res2) => {
                                        res2.neg_score();
                                        res2.moves.push_front(*mv);
                                        res = res2;
                                    },
                                    // ABList(_, _) => break 'outer,
                                    ABList(_, _) => panic!("found ABList when not root?"),
                                    ABPrune(beta, prune) => {
                                        panic!("ABPrune 1");
                                    },
                                    ABNone       => break 'outer,
                                }
                            }

                            if cfg.root {
                                list.push(res.clone());
                            }
                            (false, res)
                        },
                        ABPrune(beta, prune) => {
                            // panic!("ABPrune 2");
                            // trace!("ABPrune 2: {:?} {:?}", beta, prune);
                            continue 'outer;
                        },
                        // ABList(_, _) => break 'outer,
                        ABList(_, _) => panic!("found ABList when not root?"),
                        ABNone       => break 'outer,
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

                if !cfg.inside_null {
                    // trace!("inserting TT, zb = {:?}", g.zobrist);
                    Explorer::tt_insert_deepest(
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
                }

                // match node_type {
                //     None => {},
                //     Some(nt) => {
                //     }
                // }

                if cfg.root {
                    ABList(res.clone(), list)
                } else {
                    ABSingle(res.clone())
                }
            },
            _                    => ABNone,
        }
    }

}

/// Negamax AB
impl ExHelper {

    /// returns (can_use, SearchInfo)
    /// true: when score can be used instead of searching
    /// false: score can be used ONLY for move ordering
    pub fn check_tt_negamax(
        &self,
        ts:             &Tables,
        // g:              &Game,
        zb:             &Zobrist,
        depth:          Depth,
        tt_r:           &TTRead,
        mut stats:      &mut SearchStats,
    ) -> Option<(SICanUse,SearchInfo)> {
        // if let Some(si) = tt_r.get_one(&g.zobrist) {
        if let Some(si) = tt_r.get_one(zb) {
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

    pub fn ab_search_iter_deepening(
        &self,
        ts:             &Tables,
        mut stats:      &mut SearchStats,
        mut history:    &mut [[[Score; 64]; 64]; 2],
        depth:          Depth,
    ) -> ABResults {

        let mut stop_counter = 0;

        let mut cfg = ABConfig::new_depth(depth);
        cfg.root = true;

        let (alpha,beta) = (i32::MIN,i32::MAX);
        let (alpha,beta) = (alpha + 200,beta - 200);

        let mut g = self.game.clone();

        let tt_r = self.tt_r.clone();
        let tt_w = self.tt_w.clone();

        let res = self._ab_search_negamax(
            ts, &mut g, cfg, depth,
            0, &mut stop_counter, (alpha, beta),
            &mut stats,
            VecDeque::new(),
            &mut history,
            &tt_r, tt_w.clone());

        res
    }

    pub fn ab_search_negamax(
        &self,
        ts:             &Tables,
        mut stats:      &mut SearchStats,
        depth:          Depth,
    ) -> ABResults {

        let mut history = [[[0; 64]; 64]; 2];

        let mut stop_counter = 0;

        let mut cfg = ABConfig::new_depth(depth);
        cfg.root = true;

        let (alpha,beta) = (i32::MIN,i32::MAX);
        let (alpha,beta) = (alpha + 200,beta - 200);

        let mut g = self.game.clone();

        let tt_r = self.tt_r.clone();
        let tt_w = self.tt_w.clone();

        let res = self._ab_search_negamax(
            ts, &mut g, cfg, depth,
            0, &mut stop_counter, (alpha, beta),
            &mut stats,
            VecDeque::new(),
            &mut history,
            &tt_r, tt_w.clone());

        res
    }

    // #[cfg(feature = "nope")]
    // #[allow(unused_doc_comments,unused_labels)]
    // pub fn _ab_search_negamax(
    //     &self,
    //     ts:                      &Tables,
    //     // g:                       &Game,
    //     mut g:                   &mut Game,
    //     mut cfg:                 ABConfig,
    //     depth:                   Depth,
    //     ply:                     Depth,
    //     mut stop_counter:        &mut u16,
    //     (mut alpha, mut beta):   (i32,i32),
    //     mut stats:               &mut SearchStats,
    //     mut prev_mvs:            &mut VecDeque<(Zobrist,Move)>,
    //     mut history:             &mut [[[Score; 64]; 64]; 2],
    //     tt_r:                    &TTRead,
    //     tt_w:                    TTWrite,
    // ) -> ABResults {

    //     let moves = g.search_all(&ts);

    //     /// Score checkmate, stalemate
    //     let mut moves: Vec<Move> = match moves {
    //         Outcome::Checkmate(c) => {
    //             // let score = 100_000_000 - ply as Score;
    //             let score = CHECKMATE_VALUE - ply as Score;
    //             if !tt_r.contains_key(&g.zobrist) {
    //                 stats!(stats.leaves += 1);
    //                 stats!(stats.checkmates += 1);
    //             }

    //             return ABSingle(ABResult::new_empty(-score));

    //         },
    //         Outcome::Stalemate    => {
    //             let score = -STALEMATE_VALUE + ply as Score;
    //             if !tt_r.contains_key(&g.zobrist) {
    //                 stats!(stats.leaves += 1);
    //                 stats!(stats.stalemates += 1);
    //             }
    //             // TODO: adjust stalemate value when winning/losing
    //             return ABSingle(ABResult::new_empty(-score));
    //         },
    //         Outcome::Moves(ms)    => ms,
    //     };

    //     if depth == 0 {
    //         if !tt_r.contains_key(&g.zobrist) {
    //             stats!(stats.leaves += 1);
    //         }

    //         #[cfg(feature = "qsearch")]
    //         let score = {
    //             // trace!("    beginning qsearch, {:?}, a/b: {:?},{:?}",
    //             //        prev_mvs.front().unwrap().1, alpha, beta);
    //             let score = self.qsearch(&ts, &mut g, (ply,0), alpha, beta, &mut stats);
    //             // trace!("    returned from qsearch, score = {}", score);
    //             score
    //         };

    //         #[cfg(not(feature = "qsearch"))]
    //         let score = if g.state.side_to_move == Black {
    //             -g.evaluate(&ts).sum()
    //         } else {
    //             g.evaluate(&ts).sum()
    //         };

    //         return ABSingle(ABResult::new_empty(score));
    //     }

    //     let mut gs: Vec<(Move,Zobrist,Option<(SICanUse,SearchInfo)>)> = moves.into_iter().map(|mv| {
    //         let zb = g.zobrist.update_move_unchecked(ts, &g, mv);
    //         let tt = self.check_tt_negamax(&ts, &zb, depth, &tt_r, &mut stats);
    //         (mv,zb,tt)
    //     }).collect();

    //     let mut node_type = Node::All;

    //     let mut search_pv = true;
    //     let mut skip_pv   = false;

    //     let mut moves_searched = 0;
    //     let mut val = i32::MIN + 200;
    //     let mut val: (Option<(Zobrist,Move,ABResult)>,i32) = (None,val);
    //     let mut list = vec![];

    //     'outer: for (mv,zb,tt) in gs.into_iter() {
    //         g.make_move(ts, mv);

    //         let (a2,b2) = (-beta, -alpha);
    //         let (can_use, res) = match self._ab_search_negamax(
    //             ts,
    //             &mut g,
    //             cfg,
    //             depth,
    //             ply,
    //             &mut stop_counter,
    //             (a2,b2),
    //             &mut stats,
    //             &mut prev_mvs,
    //             &mut history,
    //             tt_r,
    //             tt_w.clone()) {
    //             ABSingle(mut res) | ABSyzygy(mut res) => {
    //                 res.moves.push_front(mv);
    //                 res.neg_score();

    //                 if cfg.root {
    //                     list.push(res.clone());
    //                 }
    //                 (false, res)
    //             },
    //             ABPrune(beta, prune) => {
    //                 continue 'outer;
    //             },
    //             ABList(_, _) => panic!("found ABList when not root?"),
    //             ABNone       => break 'outer,
    //         };

    //         let mut b = false;

    //         if res.score > val.1 {
    //             val.1 = res.score;
    //             // if !can_use { mv_seq.push(*mv) };
    //             val.0 = Some((zb, mv, res.clone()))
    //         }

    //         #[cfg(not(feature = "negamax_only"))]
    //         { if res.score >= beta { // Fail Soft
    //             b = true;
    //             // return beta;
    //         }

    //         if !b && val.1 > alpha {
    //             node_type = Node::PV;
    //             alpha = val.1;
    //             // #[cfg(feature = "pvs_search")]
    //             // if true { search_pv = false; }
    //         }

    //         if b {
    //             // node_type = Some(Node::Cut);
    //             node_type = Node::Cut;

    //             #[cfg(feature = "history_heuristic")]
    //             if !mv.filter_all_captures() {
    //                 history[g.state.side_to_move][mv.sq_from()][mv.sq_to()] += ply as Score * ply as Score;
    //             }

    //             if moves_searched == 0 {
    //                 stats!(stats.beta_cut_first.0 += 1);
    //             } else {
    //                 stats!(stats.beta_cut_first.1 += 1);
    //             }

    //             break;
    //         }}

    //         moves_searched += 1;

    //         g.unmake_move(ts);
    //     }


    //     match &val.0 {
    //         Some((zb,mv,res)) => {
    //             Self::tt_insert_deepest(
    //                 &tt_r, tt_w, g.zobrist,
    //                 SearchInfo::new(
    //                     *mv,
    //                     res.moves.clone().into(),
    //                     depth - 1,
    //                     // depth,
    //                     node_type,
    //                     res.score,
    //                 ));
    //             if cfg.root {
    //                 ABList(res.clone(), list)
    //             } else {
    //                 ABSingle(res.clone())
    //             }
    //         },
    //         _                    => ABNone,
    //     }
    // }

    #[allow(unused_doc_comments,unused_labels)]
    /// alpha: the MIN score that the maximizing player is assured of
    /// beta:  the MAX score that the minimizing player is assured of
    pub fn _ab_search_negamax(
        &self,
        ts:                      &Tables,
        // g:                       &Game,
        g:                       &Game,
        mut cfg:                 ABConfig,
        depth:                   Depth,
        ply:                     Depth,
        mut stop_counter:        &mut u16,
        (mut alpha, mut beta):   (i32,i32),
        mut stats:               &mut SearchStats,
        prev_mvs:                VecDeque<(Zobrist,Move)>,
        mut history:             &mut [[[Score; 64]; 64]; 2],
        tt_r:                    &TTRead,
        tt_w:                    TTWrite,
    ) -> ABResults {

        // trace!("negamax entry, ply {}, a/b = {:>10}/{:>10}", k, alpha, beta);

        /// Repetition checking
        {
            if let Some(k) = g.history.get(&g.zobrist) {
                if *k >= 2 {
                    let score = -STALEMATE_VALUE + ply as Score;
                    return ABSingle(ABResult::new_empty(-score));
                    // return ABSingle(ABResult::new_empty(0));
                }
            }
        }

        // TODO: bench this
        if *stop_counter > 2000 {
            if self.stop.load(SeqCst) {
                return ABNone;
            }
            {
                let r = self.best_mate.read();
                if let Some(best) = *r {
                    drop(r);
                    if best <= cfg.max_depth {
                        trace!("halting search of depth {}, faster mate found", cfg.max_depth);
                        return ABNone;
                    }
                }
            }
            *stop_counter = 0;
        } else {
            *stop_counter += 1;
        }

        let moves = g.search_all(&ts);

        let mut moves: Vec<Move> = match moves {
            Outcome::Checkmate(c) => {
                // let score = 100_000_000 - ply as Score;
                let score = CHECKMATE_VALUE - ply as Score;
                if !tt_r.contains_key(&g.zobrist) {
                    stats!(stats.leaves += 1);
                    stats!(stats.checkmates += 1);
                }

                return ABSingle(ABResult::new_empty(-score));

            },
            Outcome::Stalemate    => {
                let score = -STALEMATE_VALUE + ply as Score;
                if !tt_r.contains_key(&g.zobrist) {
                    stats!(stats.leaves += 1);
                    stats!(stats.stalemates += 1);
                }
                // TODO: adjust stalemate value when winning/losing
                return ABSingle(ABResult::new_empty(-score));
            },
            Outcome::Moves(ms)    => ms,
        };

        if cfg.root {
            moves.retain(|mv| !self.blocked_moves.contains(&mv));
        }

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
            };

            #[cfg(not(feature = "qsearch"))]
            let score = if g.state.side_to_move == Black {
                -g.evaluate(&ts).sum()
            } else {
                g.evaluate(&ts).sum()
            };

            return ABSingle(ABResult::new_empty(score));
        }

        /// Syzygy Probe
        #[cfg(feature = "syzygy")]
        if let Some(tb) = &self.syzygy {
            // let mv = Move::Quiet { from: "E5".into(), to: "F6".into(), pc: King };
            // let score = CHECKMATE_VALUE - (ply as Score + 4);
            // return ABResults::ABSingle(ABResult::new_single(mv, -score));

            match tb.probe_wdl(ts, g) {
                Ok(Wdl::Win) => {
                    // trace!("found WDL win: {:?}", Wdl::Win);
                    match tb.best_move(ts, g) {
                        Ok(Some((mv,dtz)))  => {
                            trace!("dtz,ply = {:?}, {:?}", dtz, ply);
                            // let score = CHECKMATE_VALUE - ply as Score - dtz.0 as Score;
                            let score = CHECKMATE_VALUE - dtz.add_plies(ply as i32).0.abs() as Score;

                            // XXX: wrong, but matches with other wrong mate in x count
                            let score = score + 1;
                            return ABResults::ABSingle(ABResult::new_single(mv, score));
                            // return ABResults::ABSyzygy(ABResult::new_single(mv, score));
                        },
                        Err(e) => {
                        },
                        _ => {
                        },
                    }
                },
                Ok(Wdl::Loss) => {
                    // return ABResults::ABSingle()
                },
                Ok(wdl) => {
                    trace!("found other WDL: {:?}", wdl);
                    // unimplemented!()
                },
                Err(e) => {
                    // unimplemented!()
                }
            }
        }


        #[cfg(feature = "pvs_search")]
        let mut is_pv_node = beta != alpha + 1;
        #[cfg(not(feature = "pvs_search"))]
        let is_pv_node = false;

        /// Null Move pruning
        #[cfg(feature = "null_pruning")]
        if g.state.checkers.is_empty()
            && depth >= 2
            && !is_pv_node
            && g.state.phase < 200
            && cfg.do_null {
                if self.prune_null_move_negamax(
                    ts, g, cfg, depth, ply, alpha, beta, &mut stats,
                    prev_mvs.clone(), &mut history, (tt_r, tt_w.clone())) {

                    // return ABNone;
                    // return ABSingle(ABResult::new_empty(beta));
                    return ABPrune(beta, Prune::NullMove);
                }
        }

        /// MVV LVA move ordering
        order_mvv_lva(&mut moves);

        // moves.sort_by_cached_key(|m| g.static_exchange(&ts, *m));
        // moves.reverse();

        /// History Heuristic ordering
        #[cfg(feature = "history_heuristic")]
        order_moves_history(&history[g.state.side_to_move], &mut moves);

        // /// Make move, Lookup games in Trans Table
        // let mut gs: Vec<(Move,Game,Option<(SICanUse,SearchInfo)>)> = {
        //     let mut gs0 = moves.into_iter()
        //         .flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, m) {
        //             let tt = self.check_tt_negamax(&ts, &g2.zobrist, depth, &tt_r, &mut stats);
        //             Some((m,g2,tt))
        //         } else {
        //             trace!("game not ok? {:?} {:?}", m, g);
        //             None
        //         });
        //     gs0.collect()
        // };

        let mut gs: Vec<(Move,Zobrist,Option<(SICanUse,SearchInfo)>)> = moves.into_iter()
            .map(|mv| {
                let zb = g.zobrist.update_move_unchecked(ts, g, mv);
                let tt = self.check_tt_negamax(&ts, &zb, depth, &tt_r, &mut stats);
                (mv,zb,tt)
            }).collect();

        /// Move Ordering
        order_searchinfo(&mut gs[..]);

        let mut node_type = Node::All;

        let mut search_pv = true;
        let mut skip_pv   = false;

        let mut moves_searched = 0;
        let mut val = i32::MIN + 200;
        let mut val: (Option<(Zobrist,Move,ABResult)>,i32) = (None,val);
        let mut list = vec![];

        'outer: for (mv,zb0,tt) in gs.iter() {
        // 'outer: for (mv,g2,tt) in gs.iter() {

            // let zb = g2.zobrist;

            let g2 = if let Ok(g2) = g.make_move_unchecked(ts, *mv) {
                g2
            } else { continue 'outer; };

            let zb = g2.zobrist;
            assert_eq!(zb, *zb0);
            // if zb != *zb0 {
            //     eprintln!("zb  = {:?}", zb);
            //     eprintln!("zb0 = {:?}", zb0);
            //     panic!("zb != zb0: {:?}\n{:?}", mv, g2);
            // }

            #[cfg(feature = "pvs_search")]
            if depth < 3 {
                skip_pv = true;
            }

            let (can_use,res) = match tt {

                // Some((SICanUse::UseScore,si)) => {
                //     let mut si = si.clone();
                //     // eprintln!("using si: = {:?}", si);
                //     (true, ABResult::new_with(si.moves.into(), si.score))
                // },

                // Some((SICanUse::UseScore,si)) => {
                //     let mut si = si.clone();
                //     match si.node_type {
                //         Node::PV  => {},
                //         Node::All => if si.score <= alpha {
                //             // trace!("Node::All, using alpha {}", alpha);
                //             si.score = alpha;
                //         },
                //         Node::Cut => if si.score >= beta {
                //             // trace!("Node::Cut, using beta {}", beta);
                //             si.score = beta;
                //         },
                //         _         => unimplemented!(),
                //     }
                //     (true, ABResult::new_with(si.moves.into(),si.score))
                // },
                _ => 'search: {
                    let mut pms = prev_mvs.clone();
                    pms.push_back((g.zobrist,*mv));

                    let mut cfg2 = cfg;
                    cfg2.do_null = true;
                    cfg2.root    = false;

                    let mut lmr = true;
                    let mut depth2 = depth - 1;

                    if mv.filter_all_captures() {
                        let see = g2.static_exchange(&ts, *mv).unwrap();
                        /// Capture with good SEE: do not reduce
                        if see > 0 {
                            lmr = false;
                        }
                    }

                    /// not reducing when in check replaces check extension
                    #[cfg(feature = "late_move_reduction")]
                    if lmr
                        && !is_pv_node
                        && moves_searched >= LMR_MIN_MOVES
                        // && ply >= LMR_MIN_PLY
                        && depth >= 3
                        && !mv.filter_promotion()
                        && g.state.checkers.is_empty()
                        && g2.state.checkers.is_empty()
                    {

                        // let depth3 = depth.checked_sub(LMR_REDUCTION).unwrap();
                        let depth3 = depth.checked_sub(3).unwrap();

                        // let depth3 = if moves_searched < 2 {
                        //     depth.checked_sub(1).unwrap()
                        // } else if moves_searched < 4 {
                        //     depth.checked_sub(2).unwrap()
                        // } else {
                        //     // let k = Depth::max(1, depth)
                        //     let k = 3;
                        //     depth.checked_sub(k).unwrap()
                        //     // depth.checked_sub(depth / 3).unwrap()
                        // };

                        // let depth3 = if ply < LMR_PLY_CONST {
                        //     // depth - LMR_REDUCTION
                        //     depth.checked_sub(1).unwrap()
                        // } else {
                        //     depth.checked_sub(depth / 3).unwrap()
                        // };

                        match self._ab_search_negamax(
                            ts, &g2, cfg2, depth3, ply + 1, &mut stop_counter,
                            (-beta, -alpha), &mut stats,
                            pms.clone(), &mut history, tt_r, tt_w.clone(),
                            // false,
                            // // XXX: No Null pruning inside reduced depth search ?
                            // true
                        ) {
                            ABSingle(mut res) | ABSyzygy(mut res) => {
                                res.neg_score();
                                if res.score <= alpha {
                                    stats!(stats.lmrs.0 += 1);
                                    res.moves.push_front(*mv);
                                    break 'search (false,res);
                                }
                            },
                            ABList(_, _) => panic!("found ABList when not root?"),
                            ABPrune(beta, prune) => {
                                // panic!("ABPrune 0");
                                // trace!("ABPrune 0: {:?} {:?}", beta, prune);
                            },
                            ABNone       => {},
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
                        ts, &g2, cfg2, depth2, ply + 1, &mut stop_counter,
                        (a2, b2), &mut stats,
                        pms.clone(), &mut history, tt_r, tt_w.clone()) {
                        ABSingle(mut res) | ABSyzygy(mut res) => {
                            res.moves.push_front(*mv);
                            res.neg_score();

                            #[cfg(feature = "pvs_search")]
                            if !search_pv && res.score > alpha {
                                match self._ab_search_negamax(
                                    ts, &g2, cfg2, depth2, ply + 1, &mut stop_counter,
                                    (-beta, -alpha), &mut stats,
                                    pms, &mut history, tt_r, tt_w.clone()) {
                                    ABSingle(mut res2) | ABSyzygy(mut res2) => {
                                        res2.neg_score();
                                        res2.moves.push_front(*mv);
                                        res = res2;
                                    },
                                    // ABList(_, _) => break 'outer,
                                    ABList(_, _) => panic!("found ABList when not root?"),
                                    ABPrune(beta, prune) => {
                                        panic!("ABPrune 1");
                                    },
                                    ABNone       => break 'outer,
                                }
                            }

                            if cfg.root {
                                list.push(res.clone());
                            }
                            (false, res)
                        },
                        ABPrune(beta, prune) => {
                            // panic!("ABPrune 2");
                            // trace!("ABPrune 2: {:?} {:?}", beta, prune);
                            continue 'outer;
                        },
                        // ABList(_, _) => break 'outer,
                        ABList(_, _) => panic!("found ABList when not root?"),
                        ABNone       => break 'outer,
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

                if !cfg.inside_null {
                    // trace!("inserting TT, zb = {:?}", g.zobrist);
                    Explorer::tt_insert_deepest(
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
                }

                // match node_type {
                //     None => {},
                //     Some(nt) => {
                //     }
                // }

                if cfg.root {
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


