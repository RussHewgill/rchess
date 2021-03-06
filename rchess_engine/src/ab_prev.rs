
use crate::movegen::*;
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;
use crate::explore::*;
#[cfg(feature = "syzygy")]
use crate::syzygy::{SyzygyTB, Wdl, Dtz};
use crate::alphabeta::*;

use crate::alphabeta::{ABNodeType::*,ABResults::*};

use std::sync::atomic::Ordering::{SeqCst,Relaxed};


/// Negamax AB
impl ExHelper {


    /// Steps:
    ///   0:  Check for repetition
    ///   1:  Check for Stop if mate found
    ///   2:  Generate Moves
    ///   3:  Check for Checkmate, Stalemate
    ///   4:  Qsearch if depth == 0
    ///   5:  Syzygy probe
    ///   6:  Null Pruning (off)
    ///   7:  TransTable Lookup for each move
    ///   8:  Move Ordering
    ///   Loop over moves:
    ///     9:  Futility Pruning
    ///     10: Check if TT Score can be used, else:

    #[allow(unused_doc_comments,unused_labels)]
    /// alpha: the MIN score that the maximizing player is assured of
    /// beta:  the MAX score that the minimizing player is assured of
    pub fn _ab_search_negamax(
        &self,
        ts:                      &'static Tables,
        g:                       &Game,
        mut cfg:                 ABConfig,
        depth:                   Depth,
        ply:                     Depth,
        mut stop_counter:        &mut u16,
        (mut alpha, mut beta):   (Score,Score),
        mut stats:               &mut SearchStats,
        mut stack:               &mut ABStack,
        node_type:               ABNodeType,
    ) -> ABResults {
        use crate::alphabeta::ABNodeType::*;

        // trace!("negamax entry, ply {}, a/b = {:>10}/{:>10}", k, alpha, beta);

        let is_pv: bool   = node_type != ABNodeType::NonPV;
        let is_root: bool = node_type == ABNodeType::Root;

        if self.stop.load(SeqCst) {
            return ABNone;
        }
        {
            let r = self.best_mate.read();
            if let Some(best) = *r {
                trace!("halting search of depth {}, mate found", cfg.max_depth);
                return ABNone;
                // drop(r);
                // if best <= cfg.max_depth {
                //     trace!("halting search of depth {}, faster mate found", cfg.max_depth);
                //     return ABNone;
                // }
            }
        }

        // let moves = g.search_all(&ts);

        let moves = if is_root {
            if let Some(mvs) = &self.cfg.only_moves {
                let mvs = mvs.clone().into_iter().collect();
                Outcome::Moves(mvs)
            } else {
                g.search_all(&ts)
            }
        } else {
            g.search_all(&ts)
        };

        /// Filter checkmate, stalemate
        let mut moves: Vec<Move> = match moves {
            Outcome::Checkmate(c) => {
                // let score = 100_000_000 - ply as Score;
                let score = CHECKMATE_VALUE - ply as Score;
                // if !self.tt_r.contains_key(&g.zobrist) {
                // }
                stats.leaves += 1;
                stats.checkmates += 1;

                let mv = g.last_move.unwrap();

                // return ABSingle(ABResult::new_empty(-score));
                return ABSingle(ABResult::new_single(mv, -score));

            },
            Outcome::Stalemate    => {
                let score = -STALEMATE_VALUE + ply as Score;
                // if !self.tt_r.contains_key(&g.zobrist) {
                //     stats!(stats.leaves += 1);
                //     stats!(stats.stalemates += 1);
                // }
                stats.leaves += 1;
                stats.stalemates += 1;

                // let mv = g.last_move.unwrap();
                if let Some(mv) = g.last_move {
                    // TODO: adjust stalemate value when winning/losing
                    // return ABSingle(ABResult::new_empty(-score));
                    // return ABSingle(ABResult::new_single(mv, score));
                    return ABSingle(ABResult::new_single(mv, 0));
                } else {
                    return ABNone
                }
            },
            Outcome::Moves(ms)    => ms,
        };

        /// Filter blocked moves
        if is_root {
            moves.retain(|mv| !self.cfg.blocked_moves.contains(&mv));
        }

        /// Enter Qsearch
        if depth == 0 {
            // if !self.tt_r.contains_key(&g.zobrist) {
            // }
            stats.leaves += 1;

            #[cfg(feature = "qsearch")]
            let score = {
                // trace!("    beginning qsearch, {:?}, a/b: {:?},{:?}",
                //        prev_mvs.front().unwrap().1, alpha, beta);
                let nt = if node_type == PV { PV } else { NonPV };
                let score = self.qsearch(&ts, &g, (ply,0), (alpha, beta), stack, stats, nt);
                // trace!("    returned from qsearch, score = {}", score);
                score
            };

            #[cfg(not(feature = "qsearch"))]
            let score = if g.state.side_to_move == Black {
                -g.evaluate(&ts).sum()
            } else {
                g.evaluate(&ts).sum()
            };

            // return ABSingle(ABResult::new_empty(score));
            return ABSingle(ABResult::new_single(g.last_move.unwrap(), score));
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
                            // trace!("dtz,ply = {:?}, {:?}", dtz, ply);
                            // let score = CHECKMATE_VALUE - ply as Score - dtz.0 as Score;
                            let score = CHECKMATE_VALUE - dtz.add_plies(ply as i32).0.abs() as Score;

                            // XXX: wrong, but matches with other wrong mate in x count
                            let score = score + 1;
                            // return ABResults::ABSingle(ABResult::new_single(mv, score));
                            return ABResults::ABSyzygy(ABResult::new_single(mv, score));
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
            && depth >= NULL_PRUNE_MIN_DEPTH
            && !is_pv_node
            && g.state.phase < 200
            && cfg.do_null {
                if self.prune_null_move_negamax(
                    ts, g, cfg, depth, ply, (alpha, beta), &mut stats,
                    &mut stack) {

                    // return ABNone;
                    // return ABSingle(ABResult::new_empty(beta));
                    return ABPrune(beta, Prune::NullMove);
                }
        }

        // /// History Heuristic ordering
        // #[cfg(feature = "history_heuristic")]
        // order_moves_history(&tracking.history[g.state.side_to_move], &mut moves);

        // let mut gs: Vec<(Move,Zobrist,Option<(SICanUse,SearchInfo)>)> = moves.into_iter()
        // // let mut gs: ArrayVec<(Move,Zobrist,Option<(SICanUse,SearchInfo)>),256> = moves.into_iter()
        //     .map(|mv| {
        //         let zb = g.zobrist.update_move_unchecked(ts, g, mv);
        //         let tt = self.check_tt_negamax(&ts, &zb, depth, &mut stats);
        //         (mv,zb,tt)
        //     }).collect();

        /// No change in performance, but easier to read in flamegraph
        let mut gs: Vec<(Move,Zobrist,Option<(SICanUse,SearchInfo)>)> = Vec::with_capacity(moves.len());
        for mv in moves.into_iter() {
            let zb = g.zobrist.update_move_unchecked(ts, g, mv);
            let tt = self.check_tt_negamax(&ts, zb, depth, &mut stats);
            gs.push((mv,zb,tt));
        }

        self.order_moves(ts, g, ply, &mut stack, &mut gs[..]);

        let mut node_type = Node::All;

        let mut search_pv = true;
        let mut skip_pv   = false;

        #[cfg(feature = "futility_pruning")]
        // let can_futility_prune = if depth <= 3
        let can_futility_prune = if depth == 1
            && !is_pv_node
            && g.state.checkers.is_empty()
            && alpha < STALEMATE_VALUE - 100 {
                let static_eval = self.cfg.evaluate(ts, g, &self.ph_rw);
                // static_eval + (FUTILITY_MARGIN * depth as Score) <= alpha
                static_eval + FUTILITY_MARGIN <= alpha
            } else { false };

        let mut moves_searched = 0;
        let mut val = Score::MIN + 200;
        let mut val: (Option<(Zobrist,ABResult,bool)>,Score) = (None,val);
        let mut list = vec![];

        // #[cfg(feature = "nope")]
        'outer: for (mv,zb0,tt) in gs.into_iter() {

            // match g.get_at(mv.sq_from()) {
            //     None => {
            //         eprintln!("ab_search 0: non legal move, no piece?: {:?}\n{:?}\n{:?}",
            //                   mv, g.to_fen(), g);
            //         panic!();
            //     },
            //     _ => {},
            // }

            // if self.best_mate.read().is_some() {
            //     trace!("halting {}, mate", cfg.max_depth);
            //     return ABNone;
            // }

            // let g2 = if let Ok(g2) = g._make_move_unchecked(ts, mv, Some(zb0)) {
            //     g2
            // } else { continue 'outer; };

            // // XXX: temp
            // let k0 = {
            //     let nn = &self.nnue.as_ref().unwrap();
            //     let nn = nn.borrow();
            //     // nn.ft.accum.stack_delta.len()
            //     nn.ft.accum.make_copy()
            // };

            let g2 = if let Some(g2) = self.make_move(ts, g, mv, Some(zb0), stack) {
                g2
            } else { continue 'outer; };

            #[cfg(feature = "futility_pruning")]
            if moves_searched > 0 && can_futility_prune {
            // if can_futility_prune {
                if g2.state.checkers.is_empty()
                    && !mv.filter_all_captures()
                    && !mv.filter_promotion() {
                        stats.fut_prunes += 1;
                        self.pop_nnue();
                        continue;
                    }
            }

            let zb = g2.zobrist;
            assert_eq!(zb, zb0);

            #[cfg(feature = "pvs_search")]
            if depth < 3 {
                skip_pv = true;
            }

            let (from_tt,res) = match tt {

                Some((SICanUse::UseScore,si)) => {
                    let mut si = si.clone();
                    // match si.node_type {
                    //     Node::PV  => {},
                    //     Node::All => if si.score <= alpha {
                    //         // trace!("Node::All, using alpha {}", alpha);
                    //         si.score = alpha;
                    //     },
                    //     Node::Cut => if si.score >= beta {
                    //         // trace!("Node::Cut, using beta {}", beta);
                    //         si.score = beta;
                    //     },
                    //     _         => unimplemented!(),
                    // }
                    // (true, ABResult::new_single(si.best_move, si.score))
                    (true, ABResult::new_single(mv, si.score))
                },

                _ => 'search: {

                    let mut cfg2 = cfg;
                    cfg2.do_null = true;

                    let mut lmr = true;
                    let mut depth2 = depth - 1;

                    #[cfg(feature = "late_move_reduction")]
                    if mv.filter_all_captures() {
                        let see = g2.static_exchange(&ts, mv).unwrap();
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
                        && depth >= LMR_MIN_DEPTH
                        && !mv.filter_promotion()
                        && g.state.checkers.is_empty()
                        && g2.state.checkers.is_empty()
                    {

                        let depth3 = depth.checked_sub(LMR_REDUCTION).unwrap();
                        // let depth3 = depth.checked_sub(3).unwrap();

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
                            // pms.clone(), &mut history, tt_r, tt_w.clone(),
                            &mut stack, NonPV
                        ) {
                            ABSingle(mut res) | ABSyzygy(mut res) => {
                                res.neg_score(mv);
                                if res.score <= alpha {
                                    stats!(stats.lmrs.0 += 1);
                                    // res.moves.push_front(*mv);
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
                        &mut stack,
                        NonPV
                    ) {
                        ABSingle(mut res) | ABSyzygy(mut res) => {
                            res.neg_score(mv);

                            #[cfg(feature = "pvs_search")]
                            if !search_pv && res.score > alpha {
                                match self._ab_search_negamax(
                                    ts, &g2, cfg2, depth2, ply + 1, &mut stop_counter,
                                    (-beta, -alpha), &mut stats,
                                    &mut stack,
                                    NonPV,
                                ) {
                                    ABSingle(mut res2) | ABSyzygy(mut res2) => {
                                        res2.neg_score(mv);
                                        res = res2;
                                    },
                                    // ABList(_, _) => break 'outer,
                                    ABList(_, _) => panic!("found ABList when not root?"),
                                    ABPrune(beta, prune) => {
                                        panic!("ABPrune 1");
                                    },
                                    ABNone       => {
                                        self.pop_nnue(stack);
                                        break 'outer;
                                    }
                                }
                            }

                            if is_root {
                                list.push(res.clone());
                            }
                            (false, res)
                        },
                        ABPrune(beta, prune) => {
                            // panic!("ABPrune 2");
                            // trace!("ABPrune 2: {:?} {:?}", beta, prune);
                            self.pop_nnue(stack);
                            continue 'outer;
                        },
                        // ABList(_, _) => break 'outer,
                        ABList(_, _) => panic!("found ABList when not root?"),
                        ABNone       => {
                            self.pop_nnue(stack);
                            break 'outer;
                        }
                    }

                },
            };
            let mut b = false;

            if res.score > val.1 {
                val.1 = res.score;
                val.0 = Some((zb, res.clone(),from_tt))
            }

            #[cfg(not(feature = "negamax_only"))]
            {
                if res.score >= beta { // Fail Soft
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
                        // stack.history[g.state.side_to_move][mv.sq_from()][mv.sq_to()] +=
                        //     ply as Score * ply as Score;
                        unimplemented!()
                    }

                    // #[cfg(feature = "killer_moves")]
                    // if !mv.filter_all_captures() {
                    //     // tracking.killers.increment(g.state.side_to_move, ply, &mv);
                    //     stack.killers.store(g.state.side_to_move, ply, mv);
                    // }

                    if moves_searched == 0 {
                        stats!(stats.beta_cut_first.0 += 1);
                    } else {
                        stats!(stats.beta_cut_first.1 += 1);
                    }

                    self.pop_nnue(stack);
                    break;
                }
            }

            self.pop_nnue(stack);

            // if let Some(nnue) = &self.nnue {
            //     let nn = nnue.borrow();
            //     // let k1 = nn.ft.accum.stack_delta.len();
            //     let k1 = nn.ft.accum.make_copy();
            //     if k0 != k1 {
            //         eprintln!("g.to_fen() = {:?}", g.to_fen());
            //         eprintln!("g = {:?}", g);
            //         eprintln!("(k0,k1) = {:?}", (k0,k1));
            //         panic!();
            //     }
            // }

            moves_searched += 1;
        }

        // if root && k > 5 {
        //     trace!("node_type = {:?}", node_type);
        // }

        /// XXX: stat padding by including nodes found in TT
        stats!(stats.inc_nodes_arr(ply));
        stats!(stats.nodes += 1);

        match &val.0 {
            // Some((zb,mv,mv_seq)) => Some(((mv_seq.clone(),val.1),(alpha,beta))),
            // Some((zb,mv,res)) => {
            Some((zb,res,from_tt)) => {

                if !cfg.inside_null && !from_tt {
                // if !from_tt {
                    // trace!("inserting TT, zb = {:?}", g.zobrist);
                    self.tt_insert_deepest(
                        g.zobrist,
                        SearchInfo::new(
                            // ts, g,
                            res.mv,
                            // res.moves.clone().into(),
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

                // let mut res2 = res.clone();
                // res2.mv = *mv;

                if is_root {
                    ABList(*res, list)
                } else {
                    ABSingle(*res)
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
