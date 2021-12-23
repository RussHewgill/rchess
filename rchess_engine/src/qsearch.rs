
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
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::SeqCst;
use parking_lot::{Mutex,RwLock};

pub fn exhelper_once(
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
    let best_depth = Arc::new(AtomicU8::new(0));

    #[cfg(feature = "lockless_hashmap")]
    let ptr_tt = Arc::new(crate::lockless_map::TransTable::new_mb(256));

    let helper = ExHelper {
        id:              0,
        side,
        game:            g.clone(),
        stop,
        best_mate,
        #[cfg(feature = "syzygy")]
        syzygy:          None,
        nnue: nnue.map(|x| RefCell::new(x)),
        cfg,
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

/// Quiescence 2
impl ExHelper {

    #[allow(unused_doc_comments)]
    pub fn qsearch<const NODE_TYPE: ABNodeType>(
        &self,
        ts:                       &'static Tables,
        g:                        &Game,
        (ply,qply):               (Depth,Depth),
        (mut alpha, mut beta):    (Score,Score),
        mut stack:                &mut ABStack,
        mut stats:                &mut SearchStats,
    ) -> Score {

        let stand_pat = if let Some(nnue) = &self.nnue {
            let mut nn = nnue.borrow_mut();
            nn.evaluate(&g, true)
        } else {
            let stand_pat = self.cfg.evaluate(ts, g, &self.ph_rw);
            if g.state.side_to_move == Black { -stand_pat } else { stand_pat }
        };

        /// early halt
        if self.stop.load(SeqCst) { return stand_pat; }

        stats.qt_nodes += 1;

        // if ply > stats.q_max_depth {
        //     eprintln!("new max depth = {:?}", ply);
        // }

        stats!(stats.q_max_depth = stats.q_max_depth.max(ply as u8));

        let mut allow_stand_pat = true;

        if stand_pat >= beta {
            return beta; // fail hard
            // return stand_pat; // fail soft
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut movegen = MoveGen::new_qsearch(ts, g, None, stack, qply);

        /// TODO: Delta Pruning
        // let mut big_delta = Queen.score();
        // if moves.iter().any(|mv| mv.filter_promotion()) {
        //     big_delta += Queen.score() - Pawn.score();
        // }
        // if stand_pat < alpha - big_delta {
        //     // trace!("qsearch: delta prune: {}", alpha);
        //     return alpha;
        // }

        // let mut moves_searched = 0;

        while let Some(mv) = movegen.next(stack) {
            if let Some(g2) = self.make_move(ts, g, mv, None, stack) {

                // moves_searched += 1;

                if let Some(see) = movegen.static_exchange(mv) {
                    if see < 0 {
                        self.pop_nnue(stack);
                        continue;
                    }
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

        alpha
    }

}

/// Quiescence old
impl ExHelper {

    /// alpha = the MINimum score that the MAXimizing player is assured of
    /// beta  = the MAXimum score that the MINimizing player is assured of
    #[allow(unused_doc_comments)]
    // #[allow(unreachable_code)]
    #[allow(unused_assignments)]
    // pub fn qsearch(
    pub fn qsearch2<const NODE_TYPE: ABNodeType>(
        &self,
        ts:                       &'static Tables,
        g:                        &Game,
        (ply,qply):               (Depth,Depth),
        (mut alpha, mut beta):    (Score,Score),
        mut stack:                &mut ABStack,
        mut stats:                &mut SearchStats,
        // node_type:                ABNodeType,
    ) -> Score {
        // trace!("qsearch, {:?} to move, ply {}, a/b: {:?},{:?}",
        //        g.state.side_to_move, ply, alpha, beta);

        let stand_pat = if let Some(nnue) = &self.nnue {
            let mut nn = nnue.borrow_mut();
            nn.evaluate(&g, true)
        } else {
            let stand_pat = self.cfg.evaluate(ts, g, &self.ph_rw);
            if g.state.side_to_move == Black { -stand_pat } else { stand_pat }
        };

        // return stand_pat;

        if self.stop.load(SeqCst) {
            return stand_pat;
        }

        // trace!("qsearch, stand_pat = {:?}", stand_pat);

        let mut allow_stand_pat = true;

        stats.qt_nodes += 1;
        stats!(stats.q_max_depth = stats.q_max_depth.max(ply as u8));

        if allow_stand_pat && stand_pat >= beta {
            // trace!("qsearch returning beta 0: {:?}, sp = {}", beta, stand_pat);
            return beta; // fail hard
            // return stand_pat; // fail soft
        }

        if stand_pat > alpha {
            alpha = stand_pat;
            // trace!("qsearch new alpha: {:?}", alpha);
        }

        let mut moves = if qply > QS_RECAPS_ONLY
            && !g.in_check()
            && g.state.last_capture.is_some() {
                let cap = g.state.last_capture.unwrap();
                match g.search_all_single_to(&ts, cap, None, true) {
                    Outcome::Moves(ms) => {
                        // stats.qt_hits += 1;
                        ms
                    },
                    _                  => vec![],
                }
        } else if qply > QS_RECAPS_ONLY && !g.in_check() {
            // stats.qt_misses += 1;
            return stand_pat;
        } else {
            // stats.qt_misses += 1;
            match g.search_only_captures(&ts) {
                Outcome::Moves(ms) => ms,
                _                  => {
                    // trace!("qsearch no legal capture moves");
                    if !g.in_check() {
                        allow_stand_pat = false;
                        vec![]
                    } else {
                        match g.search_all(&ts) {
                            Outcome::Moves(ms) => {
                                allow_stand_pat = false;
                                ms
                            },
                            Outcome::Checkmate(c) => {
                                // trace!("qsearch checkmate {:?}: ply {}, {}\n{:?}", c, ply, qply, g);
                                let score = CHECKMATE_VALUE - ply as Score;

                                // if g.state.side_to_move == Black {
                                //     return -score;
                                // } else {
                                //     return score;
                                // }

                                return -score;
                                // vec![]
                            },
                            Outcome::Stalemate => {
                                // trace!("qsearch stalemate");
                                // let score = -200_000_000 + ply as Score;
                                // return -score;
                                vec![]
                            },
                        }
                    }
                }
            }

        };

        /// Delta Pruning
        let mut big_delta = Queen.score();
        if moves.iter().any(|mv| mv.filter_promotion()) {
            big_delta += Queen.score() - Pawn.score();
        }
        if stand_pat < alpha - big_delta {
            // trace!("qsearch: delta prune: {}", alpha);
            return alpha;
        }

        // TODO: hash table lookup
        // debug!("")

        order_mvv_lva(&mut moves);

        // let mut movegen = MoveGen::new_qsearch(ts, g, None, qply);

        for mv in moves.into_iter() {
        // while let Some(mv) = movegen.next(stack) {

            // let k0 = {
            //     let nn = &self.nnue.as_ref().unwrap();
            //     let nn = nn.borrow();
            //     // nn.ft.accum.stack_delta.len()
            //     nn.ft.accum.make_copy()
            // };

            // if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
            if let Some(g2) = self.make_move(ts, g, mv, None, stack) {

                // trace!("qsearch: mv = {:?}, g = {:?}\n, g2 = {:?}", mv, g, g2);
                // trace!("qsearch: mv = {:?}", mv);

                if let Some(see) = g.static_exchange(&ts, mv) {
                    if see < 0 {
                        // trace!("fen = {}", g.to_fen());
                        // trace!("qsearch: SEE negative: {} {:?}", see, g);
                        // trace!("qsearch: SEE negative: {}", see);
                        self.pop_nnue(stack);
                        continue;
                    }
                }

                // let score = -self.qsearch(
                //     &ts, &g2, (ply + 1,qply + 1), (-beta, -alpha), stack, stats, node_type);
                let score = -self.qsearch2::<{NODE_TYPE}>(
                    &ts, &g2, (ply + 1,qply + 1), (-beta, -alpha), stack, stats);

                if score >= beta && allow_stand_pat {
                    // trace!("qsearch returning beta 1: {:?}", beta);
                    self.pop_nnue(stack);
                    return beta; // fail hard
                    // return stand_pat; // fail soft
                }

                if score > alpha {
                    alpha = score;
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

            }

        }

        // trace!("qsearch returning alpha 0: {:?}", alpha);
        alpha
    }

}

