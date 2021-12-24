
// use crate::heuristics::*;
use crate::movegen::*;
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;
use crate::explore::*;
#[cfg(feature = "syzygy")]
use crate::syzygy::{SyzygyTB, Wdl, Dtz};

use crate::stats;

use ABResults::*;
use arrayvec::ArrayVec;

use std::sync::atomic::Ordering::{SeqCst,Relaxed};

#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub struct ABResult {
    pub mv:       Move,
    pub score:    Score,
}

impl std::ops::Neg for ABResult {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let mut out = self;
        out.score = -self.score;
        out
    }
}

impl ABResult {

    pub fn new_null() -> Self {
        Self {
            mv: Move::NullMove,
            // score: 0,
            score: i32::MIN,
        }
    }

    pub fn new_single(mv: Move, score: Score) -> Self {
        Self {
            mv: mv,
            score,
        }
    }

    pub fn neg_score(&mut self, mv: Move) {
        self.score = -self.score;
        self.mv = mv;
    }

}

#[derive(Debug,PartialEq,PartialOrd,Clone)]
pub enum ABResults {
    ABSingle(ABResult),
    ABList(ABResult, Vec<ABResult>),
    ABSyzygy(ABResult),
    // ABMate()
    ABPrune(Score, Prune),
    ABNone,
}

impl std::ops::Neg for ABResults {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            ABResults::ABSingle(res)     => ABResults::ABSingle(-res),
            ABResults::ABList(res,vs)    => ABResults::ABList(-res,vs),
            ABResults::ABSyzygy(res)     => ABResults::ABSyzygy(-res),
            ABResults::ABPrune(score, p) => ABResults::ABPrune(-score, p),
            _                            => self,
        }
    }
}

impl ABResults {

    pub fn get_result_mv(&self, mv: Move) -> Option<ABResult> {
        let mut res = self.get_result()?;
        res.mv = mv;
        Some(res)
    }

    pub fn get_result(&self) -> Option<ABResult> {
        match self {
            Self::ABSingle(res)  => Some(*res),
            Self::ABList(res, _) => Some(*res),
            Self::ABSyzygy(res)  => Some(*res),
            Self::ABPrune(_, _)  => None,
            Self::ABNone         => None,
        }
    }

    pub fn get_scores(&self) -> Option<Vec<ABResult>> {
        match self {
            Self::ABList(_, scores) => Some(scores.clone()),
            _                       => None,
        }
    }

}

#[derive(Debug,PartialEq,PartialOrd,Clone)]
pub enum Prune {
    NullMove,
    Futility,
}

#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub enum ABNodeType {
    Root,
    PV,
    NonPV,
}

/// Make move, increment NNUE
impl ExHelper {

    // #[inline(always)]
    pub fn make_move(
        &self,
        ts:           &Tables,
        g:            &Game,
        mv:           Move,
        zb0:          Option<Zobrist>,
        mut stack:    &mut ABStack,
    ) -> Option<Game> {
        if let Ok(g2) = g._make_move_unchecked(ts, mv, zb0) {

            // push NNUE
            #[cfg(feature = "nnue")]
            if let Some(nnue) = &self.nnue {
                let mut nn = nnue.borrow_mut();
                nn.ft.make_move(&g2, mv);
            }

            stack.move_history.push((g.zobrist,mv));

            Some(g2)
        } else { None }
    }

    // #[inline(always)]
    pub fn pop_nnue(&self, mut stack: &mut ABStack) {
        #[cfg(feature = "nnue")]
        if let Some(nnue) = &self.nnue {
            let mut nn = nnue.borrow_mut();
            nn.ft.accum_pop();
        }
        stack.move_history.pop();
    }

}

/// search_single
impl ExHelper {
    pub fn ab_search_single(
        &self,
        ts:             &'static Tables,
        mut stats:      &mut SearchStats,
        mut stack:      &mut ABStack,
        depth:          Depth,
    ) -> ABResults {

        let (alpha,beta) = (Score::MIN,Score::MAX);
        let (alpha,beta) = (alpha + 200,beta - 200);

        let mut g = self.game.clone();

        #[cfg(not(feature = "new_search"))]
        let res = {
            let mut stop_counter = 0;
            let mut cfg = ABConfig::new_depth(depth);
            self._ab_search_negamax(
                ts, &mut g, cfg, depth,
                0, &mut stop_counter, (alpha, beta),
                &mut stats,
                &mut stack,
                ABNodeType::Root,
            )
        };

        // #[cfg(feature = "new_search")]
        // let res = self._ab_search_negamax2(
        //     ts,
        //     &g,
        //     (depth,0),
        //     (alpha,beta),
        //     stats,
        //     stack,
        //     ABNodeType::Root,
        //     false);

        #[cfg(feature = "new_search")]
        let res = self._ab_search_negamax2::<{ABNodeType::Root}>(
            ts,
            &g,
            (depth,0),
            (alpha,beta),
            stats,
            stack,
            false);

        res
    }
}

/// TT Probe
impl ExHelper {

    /// returns (can_use, SearchInfo)
    #[cfg(not(feature = "lockless_hashmap"))]
    pub fn check_tt_negamax(
        &self,
        ts:             &Tables,
        zb:             Zobrist,
        depth:          Depth,
        mut stats:      &mut SearchStats,
    ) -> Option<(SICanUse,SearchInfo)> {
        // if let Some(si) = tt_r.get_one(&g.zobrist) {
        if let Some(si) = self.tt_r.get_one(&zb) {
            if si.depth_searched >= depth {
                stats!(stats.tt_hits += 1);
                Some((SICanUse::UseScore,*si))
            } else {
                stats!(stats.tt_halfmiss += 1);
                Some((SICanUse::UseOrdering,*si))
            }
        } else {
            // if g.zobrist == Zobrist(0x1eebfbac03c62e9d) { println!("wat wat 3"); }
            stats!(stats.tt_misses += 1);
            // let score = self._ab_search(&ts, &g, depth - 1, k + 1, alpha, beta, !maximizing);
            // score
            None
        }
    }

    #[cfg(feature = "lockless_hashmap")]
    pub fn check_tt_negamax(
        &self,
        ts:             &Tables,
        zb:             Zobrist,
        depth:          Depth,
        mut stats:      &mut SearchStats,
    ) -> Option<(SICanUse,SearchInfo)> {
        // if let Some(si) = tt_r.get_one(&g.zobrist) {

        if let Some(si) = self.ptr_tt.probe(zb) {
        // if let Some(si) = self.tt_r.get_one(zb) {
            if si.depth_searched >= depth {
                stats!(stats.tt_hits += 1);
                Some((SICanUse::UseScore,*si))
            } else {
                stats!(stats.tt_halfmiss += 1);
                Some((SICanUse::UseOrdering,*si))
            }
        } else {
            // if g.zobrist == Zobrist(0x1eebfbac03c62e9d) { println!("wat wat 3"); }
            stats!(stats.tt_misses += 1);
            // let score = self._ab_search(&ts, &g, depth - 1, k + 1, alpha, beta, !maximizing);
            // score
            None
        }

    }

    pub fn check_tt2(
        &self,
        ts:             &Tables,
        zb:             Zobrist,
        depth:          Depth,
        mut stats:      &mut SearchStats,
    ) -> Option<SearchInfo> {
        #[cfg(feature = "lockless_hashmap")]
        if let Some(si) = self.ptr_tt.probe(zb) {
            if si.depth_searched >= depth {
                stats!(stats.tt_hits += 1);
            } else {
                stats!(stats.tt_halfmiss += 1);
            }
            Some(*si)
        } else {
            stats!(stats.tt_misses += 1);
            None
        }
        #[cfg(not(feature = "lockless_hashmap"))]
        if let Some(si) = self.tt_r.get_one(&zb) {
            if si.depth_searched >= depth {
                stats!(stats.tt_hits += 1);
            } else {
                stats!(stats.tt_halfmiss += 1);
            }
            Some(*si)
        } else {
            stats!(stats.tt_misses += 1);
            None
        }
    }

}

/// Negamax AB Refactor
impl ExHelper {
    #[allow(unused_doc_comments,unused_labels)]
    pub fn _ab_search_negamax2<const NODE_TYPE: ABNodeType>(
        &self,
        ts:                      &'static Tables,
        g:                       &Game,
        (depth,ply):             (Depth,Depth),
        (mut alpha, mut beta):   (Score,Score),
        mut stats:               &mut SearchStats,
        mut stack:               &mut ABStack,
        is_cut_node:             bool,
    ) -> ABResults {
        use ABNodeType::*;

        // trace!("negamax entry, ply {}, a/b = {:>10}/{:>10}", k, alpha, beta);

        // let mut current_stack: &mut ABStackPly = stack.get_or_push(ply);
        stack.push_if_empty(g, ply);

        #[cfg(feature = "pvs_search")]
        let mut is_pv_node = NODE_TYPE != NonPV;
        #[cfg(not(feature = "pvs_search"))]
        let is_pv_node = false;

        // #[cfg(feature = "pvs_search")]
        // let is_pv_node = beta != alpha + 1;

        let is_root_node: bool = NODE_TYPE == Root;

        let mut current_node_type = Node::All;

        /// Misc assertions
        #[cfg(feature = "pvs_search")]
        {
            assert!(is_pv_node || (alpha == beta - 1));
            assert!(!(is_pv_node && is_cut_node));
        }

        /// Repetition, Halting
        if !is_root_node {

            /// Repetition checking
            if alpha < DRAW_VALUE {
                // for (zb,_) in stack.move_history.iter().step_by(2) {

                let cycle = stack.move_history.iter().any(|&(zb2,_)| g.zobrist == zb2);
                // let cycle = stack.move_history.contains(&zb2);
                if cycle && alpha >= beta {
                    return ABSingle(ABResult::new_single(g.last_move.unwrap(), 0));
                }

            }

            /// Halted search
            if self.stop.load(std::sync::atomic::Ordering::Relaxed) {
                return ABNone;
            }

            /// Mate found
            {
                let r = self.best_mate.read();
                if let Some(best) = *r {
                    trace!("halting search, mate found");
                    return ABNone;
                }
            }

        }

        /// Qsearch
        if depth == 0 {
            // if !self.tt_r.contains_key(&g.zobrist) {
            // }
            stats.leaves += 1;

            #[cfg(feature = "qsearch")]
            let score = {
                // trace!("    beginning qsearch, {:?}, a/b: {:?},{:?}",
                //        prev_mvs.front().unwrap().1, alpha, beta);

                // let nt: ABNodeType = if NODE_TYPE == PV { PV } else { NonPV };
                // let score = self.qsearch(&ts, &g, (ply,0), (alpha, beta), stack, stats, nt);
                // // let score = self.qsearch::<{NT}>(&ts, &g, (ply,0), (alpha, beta), stack, stats);

                let score = if NODE_TYPE == PV {
                    self.qsearch::<{PV}>(&ts, &g, (ply,0), (alpha, beta), stack, stats)
                    // self.qsearch2::<{PV}>(&ts, &g, (ply,0), (alpha, beta), stack, stats)
                } else {
                    self.qsearch::<{PV}>(&ts, &g, (ply,0), (alpha, beta), stack, stats)
                    // self.qsearch2::<{PV}>(&ts, &g, (ply,0), (alpha, beta), stack, stats)
                };

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

        let msi: Option<SearchInfo> = if is_root_node { None } else {
            self.check_tt2(ts, g.zobrist, depth, stats)
        };

        /// Check for returnable TT score
        if let Some(si) = msi {
            if !is_pv_node && si.depth_searched >= depth { // XXX: depth or depth-1 ??
                return ABResults::ABSingle(ABResult::new_single(g.last_move.unwrap(), si.score));
            }
        }

        /// Syzygy Probe
        #[cfg(feature = "syzygy")]
        if let Some(tb) = &self.syzygy {
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

        /// when in check, skip early pruning
        let in_check = g.state.checkers.is_not_empty();

        // let static_eval = if in_check {
        //     None
        // } else if let Some(nnue) = &self.nnue {
        //     let mut nn = nnue.borrow_mut();
        //     let score = nn.evaluate(&g, true);
        //     stack.with(ply, |st| st.static_eval = Some(score));
        //     Some(score)
        // } else {
        //     let stand_pat = self.cfg.evaluate(ts, g, &self.ph_rw);
        //     let score = if g.state.side_to_move == Black { -stand_pat } else { stand_pat };
        //     stack.with(ply, |st| st.static_eval = Some(score));
        //     Some(score)
        // };

        // // let mut can
        // // TODO: futility pruning
        // if depth == 1
        //     && !is_pv_node
        //     && !in_check
        //     // && static_eval - FUTILITY_MARGIN >= beta
        //     && static_eval + FUTILITY_MARGIN <= alpha
        // {
        //     stats.fut_prunes += 1;
        //     return ABPrune(static_eval, Prune::Futility);
        // }

        /// null move pruning
        #[cfg(feature = "null_pruning")]
        if !is_pv_node
            // && !stack.inside_null
            && g.last_move != Some(Move::NullMove)
            && depth >= NULL_PRUNE_MIN_DEPTH
            && !in_check
            && g.state.phase < NULL_PRUNE_MIN_PHASE
        {
            let r = NULL_PRUNE_REDUCTION; // 2

            assert!(depth - 1 >= r);

            let null_depth = depth - 1 - r;

            // if let Ok(g2) = g.make_move_unchecked(ts, Move::NullMove) {
            if let Some(g2) = self.make_move(ts, g, Move::NullMove, None, stack) {

                // stack.inside_null = true;
                let res = -self._ab_search_negamax2::<{NonPV}>(
                    ts, &g2, (null_depth,ply+1), (-beta,-beta+1), stats, stack, !is_cut_node);
                // stack.inside_null = false;

                if let Some(res) = res.get_result() {
                    if res.score >= beta {
                        stats!(stats.null_prunes += 1);
                        return ABPrune(beta, Prune::NullMove);
                    }
                }

                self.pop_nnue(stack);
            }

        }

        let m_hashmove: Option<Move> = msi.map(|si| {
            let mv = si.best_move;
            let mv = PackedMove::unpack(&[mv.0,mv.1]).unwrap().convert_to_move(ts, g);
            mv
        });
        let mut movegen = MoveGen::new(ts, &g, m_hashmove, stack, depth, ply);
        // let mut movegen = MoveGen::new_all(ts, &g, m_hashmove, depth, ply);

        // self.order_moves(ts, g, ply, &mut stack, &mut gs[..]);

        // let mut do_pvs = false;
        /// true until a move is found that raises alpha
        let mut search_pvs_all = true;

        // #[cfg(feature = "pvs_search")]
        // if depth < 3 {
        //     // do_pvs = false;
        //     search_pvs_all = false;
        // }

        let mut moves_searched = 0;
        let mut val = Score::MIN + 200;
        let mut best_val: (Option<(Zobrist,ABResult)>,Score) = (None,val);
        let mut list = vec![];

        /// Loop over moves
        'outer: while let Some(mv) = movegen.next(&stack) {

            let mut next_depth = depth - 1;
            let mut extensions = 0;

            /// Prefetch hash table bucket
            let zb0 = g.zobrist.update_move_unchecked(ts, g, mv);
            self.ptr_tt.prefetch(zb0);

            /// Skip blocked moves
            if is_root_node && self.cfg.blocked_moves.contains(&mv) {
                continue 'outer;
            }

            let capture_or_promotion = mv.filter_all_captures() || mv.filter_promotion();
            // let gives_check = movegen.gives_check(mv);

            // TODO: 
            /// Shallow pruning
            if !is_root_node {
            }

            // TODO: 
            /// Singular extension
            if !is_root_node {
            }

            let g2 = if let Some(g2) = self.make_move(ts, g, mv, Some(zb0), stack) {
                g2
            } else { continue 'outer; };
            moves_searched += 1;

            next_depth += extensions;

            let res: ABResult = 'search: {
                let mut res: ABResult = ABResult::new_null();

                let mut lmr = self.cfg.late_move_reductions;

                let mut do_full_depth = true; // XXX: ??

                if lmr && mv.filter_all_captures() {
                    // let see = g.static_exchange(&ts, mv).unwrap(); // XXX: g or g2?
                    let see = movegen.static_exchange(mv).unwrap();
                    /// Capture with good SEE: do not reduce
                    if see > 0 {
                        lmr = false;
                    }
                }

                /// Late Move Reductions
                if lmr
                    && !is_pv_node
                    && moves_searched >= (if is_root_node { 2 + LMR_MIN_MOVES } else { LMR_MIN_MOVES })
                    && next_depth >= LMR_MIN_DEPTH
                    && !mv.filter_promotion()
                    // && !mv.filter_all_captures()
                    && !in_check
                    && g2.state.checkers.is_empty()
                {

                    // let depth_r = next_depth.checked_sub(LMR_REDUCTION).unwrap();
                    // // let depth_r = next_depth.checked_sub(1).unwrap();

                    let r = lmr_reduction(next_depth, moves_searched);
                    let depth_r = next_depth.checked_sub(r).unwrap();

                    // let (a2,b2) = (-beta,-alpha);
                    let (a2,b2) = (-(alpha+1),-alpha); // XXX: ??

                    // trace!("search 0");
                    let res2 = -self._ab_search_negamax2::<{NonPV}>(
                        ts, &g2, (depth_r,ply+1), (a2,b2), stats, stack, true);
                    res = if let Some(mut r) = res2.get_result_mv(mv) { r } else {
                        self.pop_nnue(stack);
                        continue 'outer;
                    };
                    if res.score <= alpha {
                        stats!(stats.lmrs.0 += 1);
                        break 'search res;
                    }

                    /// if LMR failed high
                    if res.score > alpha {
                        do_full_depth = true;
                    }

                } else {
                    // do_full_depth = is_root_node || !is_pv_node || moves_searched > 1;
                    do_full_depth = !is_pv_node || moves_searched > 1;
                }
                // #[cfg(not(feature = "late_move_reduction"))]
                // { do_full_depth = true; }

                ///   Full depth search
                /// If LMR failed or was skipped
                if do_full_depth && (!search_pvs_all || !is_pv_node) {

                    #[cfg(feature = "pvs_search")]
                    let (a2,b2) = if search_pvs_all {
                        (-beta, -alpha)
                    } else {
                        (-(alpha + 1), -alpha)
                    };
                    #[cfg(not(feature = "pvs_search"))]
                    let (a2,b2) = (-beta, -alpha);

                    res = {
                        let next_cut_node = !is_cut_node;
                        // trace!("search 1");
                        let res2 = -self._ab_search_negamax2::<{NonPV}>(
                            ts, &g2, (next_depth,ply+1), (a2,b2), stats, stack, next_cut_node);
                        if let Some(mut r) = res2.get_result_mv(mv) { r } else {
                            self.pop_nnue(stack);
                            continue 'outer;
                            // panic!();
                        }
                    };

                    /// Re-seach if limited window PV search failed
                    #[cfg(feature = "pvs_search")]
                    if !search_pvs_all && res.score > alpha && res.score < beta {
                        res = {
                            // let next_cut_node = !is_cut_node;
                            // trace!("search 2");
                            let res2 = -self._ab_search_negamax2::<{PV}>(
                                ts, &g2, (next_depth,ply+1), (-beta,-alpha), stats, stack, false);
                            if let Some(mut r) = res2.get_result_mv(mv) { r } else {
                                self.pop_nnue(stack);
                                continue 'outer;
                            }
                        };
                    }

                }

                // /// Do full PV Search on the first move
                /// Do full PV Search until a move is found that raises alpha
                #[cfg(feature = "pvs_search")]
                // if !do_full_depth && is_pv_node && moves_searched == 1 {
                // if !do_full_depth && is_pv_node && search_pvs_all {
                if is_pv_node && search_pvs_all {

                    // trace!("search 3");
                    let res2 = -self._ab_search_negamax2::<{PV}>(
                        ts, &g2, (next_depth,ply+1), (-beta, -alpha), stats, stack, false);
                    res = if let Some(mut r) = res2.get_result_mv(mv) { r } else {
                        self.pop_nnue(stack);
                        continue 'outer;
                    }

                }

                if res.mv == Move::NullMove {
                    panic!();
                    // continue 'outer;
                }
                res
            };

            if NODE_TYPE == Root {
                list.push(res);
            }

            let mut b = false;

            if res.score > best_val.1 {
                best_val.1 = res.score;
                best_val.0 = Some((g.zobrist, res))
            }

            #[cfg(not(feature = "negamax_only"))]
            {
                if res.score >= beta { // Fail Soft
                    b = true;
                    // return beta;
                }

                if !b && best_val.1 > alpha {
                    current_node_type = Node::PV;
                    alpha = best_val.1;

                    #[cfg(feature = "pvs_search")]
                    { search_pvs_all = false; }
                    // { do_pvs = true; }
                }

                /// Fail high, update stats
                if b {
                    // node_type = Some(Node::Cut);
                    current_node_type = Node::Cut;

                    if !mv.filter_all_captures() {
                        #[cfg(feature = "history_heuristic")]
                        stack.history.increment(mv, ply, g.state.side_to_move);

                        #[cfg(feature = "killer_moves")]
                        stack.killer_store(ply, mv);

                        #[cfg(feature = "countermove_heuristic")]
                        if let Some(prev_mv) = g.last_move {
                            stack.counter_moves.insert_counter_move(prev_mv, mv, g.state.side_to_move);
                        }

                    }

                    if moves_searched <= 1 {
                        stats!(stats.beta_cut_first.0 += 1);
                    } else {
                        stats!(stats.beta_cut_first.1 += 1);
                    }

                    self.pop_nnue(stack);
                    break;
                }
            }

            self.pop_nnue(stack);
        }

        /// Filter checkmate, stalemate
        if in_check && moves_searched == 0 {
            let score = CHECKMATE_VALUE - ply as Score;
            stats.leaves += 1;
            stats.checkmates += 1;
            let mv = g.last_move.unwrap();
            return ABSingle(ABResult::new_single(mv, -score));
        } else if moves_searched == 0 {
            let score = -STALEMATE_VALUE + ply as Score;
            stats.leaves += 1;
            stats.stalemates += 1;
            if let Some(mv) = g.last_move {
                // TODO: adjust stalemate value when winning/losing
                return ABSingle(ABResult::new_single(mv, 0));
            } else {
                return ABNone;
            }
        }

        /// XXX: stat padding by including nodes found in TT
        stats!(stats.inc_nodes_arr(ply));
        stats!(stats.nodes += 1);

        match &best_val.0 {
            Some((zb,res)) => {

                // if !is_root_node && current_node_type == Node::PV {
                if current_node_type == Node::PV {
                    // stack.pvs.push(res.mv);
                    stack.pvs[ply as usize] = res.mv
                }

                if !is_root_node && Some(res.mv) != movegen.hashmove {
                // if !is_root {


                    self.tt_insert_deepest(
                        g.zobrist,
                        SearchInfo::new(
                            res.mv,
                            // res.moves.len() as u8,
                            depth - 1,
                            // depth,
                            // node_type,
                            current_node_type,
                            res.score,
                        ));

                }

                if is_root_node {
                    ABList(*res, list)
                } else {
                    ABSingle(*res)
                }
            },
            _                    => ABNone,
        }

    }
}


