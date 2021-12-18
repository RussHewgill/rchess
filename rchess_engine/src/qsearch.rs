
use crate::alphabeta::ABNodeType;
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
    };

    helper
}

// pub fn qsearch_once2(
//     ts:       &Tables,
//     g:        &Game,
//     side:     Color,
//     ev_mid:   &EvalParams,
//     ev_end:   &EvalParams,
//     ph_rw:    Option<&PHTable>,
// ) -> Score {

//     let mut cfg = ExConfig::default();
//     cfg.eval_params_mid = ev_mid.clone();
//     cfg.eval_params_end = ev_end.clone();

//     let (tt_r, tt_w) = evmap::Options::default()
//         .with_hasher(FxBuildHasher::default())
//         .construct();
//     let tt_rf = tt_w.factory();
//     let tt_w = Arc::new(Mutex::new(tt_w));

//     // let (ph_rf,ph_w) = new_hash_table();
//     let ph_rw = if let Some(t) = ph_rw {
//         t.clone()
//     } else {
//         let ph_rw = PHTableFactory::new();
//         ph_rw.handle()
//     };

//     let (tx,rx): (ExSender,ExReceiver) = crossbeam::channel::unbounded();

//     let stop = Arc::new(AtomicBool::new(false));
//     let best_mate = Arc::new(RwLock::new(None));
//     let best_depth = Arc::new(AtomicU8::new(0));

//     let helper = ExHelper {
//         id:              0,
//         side,
//         game:            g.clone(),
//         stop,
//         best_mate,
//         #[cfg(feature = "syzygy")]
//         syzygy:          None,
//         cfg,
//         best_depth,
//         tx,
//         tt_r,
//         tt_w,
//         ph_rw,
//     };

//     let (alpha,beta) = (i32::MIN,i32::MAX);
//     let (alpha,beta) = (alpha + 200,beta - 200);
//     let mut stats = SearchStats::default();

//     let score = helper.qsearch(
//         ts,
//         g,
//         (0,0),
//         (alpha, beta),
//         &mut stats);

//     if side == Black {
//         -score
//     } else {
//         score
//     }

// }

/// Quiescence
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
        self.qsearch(ts, g, (0,0), (alpha,beta), stats, ABNodeType::Root)
    }

    pub fn qsearch_once(
        &self,
        ts:                       &Tables,
        g:                        &Game,
        mut stats:                &mut SearchStats,
    ) -> Score {
        let (alpha,beta) = (Score::MIN,Score::MAX);
        let (alpha,beta) = (alpha + 200,beta - 200);
        self.qsearch(ts, g, (0,0), (alpha,beta), stats, ABNodeType::Root)
    }

    /// alpha = the MINimum score that the MAXimizing player is assured of
    /// beta  = the MAXimum score that the MINimizing player is assured of
    #[allow(unused_doc_comments)]
    // #[allow(unreachable_code)]
    #[allow(unused_assignments)]
    pub fn qsearch(
        &self,
        ts:                       &Tables,
        g:                        &Game,
        (ply,qply):               (Depth,Depth),
        (mut alpha, mut beta):    (Score,Score),
        mut stats:                &mut SearchStats,
        node_type:                ABNodeType,
    ) -> Score {
        // trace!("qsearch, {:?} to move, ply {}, a/b: {:?},{:?}",
        //        g.state.side_to_move, ply, alpha, beta);

        let stand_pat = if let Some(nnue) = &self.nnue {
            // let mut nn: &mut NNUE4 = nnue.borrow_mut();
            let mut nn = nnue.borrow_mut();

            // let mut nn2 = nn.clone();
            // nn2.ft.reset_accum(g);
            // let v2 = nn2.evaluate(g, true);

            let v = nn.evaluate(&g, true);

            // // assert_eq!(v, v2);
            // if v != v2 {
            //     eprintln!("g.to_fen() = {:?}", g.to_fen());
            //     eprintln!("g = {:?}", g);
            //     eprintln!("v  = {:?}", v);
            //     eprintln!("v2 = {:?}", v2);
            //     panic!("v != v2");
            // }

            // nn.ft.accum.needs_refresh = [true; 2];
            // let v = nn.evaluate(&g, true, true); // XXX: slow, always refresh

            v
            // unimplemented!()
        } else {
            let stand_pat = self.cfg.evaluate(ts, g, &self.ph_rw);
            if g.state.side_to_move == Black { -stand_pat } else { stand_pat }
        };

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

        let mut moves = if qply > QS_RECAPS_ONLY && !g.in_check() && g.state.last_capture.is_some() {
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

        // /// No change in performance, but easier to read in flamegraph
        // let mut gs: Vec<(Move,Zobrist,Option<(SICanUse,SearchInfo)>)> = Vec::with_capacity(moves.len());
        // for mv in moves.into_iter() {
        //     let zb = g.zobrist.update_move_unchecked(ts, g, mv);
        //     let tt = self.check_tt_negamax(&ts, zb, depth, &mut stats);
        //     gs.push((mv,zb,tt));
        // }

        order_mvv_lva(&mut moves);
        // self.order_moves(ts, g, ply, &mut tracking, &mut gs[..]);

        // let ms = moves.into_iter()
        //     .flat_map(|m| g.make_move_unchecked(&ts, m).ok().map(|x| (m,x)));

        // for (mv,g2) in ms {
        for mv in moves.into_iter() {

            // let k0 = {
            //     let nn = &self.nnue.as_ref().unwrap();
            //     let nn = nn.borrow();
            //     // nn.ft.accum.stack_delta.len()
            //     nn.ft.accum.make_copy()
            // };

            // if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
            if let Some(g2) = self.make_move(ts, g, mv, None) {

                // trace!("qsearch: mv = {:?}, g = {:?}\n, g2 = {:?}", mv, g, g2);
                // trace!("qsearch: mv = {:?}", mv);

                if let Some(see) = g.static_exchange(&ts, mv) {
                    if see < 0 {
                        // trace!("fen = {}", g.to_fen());
                        // trace!("qsearch: SEE negative: {} {:?}", see, g);
                        // trace!("qsearch: SEE negative: {}", see);
                        self.pop_nnue();
                        continue;
                    }
                }

                let score = -self.qsearch(
                    &ts, &g2, (ply + 1,qply + 1), (-beta, -alpha), &mut stats, node_type);

                if score >= beta && allow_stand_pat {
                    // trace!("qsearch returning beta 1: {:?}", beta);
                    self.pop_nnue();
                    return beta; // fail hard
                    // return stand_pat; // fail soft
                }

                if score > alpha {
                    alpha = score;
                }

                self.pop_nnue();

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

