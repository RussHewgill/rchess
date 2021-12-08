
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

    let (tt_r, tt_w) = evmap::Options::default()
        .with_hasher(FxBuildHasher::default())
        .construct();
    let tt_rf = tt_w.factory();
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

    let helper = ExHelper {
        id:              0,
        side,
        game:            g.clone(),
        stop,
        best_mate,
        #[cfg(feature = "syzygy")]
        syzygy:          None,
        #[cfg(feature = "nnue")]
        // nnue: nnue.map(|x| RefCell::new(x.clone())),
        nnue: nnue.map(|x| RefCell::new(x)),
        cfg,
        best_depth,
        tx,
        tt_r,
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
        self.qsearch(ts, g, (0,0), (alpha,beta), stats)
    }

    pub fn qsearch_once(
        &self,
        ts:                       &Tables,
        g:                        &Game,
        mut stats:                &mut SearchStats,
    ) -> Score {
        let (alpha,beta) = (Score::MIN,Score::MAX);
        let (alpha,beta) = (alpha + 200,beta - 200);
        self.qsearch(ts, g, (0,0), (alpha,beta), stats)
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
    ) -> Score {
        // trace!("qsearch, {:?} to move, ply {}, a/b: {:?},{:?}",
        //        g.state.side_to_move, ply, alpha, beta);

        let stand_pat = if let Some(nnue) = &self.nnue {
            // let mut nn: &mut NNUE4 = nnue.borrow_mut();
            let mut nn = nnue.borrow_mut();
            let v = nn.evaluate(&g, true, false);
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

        order_mvv_lva(&mut moves);

        // moves.par_sort_by(|a,b| {
        // });
        // moves.sort_by_cached_key(|m| g.static_exchange(&ts, *m));
        // moves.reverse();

        // let ms = moves.into_iter()
        //     .flat_map(|m| g.make_move_unchecked(&ts, m).ok().map(|x| (m,x)));

        // for (mv,g2) in ms {
        for mv in moves.into_iter() {

            if let Ok(g2) = g.make_move_unchecked(&ts, mv) {

                // trace!("qsearch: mv = {:?}, g = {:?}\n, g2 = {:?}", mv, g, g2);
                // trace!("qsearch: mv = {:?}", mv);

                if let Some(see) = g.static_exchange(&ts, mv) {
                    if see < 0 {
                        // trace!("fen = {}", g.to_fen());
                        // trace!("qsearch: SEE negative: {} {:?}", see, g);
                        // trace!("qsearch: SEE negative: {}", see);
                        continue;
                    }
                }

                let score = -self.qsearch(&ts, &g2, (ply + 1,qply + 1), (-beta, -alpha), &mut stats);

                if score >= beta && allow_stand_pat {
                    // trace!("qsearch returning beta 1: {:?}", beta);
                    return beta; // fail hard
                    // return stand_pat; // fail soft
                }

                if score > alpha {
                    alpha = score;
                }

            }

        }

        // trace!("qsearch returning alpha 0: {:?}", alpha);
        alpha
    }

}

