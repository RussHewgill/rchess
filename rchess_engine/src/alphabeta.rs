
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
    pub mv:       Option<Move>,
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

/// New
impl ABResult {

    pub fn new_null() -> Self {
        Self {
            // mv: Move::NullMove,
            mv: None,
            // score: 0,
            score: Score::MIN,
        }
    }

    pub fn new_null_score(score: Score) -> Self {
        Self {
            mv: None,
            score,
        }
    }

    pub fn new_single(mv: Move, score: Score) -> Self {
        Self {
            mv: Some(mv),
            score,
        }
    }

    pub fn neg_score(&mut self, mv: Move) {
        self.score = -self.score;
        self.mv = Some(mv);
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

/// Get results
impl ABResults {

    pub fn get_result_mv(&self, mv: Move) -> Option<ABResult> {
        let mut res = self.get_result()?;
        // res.mv = mv;
        res.mv = Some(mv);
        Some(res)
    }

    pub fn get_result(&self) -> Option<ABResult> {
        match self {
            Self::ABSingle(res)     => Some(*res),
            Self::ABList(res, _)    => Some(*res),
            Self::ABSyzygy(res)     => Some(*res),
            // Self::ABPrune(score, _) => None,
            Self::ABPrune(score, _) => Some(ABResult::new_null_score(*score)),
            Self::ABNone            => None,
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
    MultiCut,
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
        ply:          Depth,
        mv:           Move,
        zb0:          Option<Zobrist>,
        mut stack:    &mut ABStack,
    ) -> Option<Game> {
        if let Ok(g2) = g._make_move_unchecked(ts, mv, zb0) {

            stack.with(ply, |st| {
                // st.zobrist = g2.zobrist;
                st.current_move = Some(mv);
            });

            /// push NNUE
            #[cfg(feature = "nnue")]
            if let Some(nnue) = &self.nnue {
                let mut nn = nnue.borrow_mut();
                nn.ft.make_move(&g2, mv);
            }

            // stack.move_history.push((g.zobrist,mv));
            stack.move_history.push((g2.zobrist,mv));

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
        ab:             Option<(Score,Score)>,
        depth:          Depth,
    ) -> ABResults {

        // let (alpha,beta) = (Score::MIN,Score::MAX);
        // let (alpha,beta) = (alpha + 200,beta - 200);

        let (alpha,beta) = if let Some(ab) = ab { ab } else {
            (Score::MIN + 200, Score::MAX - 200)
        };

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
        let res = self.ab_search::<{ABNodeType::Root}>(
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

/// Repetition checking
impl ExHelper {

    pub fn has_cycle(
        &self,
        ts:                      &'static Tables,
        g:                       &Game,
        (alpha, beta):           (Score,Score),
        (ply,depth):             (Depth,Depth),
        mut stats:               &mut SearchStats,
        mut stack:               &mut ABStack,
    ) -> bool {
        use crate::cuckoo::*;

        // let end = Depth::min(g.halfmove)
        let end = g.halfmove as usize;
        if end < 3 { return false };

        let key0 = g.zobrist;

        // let cuckoo = &CUCKOO_TABLE;

        let last = stack.move_history.len() - 1;

        assert_eq!(g.zobrist, stack.move_history[last].0);

        // let mut key_other = !(g.zobrist ^ stack.move_history[last - 1].0);

        let mut i = 3;
        while i <= end {

            // let zb      = stack.move_history[i];
            // let zb_prev = stack.move_history[i+2];

            let zb_prev = if let Some(zb_prev) = stack.move_history.get(i+2) { zb_prev } else {
                break;
            };

            let mv_key = key0 ^ zb_prev.0;

            if let Some(k) = CUCKOO_TABLE.get_key(mv_key) {

                /// XXX: from,to OR to,from ??
                if let Some((from,to)) = CUCKOO_TABLE.cuckoo_move[k] {
                    if ((ts.between(from, to) ^ BitBoard::single(to)) & g.all_occupied()).is_empty() {

                        if ply > i as Depth {
                            return true;
                        }

                        // For nodes before or at the root, check that the move is a
                        // repetition rather than a move to the current position.
                        // In the cuckoo table, both moves Rc1c5 and Rc5c1 are stored in
                        // the same location, so we have to select which square to check.

                        let ss = if g.all_occupied().is_one_at(from) {
                            from
                        } else if g.all_occupied().is_one_at(to) {
                            to
                        } else {
                            panic!("wat");
                        };

                        if Some(g.state.side_to_move) != g.get_side_at(ss) {
                            continue;
                        }

                        // TODO: need extra repetition at root

                    }
                }
            }

            i += 2;
        }


        // /// Repetition checking
        // if alpha < DRAW_VALUE {
        //     // for (zb,_) in stack.move_history.iter().step_by(2) {
        //     let cycle = stack.move_history.iter().any(|&(zb2,_)| g.zobrist == zb2);
        //     // let cycle = stack.move_history.contains(&zb2);
        //     if cycle && alpha >= beta {
        //         debug!("found cycle, {:?}, {:?}", alpha, beta);
        //         // return ABSingle(ABResult::new_single(g.last_move.unwrap(), DRAW_VALUE));
        //         return true;
        //     } else {
        //         debug!("found cycle but no return, {:?}, {:?}", alpha, beta);
        //     }
        // } else {
        //     let cycle = stack.move_history.iter().any(|&(zb2,_)| g.zobrist == zb2);
        //     if cycle {
        //         debug!("found cycle but alpha < DRAW_VALUE, {:?}, {:?}", alpha, beta);
        //     }
        // }

        false
    }

}

/// Negamax AB Refactor
impl ExHelper {
    #[allow(unused_doc_comments,unused_labels)]
    pub fn ab_search<const NODE_TYPE: ABNodeType>(
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
        stack.with(ply, |st| st.material = g.state.material);

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

            let cycle = self.has_cycle(ts, g, (alpha,beta), (ply,depth), stats, stack);
            if cycle {
                // eprintln!("cycle = {:?}", cycle);
                return ABSingle(ABResult::new_single(g.last_move.unwrap(), DRAW_VALUE));
            }

            // /// Repetition checking
            // if alpha < DRAW_VALUE {
            //     // for (zb,_) in stack.move_history.iter().step_by(2) {
            //     let cycle = stack.move_history.iter().any(|&(zb2,_)| g.zobrist == zb2);
            //     // let cycle = stack.move_history.contains(&zb2);
            //     if cycle && alpha >= beta {
            //         // debug!("found cycle, {:?}, {:?}", alpha, beta);
            //         return ABSingle(ABResult::new_single(g.last_move.unwrap(), DRAW_VALUE));
            //     } else {
            //         // debug!("found cycle but no return, {:?}, {:?}", alpha, beta);
            //     }
            // } else {
            //     // let cycle = stack.move_history.iter().any(|&(zb2,_)| g.zobrist == zb2);
            //     // if cycle {
            //     //     // debug!("found cycle but alpha < DRAW_VALUE, {:?}, {:?}", alpha, beta);
            //     // }
            // }

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
            // if !is_pv_node && si.depth_searched >= depth { // XXX: depth or depth-1 ??
            //     return ABResults::ABSingle(ABResult::new_single(g.last_move.unwrap(), si.score));
            // }

            if si.depth_searched >= depth && (depth == 0 || !is_pv_node) {

                if si.node_type == Node::PV
                    || (si.node_type == Node::Cut && si.score >= beta)
                    || (si.node_type == Node::All && si.score <= alpha)
                {
                    return ABResults::ABSingle(ABResult::new_single(g.last_move.unwrap(), si.score));
                }

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
                            // let score = CHECKMATE_VALUE - dtz.add_plies(ply as i32).0.abs() as Score;
                            let score = TB_WIN_VALUE - dtz.add_plies(ply as i32).0.abs() as Score;

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

        /// Static eval of position
        let static_eval = if in_check {
            /// In check, no static eval
            stack.with(ply, |st| st.in_check = true);
            None
        } else if msi.map(|si| si.depth_searched >= depth).unwrap_or(false) {
            let score = msi.unwrap().score;
        // } else if let Some(score) = msi.map(|si| si.score) {
            /// Get search score from TT
            stack.with(ply, |st| st.static_eval = Some(score));
            Some(score)
        } else if let Some(nnue) = &self.nnue {
            /// NNUE Eval, cheap-ish
            /// TODO: bench vs evaluate
            let mut nn = nnue.borrow_mut();
            let score = nn.evaluate(&g, true);
            stack.with(ply, |st| st.static_eval = Some(score));
            Some(score)
        } else {
            let stand_pat = self.cfg.evaluate(ts, g, &self.ph_rw);
            let score = if g.state.side_to_move == Black { -stand_pat } else { stand_pat };
            stack.with(ply, |st| st.static_eval = Some(score));
            Some(score)
        };
        stack.with(ply, |st| st.static_eval = static_eval);

        let mut improving   = !in_check;
        if let Some(eval) = static_eval {
            if let Some(prev1) = stack.get(ply - 2).map(|st| st.static_eval).flatten() {
                improving = eval > prev1;
            } else if let Some(prev2) = stack.get(ply - 4).map(|st| st.static_eval).flatten() {
                improving = eval > prev2;
            }
        }

        let mut can_futility_prune = false;
        /// TODO: futility pruning
        #[cfg(feature = "futility_pruning")]
        if depth == 1
            && !is_pv_node
            && !in_check
            && static_eval.unwrap() - FUTILITY_MARGIN >= beta
            // && static_eval.unwrap() + FUTILITY_MARGIN <= alpha
            && static_eval.unwrap() < FUTILITY_MIN_ALPHA
        {
            // let eval = self.cfg.evaluate(ts, g, &self.ph_rw);
            // if eval + FUTILITY_MARGIN 
            // stats.fut_prunes += 1;
            // return ABPrune(static_eval.unwrap(), Prune::Futility);
            can_futility_prune = true;
        };

        /// Reverse Futility Pruning, Static Null Pruning
        if !is_pv_node
            && !in_check
            && depth <= RFP_MIN_DEPTH
            && static_eval.unwrap() - RFP_MARGIN * depth as Score > beta
        {
            return ABPrune(static_eval.unwrap(), Prune::Futility);
        }

        /// Null move pruning
        /// skip when TT hit suggests it will fail
        #[cfg(feature = "null_pruning")]
        if !is_pv_node
            // && !stack.inside_null // don't ever recursively null prune
            && !in_check
            // && g.last_move != Some(Move::NullMove) // don't null prune twice in a row
            && stack.get_with(ply - 1, |st| st.current_move != Some(Move::NullMove)).unwrap_or(true)
            && stack.get_with(ply - 2, |st| st.current_move != Some(Move::NullMove)).unwrap_or(true)
            && depth >= NULL_PRUNE_MIN_DEPTH
            && g.state.phase < NULL_PRUNE_MIN_PHASE
            && g.state.material.any_non_pawn(g.state.side_to_move)
            && msi.map(|si| si.node_type != Node::All || si.score >= beta).unwrap_or(false)
        {
            // let r = NULL_PRUNE_REDUCTION; // 2
            let r = 3 + depth / 6;

            // assert!(depth - 1 >= r);

            // let null_depth = depth - 1 - r;
            let null_depth = (depth - 1).checked_sub(r).unwrap_or(0);

            // if let Ok(g2) = g.make_move_unchecked(ts, Move::NullMove) {
            if let Some(g2) = self.make_move(ts, g, ply, Move::NullMove, None, stack) {

                // stack.inside_null = true;
                let res = -self.ab_search::<{NonPV}>(
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

        let mut depth = depth;

        /// Lower depth for positions not in TT
        if !is_root_node
            && is_pv_node
            && depth >= 6
            && msi.is_none() {
                depth -= 2;
            }

        if !is_root_node
            && is_cut_node
            && depth >= 9
            && msi.is_none() {
                depth -= 1;
            }

        // /// Reset killers for next ply
        // stack.killers_clear(ply + 2);

        let m_hashmove: Option<Move> = msi.map(|si| {
            let mv = si.best_move;
            // let mv = PackedMove::unpack(&[mv.0,mv.1]).unwrap().convert_to_move(ts, g);
            mv
        });

        let mut movegen = MoveGen::new(ts, &g, m_hashmove, stack, depth, ply);

        // let mut movegen = if is_root_node {
        //     let root_moves = self.root_moves.borrow();
        //     MoveGen::new_root(ts, &g, &root_moves)
        // } else {
        //     MoveGen::new(ts, &g, m_hashmove, stack, depth, ply)
        // };

        // /// true until a move is found that raises alpha
        // let mut search_pvs_all = true;

        // #[cfg(feature = "pvs_search")]
        // if depth < 3 {
        //     // do_pvs = false;
        //     search_pvs_all = false;
        // }

        let mut moves_searched = 0;
        let mut val = Score::MIN + 200;
        let mut best_val: (Option<ABResult>,Score) = (None,val);
        let mut list = vec![];

        let mut captures_searched: ArrayVec<Move, 64> = ArrayVec::new();
        let mut quiets_searched: ArrayVec<Move, 64>   = ArrayVec::new();

        /// Loop over moves
        'outer: while let Some(mv) = movegen.next(&stack) {

            let mut next_depth = depth - 1;
            let mut extensions = 0;

            /// Prefetch hash table bucket
            let zb0 = g.zobrist.update_move_unchecked(ts, g, mv);
            #[cfg(feature = "lockless_hashmap")]
            self.ptr_tt.prefetch(zb0);

            /// Skip blocked moves
            if is_root_node && self.cfg.blocked_moves.contains(&mv) {
                continue 'outer;
            }

            let capture_or_promotion = mv.filter_all_captures() || mv.filter_promotion();
            let gives_check = movegen.gives_check(mv);

            // /// Move Count pruning
            // if best_val.1 > -CHECKMATE_VALUE
            //     // && depth <= LMR_MIN_DEPTH
            //     && depth <= 8 // XXX: ??
            //     && moves_searched >= futility_move_count(improving, depth) {
            //         movegen.skip_quiets = true;
            //     }

            /// Futility prune
            #[cfg(feature = "futility_pruning")]
            if can_futility_prune
                && moves_searched > 1
                && best_val.0.is_some()
                && !in_check
                && !gives_check
                && !mv.filter_all_captures()
                && !mv.filter_promotion()
            {
                stats.fut_prunes += 1;
                continue;
            }

            // /// Shallow pruning
            // if !is_root_node
            //     && g.state.material.any_non_pawn(g.state.side_to_move)
            // {
            //     let lmr_depth = next_depth - lmr_reduction(depth, moves_searched);
            //     if capture_or_promotion || gives_check {
            //         if !gives_check
            //             && lmr_depth < 1
            //             && (mv.filter_all_captures() && stack.capture_history.get(mv) < 0) {
            //                 continue;
            //             }
            //         // if !movegen.static_exchange_ge(mv, 200 * depth as Score) {
            //         //     continue;
            //         // }
            //     } else {
            //     }
            // }

            /// Singular extension
            #[cfg(feature = "singular_extensions")]
            if let Some(si) = msi {
                if !is_root_node
                    && depth >= 7
                    && Some(mv) == m_hashmove
                    && si.node_type == Node::Cut // lower bound
                    && si.depth_searched >= depth - 3
                    // && si.eval.is_some()
                    && static_eval.is_some()
                {
                    // let tt_eval = si.eval.unwrap(); // TODO: let_chains
                    let tt_eval = static_eval.unwrap(); // TODO: let_chains

                    let sing_beta  = tt_eval - 3 * depth as Score;
                    let sing_depth = (depth - 1) / 2;

                    stack.with(ply, |st| st.forbidden_move = Some(mv));

                    let res2 = self.ab_search::<{NonPV}>(
                        ts, &g, (sing_depth,ply+1), (sing_beta-1,sing_beta), stats, stack, is_cut_node);
                    stack.with(ply, |st| st.forbidden_move = None);
                    if let Some(res) = res2.get_result_mv(mv) {

                        if res.score < sing_beta {
                            extensions = 1;
                            // TODO: limit LMR?
                            // TODO: limit explosion?
                        } else if sing_beta >= beta {

                            return ABPrune(sing_beta, Prune::MultiCut);
                        } else if tt_eval >= beta {
                            extensions -= 2;
                        }

                    }
                }
            } else if (is_pv_node || is_cut_node)
                && capture_or_promotion
                && moves_searched != 1
            {
                // Capture extensions for pv node
                extensions += 1;
            } else if gives_check
                && depth > 6
                && static_eval.map(|s| s.abs() > 100).unwrap_or(false)
            {
                // Check extensions
                extensions += 1;
            }

            /// Make move
            let g2 = if let Some(g2) = self.make_move(ts, g, ply, mv, Some(zb0), stack) {
                g2
            } else {
                continue 'outer;
            };
            moves_searched += 1;

            next_depth += extensions;

            let res: ABResult = 'search: {
                let mut res: ABResult = ABResult::new_null();

                let mut lmr = self.cfg.late_move_reductions;

                let mut do_full_depth = true; // XXX: ??

                if lmr && mv.filter_all_captures() {
                    // let see = g.static_exchange(&ts, mv).unwrap(); // XXX: g or g2?
                    let see = movegen.static_exchange_ge(mv, 1);
                    // let see = g.static_exchange_ge(ts, mv, 1);
                    /// Capture with good SEE: do not reduce
                    if see {
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
                    && !gives_check
                    && g2.state.checkers.is_empty()
                {

                    // let depth_r = next_depth.checked_sub(LMR_REDUCTION).unwrap();
                    // // let depth_r = next_depth.checked_sub(1).unwrap();

                    let mut r = lmr_reduction(next_depth, moves_searched) as i16;

                    if mv.filter_quiet() {

                        if !is_pv_node { r += 1; }

                        if !improving { r += 1; }

                        /// King evasions
                        if in_check && mv.piece() == Some(King) {
                            r += 1;
                        }

                        if movegen.is_killer(stack, mv) || movegen.is_counter(stack, mv) {
                            r -= 1;
                        }

                    }

                    let r = (r as Depth).clamp(1, next_depth + 1);
                    let depth_r = next_depth.checked_sub(r).unwrap();

                    let (a2,b2) = (-(alpha+1),-alpha); // XXX: ??

                    // trace!("search 0");
                    let res2 = -self.ab_search::<{NonPV}>(
                        ts, &g2, (depth_r,ply+1), (a2,b2), stats, stack, true);

                    res = if let Some(r) = res2.get_result_mv(mv) { r } else {
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
                    // do_full_depth = !is_pv_node || moves_searched > 1;
                    do_full_depth = !(is_pv_node && moves_searched == 1);
                }
                // #[cfg(not(feature = "late_move_reduction"))]
                // { do_full_depth = true; }

                /// Full depth search if no LMR and not PV Node's first search
                if do_full_depth {
                    let (a2,b2) = (-(alpha+1),-alpha); // XXX: ??
                    let res2 = -self.ab_search::<{NonPV}>(
                        ts, &g2, (next_depth,ply+1), (a2,b2), stats, stack, !is_cut_node);
                    res = if let Some(r) = res2.get_result_mv(mv) { r } else {
                        self.pop_nnue(stack);
                        continue 'outer;
                    };
                }

                /// Search PV with full window
                if is_pv_node && (moves_searched == 1 || res.score > alpha) {
                    let res2 = -self.ab_search::<{PV}>(
                        ts, &g2, (next_depth,ply+1), (-beta, -alpha), stats, stack, false);
                    res = if let Some(r) = res2.get_result_mv(mv) { r } else {
                        self.pop_nnue(stack);
                        continue 'outer;
                    };
                }

                /// Prev
                #[cfg(feature = "nope")]
                {

                    ///   Full depth search
                    /// If LMR failed or was skipped
                    // if do_full_depth {
                    if false && do_full_depth {

                        // let (a2,b2) = if search_pvs_all || cfg!(not(feature = "pvs_search")) {
                        //     (-beta, -alpha)
                        // } else {
                        //     (-(alpha + 1), -alpha)
                        // };

                        #[cfg(feature = "pvs_search")]
                        let (a2,b2) = if search_pvs_all {
                            (-beta, -alpha)
                        } else {
                            (-(alpha + 1), -alpha)
                        };
                        #[cfg(not(feature = "pvs_search"))]
                        let (a2,b2) = (-beta, -alpha);

                        let res2 = -self.ab_search::<{NonPV}>(
                            ts, &g2, (next_depth,ply+1), (a2,b2), stats, stack, !is_cut_node);

                        res = if let Some(r) = res2.get_result_mv(mv) { r } else {
                            self.pop_nnue(stack);
                            continue 'outer;
                        };

                    }

                    // if false && do_full_depth && (!search_pvs_all || !is_pv_node) {
                    // if do_full_depth && (!search_pvs_all || !is_pv_node) {
                    if do_full_depth {

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
                            let res2 = -self.ab_search::<{NonPV}>(
                                ts, &g2, (next_depth,ply+1), (a2,b2), stats, stack, next_cut_node);
                            if let Some(r) = res2.get_result_mv(mv) { r } else {
                                self.pop_nnue(stack);
                                continue 'outer;
                            }
                        };

                        /// Re-seach if limited window PV search failed
                        #[cfg(feature = "pvs_search")]
                        if !search_pvs_all && res.score > alpha && res.score < beta {
                            res = {
                                // let next_cut_node = !is_cut_node;
                                // trace!("search 2");
                                let res2 = -self.ab_search::<{PV}>(
                                    ts, &g2, (next_depth,ply+1), (-beta,-alpha), stats, stack, false);
                                if let Some(r) = res2.get_result_mv(mv) { r } else {
                                    self.pop_nnue(stack);
                                    continue 'outer;
                                }
                            };
                        }

                    }

                    let search_pv = moves_searched == 1
                        || (res.score > alpha && (is_root_node || res.score < beta));

                    /// Do full PV Search until a move is found that raises alpha
                    #[cfg(feature = "pvs_search")]
                    // if is_pv_node && search_pvs_all {
                    if is_pv_node && search_pv {
                        let res2 = -self.ab_search::<{PV}>(
                            ts, &g2, (next_depth,ply+1), (-beta, -alpha), stats, stack, false);

                        res = if let Some(r) = res2.get_result_mv(mv) { r } else {
                            self.pop_nnue(stack);
                            continue 'outer;
                        };

                    }

                }

                // if res.mv == Move::NullMove {
                if res.mv.is_none() {
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
                // best_val.0 = Some((g.zobrist, res))
                best_val.0 = Some(res);
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

                    // #[cfg(feature = "pvs_search")]
                    // { search_pvs_all = false; }
                }

                // if !b && Some(mv) != best_val.0.map(|x| x.mv).flatten() {
                if !b {
                    if mv.filter_capture_or_promotion() {
                        // captures_searched.push(mv);
                        captures_searched.try_push(mv).unwrap_or_else(|_| {});
                    } else {
                        // quiets_searched.push(mv);
                        quiets_searched.try_push(mv).unwrap_or_else(|_| {});
                    }
                }

                /// Fail high, update stats
                if b {
                    // node_type = Some(Node::Cut);
                    current_node_type = Node::Cut;

                    if !mv.filter_all_captures() {

                        // #[cfg(feature = "history_heuristic")]
                        // stack.history.update(mv, g.state.side_to_move, ABStack::stat_bonus(depth));

                        #[cfg(feature = "killer_moves")]
                        stack.killers_store(ply, mv);

                        #[cfg(feature = "countermove_heuristic")]
                        if let Some(prev_mv) = g.last_move {
                            stack.counter_moves.insert_counter_move(prev_mv, mv, g.state.side_to_move);
                            // stack.counter_moves.insert_counter_move(prev_mv, mv);
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
        if in_check && (moves_searched == 0 || best_val.0.is_none()) {
            let score = CHECKMATE_VALUE - ply as Score;
            stats.leaves += 1;
            stats.checkmates += 1;
            let mv = g.last_move.unwrap();
            return ABSingle(ABResult::new_single(mv, -score));
        } else if moves_searched == 0 || best_val.0.is_none() {
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

        // if best_val.0.is_none() {
        //     eprintln!("moves_searched = {:?}", moves_searched);
        //     eprintln!("(alpha,beta) = {:?}", (alpha,beta));
        //     panic!("wat");
        // }

        /// XXX: stat padding by including nodes found in TT
        stats!(stats.inc_nodes_arr(ply));
        stats!(stats.nodes += 1);

        /// Update hash table and return
        match &best_val.0 {
            Some(res) => {

                let mv = res.mv.unwrap();

                stack.update_history(
                    g, mv, res.score, beta, captures_searched, quiets_searched, ply, depth);

                // if !is_root_node && current_node_type == Node::PV {
                if current_node_type == Node::PV {
                    // stack.pvs.push(res.mv);
                    stack.pvs[ply as usize] = mv
                }

                // if !is_root_node && Some(res.mv) != movegen.hashmove {
                if !is_root_node && Some(mv) != movegen.hashmove {
                // if !is_root {


                    self.tt_insert_deepest(
                        g.zobrist,
                        SearchInfo::new(
                            // res.mv,
                            mv,
                            // res.moves.len() as u8,
                            depth - 1,
                            // depth,
                            // node_type,
                            current_node_type,
                            res.score,
                            // static_eval,
                        ));

                }

                if is_root_node {
                    ABList(*res, list)
                } else {
                    ABSingle(*res)
                }
            },
            // _                    => ABNone,
            _                    => panic!("no moves found at node?"),
        }

    }
}


