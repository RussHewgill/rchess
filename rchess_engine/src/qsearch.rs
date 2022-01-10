
use crate::alphabeta::ABNodeType;
use crate::movegen::MoveGen;
use crate::sf_compat::NNUE4;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::explore::*;
use crate::tuning::*;
use crate::pawn_hash_table::*;
use crate::evmap_tables::*;

use std::cell::RefCell;
use std::sync::atomic::AtomicI16;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::{SeqCst,Relaxed};
use parking_lot::{Mutex,RwLock};

pub fn exhelper_once(
    ts:       &'static Tables,
    g:        &Game,
    side:     Color,
    ev_mid:   &EvalParams,
    ev_end:   &EvalParams,
    ph_rw:    Option<&PHTable>,
    nnue:     Option<NNUE4>,
) -> ExHelper {
    let mut cfg = ExConfig::default();
    cfg.eval_params_mid = ev_mid.clone();
    cfg.eval_params_end = ev_end.clone();

    #[cfg(not(feature = "lockless_hashmap"))]
    let (tt_r, tt_w) = evmap::Options::default()
        .with_hasher(FxBuildHasher::default())
        .construct();
    #[cfg(not(feature = "lockless_hashmap"))]
    let tt_rf = tt_w.factory();
    #[cfg(not(feature = "lockless_hashmap"))]
    let tt_w = Arc::new(Mutex::new(tt_w));

    let ph_rw = if let Some(t) = ph_rw {
        t.clone()
    } else {
        let ph_rw = PHTableFactory::new();
        ph_rw.handle()
    };

    let (tx,rx): (ExSender,ExReceiver) = crossbeam::channel::unbounded();

    let stop = Arc::new(AtomicBool::new(false));
    let best_mate = Arc::new(RwLock::new(None));
    let best_depth = Arc::new(AtomicI16::new(0));

    #[cfg(feature = "lockless_hashmap")]
    let ptr_tt = Arc::new(crate::lockless_map::TransTable::new_mb(256));

    let root_moves = MoveGen::gen_all(ts, g);

    let helper = ExHelper {
        id:              0,
        side,
        game:            g.clone(),
        root_moves:      RefCell::new(root_moves),
        stop,
        best_mate,
        #[cfg(feature = "syzygy")]
        syzygy:          None,
        nnue: nnue.map(|x| RefCell::new(x)),
        cfg,
        params: SParams::default(),
        best_depth,
        tx,
        #[cfg(feature = "lockless_hashmap")]
        ptr_tt,
        #[cfg(not(feature = "lockless_hashmap"))]
        tt_r,
        #[cfg(not(feature = "lockless_hashmap"))]
        tt_w,
        ph_rw,
        move_history: vec![],
    };

    helper
}

/// Search once
impl ExHelper {

    pub fn qsearch_once_mut(
        &mut self,
        ts:                       &Tables,
        g:                        &Game,
        mut stats:                &mut SearchStats,
    ) -> Score {
        let (alpha,beta) = (Score::MIN,Score::MAX);
        let (alpha,beta) = (alpha + 200,beta - 200);
        self.game = g.clone();
        self.side = g.state.side_to_move;
        let mut stack = ABStack::new();
        // self.qsearch(ts, g, (0,0), (alpha,beta), &mut stack, stats, ABNodeType::Root)
        // self.qsearch2::<{ABNodeType::Root}>(ts, g, (0,0), (alpha,beta), &mut stack, stats)
        unimplemented!()
    }

    pub fn qsearch_once(
        &self,
        ts:                       &Tables,
        g:                        &Game,
        mut stats:                &mut SearchStats,
    ) -> Score {
        let (alpha,beta) = (Score::MIN,Score::MAX);
        let (alpha,beta) = (alpha + 200,beta - 200);
        let mut stack = ABStack::new();
        // self.qsearch(ts, g, (0,0), (alpha,beta), &mut stack, stats, ABNodeType::Root)
        // self.qsearch2::<{ABNodeType::Root}>(ts, g, (0,0), (alpha,beta), &mut stack, stats)
        unimplemented!()
    }

}

/// Quiescence static eval
impl ExHelper {

    pub fn get_static_eval_qsearch(
        &self,
        ts:           &'static Tables,
        g:            &Game,
        ply:          Depth,
        mut stack:    &mut ABStack,
        meval:        Option<Score>,
        msi:          Option<SearchInfo>,
        allow_sp:     &mut bool,
    ) -> Score {

        if g.in_check() {
            *allow_sp = false;
            return Score::MIN;
        }

        if msi.is_some() || meval.is_some() {

            let mut eval = if let Some(eval) = meval { eval } else {
                self.eval_nn_or_hce(ts, g)
            };

            if let Some(si) = msi {
                let bb = if si.score > eval { Node::Lower } else { Node::Upper };
                if si.node_type == bb {
                    eval = si.score;
                }
            }
            return eval;
        } else if let Some(eval) = meval {
            return eval;
        }

        if let Some(nnue) = &self.nnue {
            let mut nn = nnue.borrow_mut();
            nn.evaluate(&g, true)
        } else {
            let stand_pat = self.cfg.evaluate(ts, g, &self.ph_rw);
            if g.state.side_to_move == Black { -stand_pat } else { stand_pat }
        }

    }

}

/// Quiescence static eval
impl ExHelper {
    pub fn best_case_move(
        &self,
        ts:         &'static Tables,
        g:          &Game,
    ) -> Score {
        const PCS: [Piece; 4] = [Queen,Rook,Bishop,Knight];

        let mut score = Pawn.score();
        let side = g.state.side_to_move;

        for pc in PCS {
            if g.state.material.get(pc, side) > 0 {
                score = pc.score();
                break;
            }
        }

        let rank7 = if side == White { BitBoard::mask_rank(6) } else { BitBoard::mask_rank(1) };

        let proms = g.get(Pawn, side) & rank7;
        if proms.is_not_empty() {
            score += Queen.score() - Pawn.score();
        }

        score
    }
}

/// Quiescence with TT
impl ExHelper {

    #[cfg(feature = "tt_in_qsearch")]
    pub fn qsearch<const NODE_TYPE: ABNodeType>(
        &self,
        ts:                       &'static Tables,
        g:                        &Game,
        (ply,qply,depth):         (Depth,Depth,Depth),
        (mut alpha, mut beta):    (Score,Score),
        mut stack:                &mut ABStack,
        mut stats:                &mut SearchStats,
    ) -> Score {
        use ABNodeType::*;

        stack.push_if_empty(g, ply);
        stack.with(ply, |st| st.material = g.state.material);

        stats.qt_nodes += 1;
        stats.q_max_depth = stats.q_max_depth.max(ply as u8);

        let in_check = g.in_check();
        let mut allow_stand_pat = true;

        /// check repetition
        if Self::has_cycle(ts, g, stats, stack) || ply >= MAX_SEARCH_PLY {
            return draw_value(stats);
        }

        /// TODO: ??
        // let tt_depth = 1;
        let tt_depth = if in_check {
            1
        } else if depth >= DEPTH_QSEARCH_CHECKS {
            DEPTH_QSEARCH_CHECKS
        } else {
            DEPTH_QSEARCH_NOCHECKS
        };

        /// TT Lookup
        let (meval,msi): (Option<Score>,Option<SearchInfo>) = self.check_tt2(ts, g.zobrist, tt_depth, stats);

        /// Check for returnable TT value
        if let Some(si) = msi {

            if NODE_TYPE != PV
                && si.depth_searched >= tt_depth
                && si.node_type == if si.score >= beta { Node::Lower } else { Node::Upper }
            {
                stats.qt_tt_returns += 1;
                return si.score;
            }

            // if si.node_type == Node::Exact
            //     || (si.node_type == Node::Lower && si.score >= beta)
            //     || (si.node_type == Node::Upper && si.score <= alpha) {
            //         stats.qt_tt_returns += 1;
            //         return si.score;
            //     }

        }

        let stand_pat = self.get_static_eval_qsearch(ts, g, ply, stack, meval, msi, &mut allow_stand_pat);

        /// early halt
        if allow_stand_pat && self.stop.load(Relaxed) { return stand_pat; }

        if allow_stand_pat && stand_pat >= beta {

            if meval.is_none() && msi.is_none() {
                self.tt_insert_deepest_eval(g.zobrist, Some(stand_pat));
            }

            return beta; // fail hard
            // return stand_pat; // fail soft // XXX: is faster ??
        }

        /// TODO: Delta Pruning
        if self.params.qs_delta_margin.max(self.best_case_move(ts, g)) < alpha - stand_pat {
            // return alpha; // XXX: doesn't work at all?
            return stand_pat;
        }

        // if NODE_TYPE == PV && stand_pat > alpha {
        // if stand_pat > alpha {
        if allow_stand_pat && stand_pat > alpha {
            alpha = stand_pat;
        }

        /// TODO: use hashmove or not?
        // let m_hashmove = msi.map(|si| si.best_move);
        // let mut movegen = MoveGen::new_qsearch(ts, g, m_hashmove, stack, qply);
        let mut movegen = MoveGen::new_qsearch(ts, g, None, stack, qply);

        let mut best_score = Score::MIN;
        let mut best_move: Option<Move> = None;

        while let Some(mv) = movegen.next(stack) {
            if let Some(g2) = self.make_move(ts, g, ply, mv, None, stack) {

                let see = movegen.static_exchange_ge(mv, 1);
                if !see {
                    self.pop_nnue(stack);
                    continue;
                }

                let score = -self.qsearch::<{NODE_TYPE}>(
                    &ts, &g2, (ply + 1,qply + 1, depth - 1), (-beta, -alpha), stack, stats);

                if score >= beta && allow_stand_pat {
                    self.pop_nnue(stack);
                    return beta; // fail hard // XXX: works better, but shouldn't ??
                    // return stand_pat; // fail soft
                }

                if score > best_score {
                    best_score = score;

                    if score > alpha {
                        best_move = Some(mv);
                        alpha = score;
                        // // XXX: why?
                        // if score < beta {
                        //     alpha = score;
                        // }
                    }

                }

                // /// XXX: from stockfish
                // if score > best_score {
                //     best_score = score;
                //     if score > alpha {
                //         best_move = Some(mv);
                //         if NODE_TYPE == PV && score < beta {
                //             alpha = score;
                //         } else {
                //             self.pop_nnue(stack);
                //             break;
                //         }
                //     }
                // }

                self.pop_nnue(stack);
            }
        }

        if in_check && best_move.is_none() {
            let score = CHECKMATE_VALUE - ply as Score;
            // return -score; // XXX: backward ?
            return score;
        }

        if let Some(mv) = best_move {
            let eval = if in_check { Some(stand_pat) } else { None };
            let bound = if best_score >= beta {
                Node::Lower
            } else if NODE_TYPE == PV {
                Node::Exact
            } else {
                Node::Upper
            };
            self.tt_insert_deepest(
                g.zobrist,
                eval,
                SearchInfo::new(
                    mv,
                    tt_depth,
                    bound,
                    best_score,
                    eval
                )
            );
        }

        alpha
        // stand_pat
    }

    // #[cfg(feature = "tt_in_qsearch")]
    #[cfg(feature = "nope")]
    pub fn qsearch2<const NODE_TYPE: ABNodeType>(
        &self,
        ts:                       &'static Tables,
        g:                        &Game,
        (ply,qply):               (Depth,Depth),
        (mut alpha, mut beta):    (Score,Score),
        mut stack:                &mut ABStack,
        mut stats:                &mut SearchStats,
    ) -> Score {
        use ABNodeType::*;

        // stack.push_if_empty(g, ply);
        // stack.with(ply, |st| st.material = g.state.material);

        let in_check = g.in_check();
        let mut allow_stand_pat = true;

        /// TODO: ??
        // let tt_depth = ply;
        // let tt_depth = if in_check { 1 } else { 0 };
        let tt_depth = 1;

        // /// TT Lookup
        // #[cfg(feature = "tt_in_qsearch")]
        // let (meval,msi): (Option<Score>,Option<SearchInfo>) = self.check_tt2(ts, g.zobrist, tt_depth, stats);

        // /// Check for returnable TT value
        // #[cfg(feature = "tt_in_qsearch")]
        // if let Some(si) = msi {
        //     if NODE_TYPE != PV
        //     // && si.node_type == if si.score >= beta { Node::Lower } else { Node::Upper }
        //     {
        //         stats.qt_tt_returns += 1;
        //         return si.score;
        //     }
        // }

        let stand_pat = if in_check {
            allow_stand_pat = false;
            Score::MIN
        } else if let Some(nnue) = &self.nnue {
            let mut nn = nnue.borrow_mut();
            nn.evaluate(&g, true)
        } else {
            let stand_pat = self.cfg.evaluate(ts, g, &self.ph_rw);
            if g.state.side_to_move == Black { -stand_pat } else { stand_pat }
        };

        /// early halt
        // if self.stop.load(Relaxed) { return stand_pat; }
        if allow_stand_pat && self.stop.load(Relaxed) { return stand_pat; }

        /// check repetition
        if Self::has_cycle(ts, g, stats, stack) {
            let score = draw_value(stats);
            // return ABSingle(ABResult::new_single(g.last_move.unwrap(), score));
            return score;
        }

        stats.qt_nodes += 1;

        // if ply > stats.q_max_depth {
        //     eprintln!("new max depth = {:?}", ply);
        // }

        stats!(stats.q_max_depth = stats.q_max_depth.max(ply as u8));

        // if stand_pat >= beta {
        if allow_stand_pat && stand_pat >= beta {

            // self.tt_insert_deepest_eval(g.zobrist, Some(stand_pat));

            return beta; // fail hard
            // return stand_pat; // fail soft
        }

        // if stand_pat > alpha {
        if allow_stand_pat && stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut movegen = MoveGen::new_qsearch(ts, g, None, stack, qply);

        // /// TODO: Delta Pruning
        // let mut big_delta = Queen.score();
        // if moves.iter().any(|mv| mv.filter_promotion()) {
        //     big_delta += Queen.score() - Pawn.score();
        // }
        // if stand_pat < alpha - big_delta {
        //     // trace!("qsearch: delta prune: {}", alpha);
        //     return alpha;
        // }

        // let mut moves_searched = 0;

        // let mut best_move  = None;
        // let mut best_score = Score::MIN;

        while let Some(mv) = movegen.next(stack) {
            if let Some(g2) = self.make_move(ts, g, ply, mv, None, stack) {

                // moves_searched += 1;

                let see = movegen.static_exchange_ge(mv, 1);
                if !see {
                    self.pop_nnue(stack);
                    continue;
                }

                let score = -self.qsearch::<{NODE_TYPE}>(
                    &ts, &g2, (ply + 1,qply + 1), (-beta, -alpha), stack, stats);

                if score >= beta && allow_stand_pat {
                    self.pop_nnue(stack);
                    return beta; // fail hard
                    // return stand_pat; // fail soft
                }

                if score > alpha {
                    alpha = score;
                }

                self.pop_nnue(stack);
            }
        }

        // if let Some(mv) = best_move {
        //     let eval = if in_check { Some(stand_pat) } else { None };
        //     let bound = if best_score >= beta {
        //         Node::Lower
        //     } else if NODE_TYPE == PV {
        //         Node::Exact
        //     } else {
        //         Node::Upper
        //     };
        //     self.tt_insert_deepest(
        //         g.zobrist,
        //         eval,
        //         SearchInfo::new(
        //             mv,
        //             tt_depth,
        //             bound,
        //             best_score,
        //             eval
        //         )
        //     );
        // }

        alpha
    }

}

/// Quiescence
impl ExHelper {

    #[cfg(not(feature = "tt_in_qsearch"))]
    pub fn qsearch<const NODE_TYPE: ABNodeType>(
        &self,
        ts:                       &'static Tables,
        g:                        &Game,
        (ply,qply,depth):         (Depth,Depth,Depth),
        (mut alpha, mut beta):    (Score,Score),
        mut stack:                &mut ABStack,
        mut stats:                &mut SearchStats,
    ) -> Score {
        use ABNodeType::*;

        let is_pv_node = NODE_TYPE != NonPV;

        assert!(alpha < beta);
        assert!(is_pv_node || (alpha == beta - 1));
        assert!(depth <= 0);

        stats.qt_nodes += 1;
        stats.q_max_depth = stats.q_max_depth.max(ply as u8);

        /// check repetition
        if Self::has_cycle(ts, g, stats, stack) || ply >= MAX_SEARCH_PLY {
            return draw_value(stats);
        }

        let stand_pat = if let Some(nnue) = &self.nnue {
            let mut nn = nnue.borrow_mut();
            nn.evaluate(&g, true)
        } else {
            let stand_pat = self.cfg.evaluate(ts, g, &self.ph_rw);
            if g.state.side_to_move == Black { -stand_pat } else { stand_pat }
        };

        /// early halt
        if self.stop.load(Relaxed) { return stand_pat; }

        let mut allow_stand_pat = true;

        if stand_pat >= beta {
            return beta; // fail hard
            // return stand_pat; // fail soft
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut movegen = MoveGen::new_qsearch(ts, g, None, stack, qply);

        if qply > 0 { movegen.skip_quiets = true; }

        /// TODO: Delta Pruning
        if !g.in_check()
            && g.state.phase < self.params.qs_delta_max_phase
            && self.params.qs_delta_margin.max(self.best_case_move(ts, g)) < alpha - stand_pat
        {
            stats.qs_delta_prunes += 1;
            // return alpha; // XXX: doesn't work at all?
            return stand_pat;
            // return beta; // XXX: much faster, but shouldn't work
        }

        // let mut moves_searched = 0;
        // let mut best_move: Option<Move> = None;

        while let Some(mv) = movegen.next(stack) {

            let see = movegen.static_exchange_ge(mv, 1);
            if !see {
                continue;
            }

            if let Some(g2) = self.make_move(ts, g, ply, mv, None, stack) {

                // moves_searched += 1;

                // let see = movegen.static_exchange_ge(mv, 1);
                // if !see {
                //     self.pop_nnue(stack);
                //     continue;
                // }

                let score = -self.qsearch::<{NODE_TYPE}>(
                    &ts, &g2, (ply + 1,qply + 1,depth - 1), (-beta, -alpha), stack, stats);

                if score >= beta && allow_stand_pat {
                    self.pop_nnue(stack);
                    return beta; // fail hard
                    // return stand_pat; // fail soft
                }

                if score > alpha {
                    // best_move = Some(mv);
                    alpha = score;
                }

                self.pop_nnue(stack);
            }
        }

        // if g.in_check() && best_move.is_none() {
        //     let score = CHECKMATE_VALUE - ply as Score;
        //     // return -score; // XXX: backward ?
        //     return score;
        // }

        alpha
    }

}



