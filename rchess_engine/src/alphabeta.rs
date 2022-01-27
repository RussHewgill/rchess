
use crate::lockless_map::TTEval;
// use crate::heuristics::*;
use crate::movegen::*;
use crate::searchstats;
// use crate::threading::ExThread;
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

// #[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Debug,PartialEq,Clone,Copy)]
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

// #[derive(Debug,PartialEq,PartialOrd,Clone)]
#[derive(Debug,PartialEq,Clone)]
pub enum ABResults {
    ABSingle(ABResult),
    ABList(ABResult, Vec<ABResult>),
    ABSyzygy(ABResult),
    // ABMate()
    ABPrune(Score, Prune),
    ABHalt,
    // ABNone,
    ABUninit,
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

    pub fn print_type(&self) {
        match self {
            Self::ABSingle(res)     => eprint!("ABSingle"),
            Self::ABList(res, _)    => eprint!("ABList"),
            Self::ABSyzygy(res)     => eprint!("ABSyzygy"),
            // Self::ABPrune(score, _) => None,
            Self::ABPrune(score, _) => eprint!("ABPrune"),
            Self::ABHalt            => eprint!("ABHAlt"),
            // Self::ABNone            => None,
            Self::ABUninit          => eprint!("ABUninit"),
        }
    }

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
            Self::ABHalt            => None,
            // Self::ABNone            => None,
            Self::ABUninit          => None,
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
        stack:        &mut ABStack,
    ) -> Option<Game> {
        if let Ok(g2) = g._make_move_unchecked(ts, mv, zb0) {

            stack.with(ply, |st| { st.current_move = Some(mv); });

            /// push NNUE
            #[cfg(feature = "nnue")]
            if let Some(nnue) = &self.nnue {
                let mut nn = nnue.borrow_mut();
                nn.ft.make_move(&g2, mv);
            }

            stack.move_history.push((g2.zobrist,mv));

            Some(g2)
        } else { None }
    }

    // #[inline(always)]
    pub fn pop_nnue(&self, stack: &mut ABStack) {
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
        // ts:             &'static Tables,
        ts:             &Tables,
        stats:          &mut SearchStats,
        stack:          &mut ABStack,
        ab:             Option<(Score,Score)>,
        depth:          Depth,
    ) -> ABResults {

        // let (alpha,beta) = (Score::MIN,Score::MAX);
        // let (alpha,beta) = (alpha + 200,beta - 200);

        let (alpha,beta) = if let Some(ab) = ab { ab } else {
            (Score::MIN + 200, Score::MAX - 200)
        };

        let g = self.game.clone();

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

    pub fn check_tt2(
        &self,
        ts:             &Tables,
        zb:             Zobrist,
        depth:          Depth,
        mut stats:      &mut SearchStats,
    // ) -> Option<SearchInfo> {
    ) -> (Option<Score>, Option<SearchInfo>) {
        #[cfg(feature = "lockless_hashmap")]
        {
            let (meval,msi) = self.ptr_tt.probe(zb);
            match (meval,msi) {
                (Some(TTEval::Check),None) => {
                    stats.tt_eval += 1;
                },
                (None,None) => {
                    stats!(stats.tt_misses += 1);
                },
                (Some(eval),None) => {
                    stats.tt_eval += 1;
                },
                (_,Some(si)) => {
                    if si.depth_searched >= depth {
                        stats!(stats.tt_hits += 1);
                    } else {
                        stats!(stats.tt_halfmiss += 1);
                    }
                },
            }
            let meval = meval.map(|x| x.to_option()).flatten();
            (meval,msi)
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

    // #[cfg(feature = "nope")]
    pub fn has_cycle(
        // ts:                      &'static Tables,
        ts:                      &Tables,
        g:                       &Game,
        stats:                   &mut SearchStats,
        stack:                   &ABStack,
    ) -> bool {

        let end = g.halfmove as usize;
        if end < 3 { return false; }

        let last = stack.move_history.len() as i32 - 1;
        if last <= 0 { return false; }

        assert_eq!(g.zobrist, stack.move_history[last as usize].0);

        let end = end as i32 - last;

        let zb0 = g.zobrist;

        let mut i = last - 2;
        while i >= end {

            let (zb_prev,mv) = if let Some(zb) = stack.move_history.get(i as usize) {
                zb
            } else { break; };

            if *zb_prev == zb0 {
                return true;
            }

            i -= 2;
        }

        false
    }

    #[cfg(feature = "nope")]
    pub fn has_cycle(
        // &self,
        // ts:                      &'static Tables,
        ts:                      &Tables,
        g:                       &Game,
        ply:                     Depth,
        mut stats:               &mut SearchStats,
        stack:                   &ABStack,
    ) -> bool {
        use crate::cuckoo::*;

        // let end = Depth::min(g.halfmove)
        let end = g.halfmove as usize;
        if end < 3 { return false };

        let key0 = g.zobrist;

        // let cuckoo = &CUCKOO_TABLE;

        if stack.move_history.len() == 0 { return false; }
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

        false
    }

}

/// Get eval
impl ExHelper {

    pub fn get_static_eval(
        &self,
        // ts:           &'static Tables,
        ts:           &Tables,
        g:            &Game,
        ply:          Depth,
        stack:        &mut ABStack,
        meval:        Option<Score>,
        msi:          Option<SearchInfo>,
    ) -> Option<Score> {

        if g.state.in_check {
            stack.with(ply, |st| {
                st.in_check    = true;
                st.static_eval = None;
            });
            None
        // } else if let Some(si) = msi {
        } else if msi.is_some() || meval.is_some() {

            let mut eval = if let Some(eval) = meval { eval } else {
                // self.eval_nn_or_hce(ts, g)
                self.evaluate(ts, g, false)
            };

            if let Some(si) = msi {
                let bb = if si.score > eval { Node::Lower } else { Node::Upper };
                if si.node_type == bb {
                    eval = si.score;
                }
            }

            // if si.depth_searched >= depth {
            //     stack.with(ply, |st| st.static_eval = Some(eval));
            //     return Some(eval);
            // }

            stack.with(ply, |st| st.static_eval = Some(eval));

            Some(eval)
        } else {
            // let eval = self.eval_nn_or_hce(ts, g);
            let eval = self.evaluate(ts, g, false);
            stack.with(ply, |st| st.static_eval = Some(eval));

            self.tt_insert_deepest_eval(g.zobrist, Some(eval));

            Some(eval)
        }

    }

}

/// search_explosion
impl ExHelper {
    pub fn search_explosion(&self, stack: &mut ABStack, stats: &SearchStats) -> bool {

        let explosive = stack.double_ext_avg[White].is_greater(2, 100)
            || stack.double_ext_avg[Black].is_greater(2, 100);

        // if explosive {
        //     // stack.exploding 
        // }

        // unimplemented!()
        explosive
    }
}

/// Negamax AB Refactor
impl ExHelper {
    #[allow(unused_doc_comments,unused_labels)]
    /// alpha: the MIN score that the maximizing player is assured of
    /// beta:  the MAX score that the minimizing player is assured of
    pub fn ab_search<const NODE_TYPE: ABNodeType>(
        &self,
        // ts:                      &'static Tables,
        ts:                      &Tables,
        g:                       &Game,
        (depth,ply):             (Depth,Depth),
        (mut alpha, mut beta):   (Score,Score),
        mut stats:               &mut SearchStats,
        mut stack:               &mut ABStack,
        is_cut_node:             bool,
    ) -> ABResults {
        use ABNodeType::*;

        // trace!("negamax entry, ply {}, a/b = {:>10}/{:>10}", k, alpha, beta);

        // XXX: doesn't seem to ever happen
        // /// limit search explosion
        // let depth = if ply > 10
        //     && self.search_explosion(stack, &stats)
        //     && depth > stack.get_with(ply - 1, |st| st.depth).unwrap()
        // {
        //     stack.get_with(ply - 1, |st| st.depth).unwrap()
        // } else { depth };

        // let mut current_stack: &mut ABStackPly = stack.get_or_push(ply);
        // stack.push_if_empty(g, ply);
        // stack.with(ply, |st| st.material = g.state.material);

        stack.init_node(ply, depth, g);

        #[cfg(feature = "pvs_search")]
        let is_pv_node = NODE_TYPE != NonPV;
        #[cfg(not(feature = "pvs_search"))]
        let is_pv_node = false;

        // #[cfg(feature = "pvs_search")]
        // let is_pv_node = beta != alpha + 1;

        let is_root_node: bool = NODE_TYPE == Root;

        // let current_node_type = Node::Upper; // not used

        /// when in check, skip early pruning
        let in_check = g.state.in_check;

        let val = Score::MIN + 200;
        let mut best_val: (Option<ABResult>,Score) = (None,val);
        let mut max_score = Score::MAX;

        /// Max search depth
        if ply >= MAX_SEARCH_PLY {
            if !in_check {
                // let score = self.eval_nn_or_hce(ts, g);
                let score = self.evaluate(ts, g, false);
                return ABSingle(ABResult::new_null_score(score));
            } else {
                let score = draw_value(stats);
                return ABSingle(ABResult::new_null_score(score));
            }
        }

        /// Misc assertions
        #[cfg(feature = "pvs_search")]
        {
            assert!(is_pv_node || (alpha == beta - 1));
            assert!(!(is_pv_node && is_cut_node));
        }
        assert!(depth < MAX_SEARCH_PLY);
        if depth < 0 {
            eprintln!("wat: depth = {:?}", depth);
        }
        assert!(depth >= 0);

        /// Step 1. Repetition
        if !is_root_node
            && alpha < DRAW_VALUE
            && Self::has_cycle(ts, g, stats, stack)
        {
            // eprintln!("ply, mv, cycle = {:?}, {:?}, {}", ply, g.last_move.unwrap(), cycle);
            let score = draw_value(stats);
            return ABSingle(ABResult::new_single(g.last_move.unwrap(), score));
        }

        /// Step 2. Halting for mate, time, etc
        if !is_root_node {

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
                // return ABNone;
                return ABHalt;
            }

            // /// Mate found
            // {
            //     let r = self.best_mate.read();
            //     if let Some(best) = *r {
            //         trace!("halting search, mate found");
            //         return ABHalt;
            //     }
            // }

            /// Mate found
            if self.best_mate.load(Relaxed) != -1 {
                trace!("halting search, mate found");
                return ABHalt;
            }

        }

        /// Step 3. Qsearch at zero depth
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
                    self.qsearch::<{PV}>(&ts, &g, (ply,0,0), (alpha, beta), stack, stats)
                } else {
                    self.qsearch::<{NonPV}>(&ts, &g, (ply,0,0), (alpha, beta), stack, stats)
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

        /// update double extension running average
        stack.update_double_extension_avg(ply, g.state.side_to_move);

        /// Step 4. Transposition Table probe
        // TODO: should have a vec of moves stored for root moves instead of using TT
        let (meval,msi): (Option<Score>,Option<SearchInfo>) = if is_root_node { (None,None) } else {
            self.check_tt2(ts, g.zobrist, depth, stats)
        };

        // let (meval,msi): (Option<Score>,Option<SearchInfo>) = self.check_tt2(ts, g.zobrist, depth, stats);

        /// Step 4b. Check for returnable TT score
        if let Some(si) = msi {
            // if !is_pv_node && si.depth_searched >= depth { // XXX: depth or depth-1 ??
            //     return ABResults::ABSingle(ABResult::new_single(g.last_move.unwrap(), si.score));
            // }

            /// Only return if the TT entry has a greater depth
            if si.depth_searched >= depth
                && (depth == 0 || !is_pv_node)
            {

                if si.node_type == Node::Exact
                    || (si.node_type == Node::Lower && si.score >= beta)
                    || (si.node_type == Node::Upper && si.score <= alpha)
                {
                    return ABResults::ABSingle(ABResult::new_single(g.last_move.unwrap(), si.score));
                }

            }

        }

        /// Step 5. Syzygy Probe
        #[cfg(feature = "syzygy")]
        if let Some(tb) = &self.syzygy {
            // debug!("// TODO: syzygy probe handling");

            // if !is_root_node
            if !g.state.castling.any()
                && g.halfmove == 0
            {
                if let Ok(wdl) = tb.probe_wdl(ts, g) {
                    let wdls = wdl as Score;
                    if let Ok(Some((mv,dtz))) = tb.best_move(ts, g) {
                        // println!("wat 0, wdl = {:?}", wdls);

                        let drawscore = 1;

                        let score = if wdls < -drawscore {
                            -MATE_IN_MAX_PLY + ply as Score + 1
                        } else if wdls > drawscore {
                            MATE_IN_MAX_PLY - ply as Score - 1
                        } else {
                            DRAW_VALUE + 2 * wdls * drawscore
                        };

                        let bound = if wdls < -drawscore {
                            Node::Upper
                        } else if wdls > drawscore {
                            Node::Lower
                        } else {
                            Node::Exact
                        };

                        // eprintln!("(score,bound) = {:?}", (score,bound));
                        // eprintln!("(alpha,beta) = {:?}", (alpha,beta));

                        // self.tt_insert_deepest_eval(g.zobrist, Some(score));
                        self.tt_insert_deepest(g.zobrist, None, SearchInfo::new(
                            mv,
                            depth + 6, // XXX: stockfish does + 6, not sure why
                            bound,
                            score,
                            None,
                        ));

                        return ABResults::ABSyzygy(ABResult::new_single(mv, score));

                        // if bound == Node::Exact
                        //     || (if bound == Node::Lower { score >= beta } else { score <= alpha })
                        // {
                        //     println!("wat 1");
                        //     self.tt_insert_deepest(g.zobrist, None, SearchInfo::new(
                        //         mv,
                        //         depth + 6, // XXX: stockfish does + 6, not sure why
                        //         bound,
                        //         score,
                        //         None,
                        //     ));
                        //     return ABResults::ABSyzygy(ABResult::new_single(mv, score));
                        // } else {
                        //     println!("wat 2");
                        // }

                        // if is_pv_node {
                        //     if bound == Node::Lower {
                        //         best_val.1 = score;
                        //         alpha      = alpha.max(score);
                        //     } else {
                        //         max_score = score;
                        //     }
                        // }

                    }

                }
            }


            #[cfg(feature = "nope")]
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

                            let draw_score = 0;

                            let bound = if wdl < -draw_score {
                                Node::Upper
                            } else if wdl > draw_score {
                                Node::Lower
                            } else {
                                Node::Exact
                            };

                            self.tt_insert_deepest(g.zobrist, None, SearchInfo::new(
                                mv,
                                depth + 6, // XXX: stockfish does + 6, not sure why
                                bound,
                                score,
                                None,
                            ));

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

        /// Static eval, possibly from TT
        let static_eval = self.get_static_eval(ts, g, ply, stack, meval, msi);

        let mut improving = !in_check;
        if let Some(eval) = static_eval {
            if let Some(prev1) = stack.get(ply - 2).map(|st| st.static_eval).flatten() {
                improving = eval > prev1;
            } else if let Some(prev2) = stack.get(ply - 4).map(|st| st.static_eval).flatten() {
                improving = eval > prev2;
            }
        }

        let mut can_futility_prune = false;
        /// Step 6. Futility pruning check
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

        /// Step 7. Reverse Futility Pruning, Static Null Pruning
        if !is_pv_node
            && !in_check
            && depth <= RFP_MIN_DEPTH
            && static_eval.unwrap() - RFP_MARGIN * depth as Score > beta
        {
            return ABPrune(static_eval.unwrap(), Prune::Futility);
        }

        /// Step 8. Null move pruning
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
            // && msi.map(|si| si.node_type != Node::Upper || si.score >= beta).unwrap_or(false)
            && (msi.is_none()
                || msi.unwrap().node_type != Node::Upper
                || msi.unwrap().score >= beta) // from Ethereal
        {
            // let r = NULL_PRUNE_REDUCTION; // 2
            let r = 3 + depth / 6;

            // assert!(depth - 1 >= r);

            // let null_depth = depth - 1 - r;
            let null_depth = (depth - 1).checked_sub(r).unwrap_or(0);
            // let null_depth = null_depth.max(0);
            // assert!(null_depth >= 0);

            if null_depth > 0 {
                // if let Ok(g2) = g.make_move_unchecked(ts, Move::NullMove) {
                if let Some(g2) = self.make_move(ts, g, ply, Move::NullMove, None, stack) {

                    // stack.inside_null = true;
                    let res = -self.ab_search::<{NonPV}>(
                        ts, &g2, (null_depth,ply+1), (-beta,-beta+1), stats, stack, !is_cut_node);
                    // stack.inside_null = false;

                    if let Some(res) = res.get_result() {
                        if res.score >= beta {
                            stats!(stats.null_prunes += 1);
                            self.pop_nnue(stack);
                            return ABPrune(beta, Prune::NullMove);
                        }
                    }

                    self.pop_nnue(stack);
                }
            }

        }


        /// Step 9. Lower depth for positions not in TT
        /// Stockfish does this, not sure why. Seems to work
        let depth = {
            let mut depth = depth;
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
            depth
        };

        // /// Reset killers for next ply
        // stack.killers_clear(ply + 2);

        let m_hashmove: Option<Move> = msi.map(|si| {
            let mv = si.best_move;
            // let mv = PackedMove::unpack(&[mv.0,mv.1]).unwrap().convert_to_move(ts, g);
            mv
        });

        /// Step 10. initialize move generator
        let mut movegen = MoveGen::new(ts, &g, m_hashmove, stack, depth, ply);
        // let mut movegen = MoveGen::new(ts, &g, m_hashmove, stack, depth, ply, stack.move_history.clone());

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
        let mut moves_searched_best = 0;
        let mut list = vec![];

        let mut captures_searched: ArrayVec<Move, 64> = ArrayVec::new();
        let mut quiets_searched: ArrayVec<Move, 64>   = ArrayVec::new();

        /// Step 11. Loop over moves
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
            // let gives_check = false;

            // /// Step _. Move Count pruning
            // if best_val.1 > -CHECKMATE_VALUE
            //     // && depth <= LMR_MIN_DEPTH
            //     && depth <= 8 // XXX: ??
            //     && moves_searched >= futility_move_count(improving, depth) {
            //         movegen.skip_quiets = true;
            //     }

            /// Step 12. Futility prune
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

            // /// Step _. Shallow pruning
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

            /// Step 13. Singular extension
            #[cfg(feature = "singular_extensions")]
            if let Some(si) = msi {
                if !is_root_node
                    && depth >= 7
                    && Some(mv) == m_hashmove
                    && si.node_type == Node::Lower // lower bound
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
                            if !is_pv_node
                                && res.score < sing_beta - 75 // XXX: sf magic
                                && stack.get_with(ply, |st| st.double_extensions).unwrap_or(0) <= 6
                            {
                                extensions = 2;
                                stats.sing_exts.two += 1;
                            } else {
                                stats.sing_exts.one += 1;
                            }

                        } else if sing_beta >= beta {

                            stats.sing_exts.prunes += 1;
                            return ABPrune(sing_beta, Prune::MultiCut);

                        } else if tt_eval >= beta {
                            stats.sing_exts.reduce += 1;
                            extensions -= 2;
                        }

                    }
                }
            // } else if (is_pv_node || is_cut_node)
            } else if is_pv_node // works way better
                && capture_or_promotion
                && moves_searched != 1
                && moves_searched != 0 // oops
            {
                // Capture extensions for pv node
                stats.sing_exts.capture += 1;
                extensions += 1;
            } else if gives_check
                && depth > 6
                && static_eval.map(|s| s.abs() > 100).unwrap_or(false)
            {
                // Check extensions
                stats.sing_exts.check += 1;
                extensions += 1;
            }

            /// Step 14. Make move
            let g2 = if let Some(g2) = self.make_move(ts, g, ply, mv, Some(zb0), stack) {
                g2
            } else {
                continue 'outer;
            };
            moves_searched += 1;

            next_depth += extensions;
            next_depth = next_depth.max(0);
            stack.update_double_extension(ply, extensions);

            /// Step 15. Recursively search for each move
            let res: ABResult = 'search: {
                let mut res: ABResult = ABResult::new_null();

                let mut lmr = self.cfg.late_move_reductions;

                let mut do_full_depth = true; // XXX: ??

                /// TODO: 
                // let history = 

                /// Step 16a. Skip LMR for good static exchanges
                if lmr && mv.filter_all_captures() && !mv.filter_promotion() {
                    // let see = g.static_exchange(&ts, mv).unwrap(); // XXX: g or g2?
                    let see = movegen.static_exchange_ge(mv, 1);
                    // let see = g.static_exchange_ge(ts, mv, 1);
                    /// Capture with good SEE: do not reduce
                    if see {
                        lmr = false;
                    }
                }

                /// Step 16b. Late Move Reductions
                if lmr
                    && !is_pv_node
                    && moves_searched >= (
                        if is_root_node { 2 + self.params.lmr_min_moves } else { self.params.lmr_min_moves })
                    // && next_depth >= LMR_MIN_DEPTH
                    && next_depth >= self.params.lmr_min_depth
                    && !mv.filter_promotion()
                    && !mv.filter_all_captures()
                    && !in_check // XXX: needed?
                    && !g2.state.in_check
                    // && !gives_check // redundant
                {
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
                    // assert!(depth_r >= 0);

                    let (a2,b2) = (-(alpha+1),-alpha); // XXX: ??

                    // trace!("search 0");
                    let res2 = -self.ab_search::<{NonPV}>(
                        ts, &g2, (depth_r,ply+1), (a2,b2), stats, stack, true);

                    res = if let Some(r) = res2.get_result_mv(mv) { r } else {
                        self.pop_nnue(stack);
                        continue 'outer;
                    };

                    if res.score <= alpha {
                        stats!(stats.lmrs += 1);
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

                /// Step 17a. Full depth search if no LMR and not PV Node's first search
                if do_full_depth {
                    let (a2,b2) = (-(alpha+1),-alpha); // XXX: ??
                    let res2 = -self.ab_search::<{NonPV}>(
                        ts, &g2, (next_depth,ply+1), (a2,b2), stats, stack, !is_cut_node);
                    res = if let Some(r) = res2.get_result_mv(mv) { r } else {
                        self.pop_nnue(stack);
                        continue 'outer;
                    };
                }

                /// Step 17b. Search PV with full window
                if is_pv_node && (moves_searched == 1 || res.score > alpha) {
                    let res2 = -self.ab_search::<{PV}>(
                        ts, &g2, (next_depth,ply+1), (-beta, -alpha), stats, stack, false);
                    res = if let Some(r) = res2.get_result_mv(mv) { r } else {
                        self.pop_nnue(stack);
                        continue 'outer;
                    };
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
                if is_pv_node {
                    moves_searched_best = moves_searched;
                }
            }

            /// Step 18. update alpha, beta, stats
            #[cfg(not(feature = "negamax_only"))]
            {
                if res.score >= beta { // Fail Soft
                    b = true;
                    // return beta;
                }

                if !b && best_val.1 > alpha {
                    alpha = best_val.1;
                }

                if !b {
                    if mv.filter_capture_or_promotion() {
                        captures_searched.try_push(mv).unwrap_or_else(|_| {});
                    } else {
                        quiets_searched.try_push(mv).unwrap_or_else(|_| {});
                    }
                }

                /// Fail high, update stats
                if b {

                    stack.update_history(
                        g, mv, res.score, beta, captures_searched, quiets_searched, ply, depth);

                    // if !mv.filter_all_captures() {
                    //     // #[cfg(feature = "history_heuristic")]
                    //     // stack.history.update(mv, g.state.side_to_move, ABStack::stat_bonus(depth));
                    //     // #[cfg(feature = "killer_moves")]
                    //     // stack.killers_store(ply, mv);
                    //     // #[cfg(feature = "countermove_heuristic")]
                    //     // if let Some(prev_mv) = g.last_move {
                    //     //     stack.counter_moves.insert_counter_move(prev_mv, mv, g.state.side_to_move);
                    //     //     // stack.counter_moves.insert_counter_move(prev_mv, mv);
                    //     // }
                    // }

                    if moves_searched <= 1 {
                        stats!(stats.beta_cut_first += 1);
                    } else {
                        stats!(stats.beta_cut_not_first += 1);
                    }

                    self.pop_nnue(stack);
                    break;
                }
            }

            self.pop_nnue(stack);
        }

        /// Step 19. Filter checkmate, stalemate
        if in_check
            && (moves_searched == 0 || best_val.0.is_none())
        {
            let score = CHECKMATE_VALUE - ply as Score;
            stats.leaves += 1;
            stats.checkmates += 1;
            let mv = g.last_move.unwrap();
            return ABSingle(ABResult::new_single(mv, -score));
        } else if moves_searched == 0
            || best_val.0.is_none()
        {
            // let score = -DRAW_VALUE + ply as Score;
            let score = draw_value(stats);
            stats.leaves += 1;
            stats.stalemates += 1;
            if let Some(mv) = g.last_move {
                // TODO: adjust stalemate value when winning/losing
                return ABSingle(ABResult::new_single(mv, score));
            } else {
                debug!("draw, but no g.last_move?");
                // return ABNone;
                // panic!();
                return ABSingle(ABResult::new_null_score(score));
            }
        }

        // if best_val.0.is_none() {
        //     eprintln!("moves_searched = {:?}", moves_searched);
        //     eprintln!("(alpha,beta) = {:?}", (alpha,beta));
        //     panic!("wat");
        // }

        /// XXX: stat padding by including nodes found in TT
        // stats!(stats.max_depth_search = stats.max_depth_search.max(ply as u8));
        stats!(stats.max_depth_search.max_mut(ply as u32));
        stats!(stats.inc_nodes_arr(ply));
        stats!(stats.nodes += 1);

        if is_pv_node {
            stats.ns_pv += 1;

            if let Some(mut x) = stats.nth_best_pv_mv.0.get_mut(moves_searched_best as usize - 1) {
                *x += 1;
            }

        } else if is_cut_node {
            stats.ns_cut += 1;
        } else {
            stats.ns_all += 1;
        }

        // /// syzygy ??
        // if is_pv_node {
        //     // best_val.1 = max_score;
        //     best_val.1 = best_val.1.min(max_score);
        // }

        /// Step 20. Update hash table and return
        match &best_val.0 {
            Some(res) => {

                let mv = res.mv.unwrap();

                // stack.update_history(
                //     g, mv, res.score, beta, captures_searched, quiets_searched, ply, depth);

                let bound = if res.score >= beta {
                    Node::Lower
                } else if is_pv_node {
                    Node::Exact
                } else {
                    Node::Upper
                };

                // if !is_root_node && current_node_type == Node::PV {
                // if current_node_type == Node::Exact {
                if bound == Node::Exact {
                    // stack.pvs.push(res.mv);
                    stack.pvs[ply as usize] = mv
                }

                if !is_root_node && Some(mv) != movegen.hashmove {
                // if !is_root_node {
                // if !is_root {

                    // assert_eq!(bound, current_node_type);

                    self.tt_insert_deepest(
                        g.zobrist,
                        static_eval,
                        SearchInfo::new(
                            // res.mv,
                            mv,
                            // res.moves.len() as u8,
                            depth - 1,
                            // depth,
                            // node_type,
                            // current_node_type,
                            bound,
                            res.score,
                            static_eval,
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


