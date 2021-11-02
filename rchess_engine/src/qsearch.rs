
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::explore::*;

/// Quiescence
impl Explorer {

    /// alpha = the MINimum score that the MAXimizing player is assured of
    /// beta  = the MAXimum score that the MINimizing player is assured of
    #[allow(unused_doc_comments)]
    // #[allow(unreachable_code)]
    pub fn quiescence(
        &self,
        ts:             &Tables,
        g:              &Game,
        // mut ms:         Vec<Move>,
        k:              i16,
        mut alpha:      i32,
        mut beta:       i32,
        maximizing:     bool,
        mut stats:      &mut SearchStats,
    ) -> Score {
        unimplemented!()
    }

    /// alpha = the MINimum score that the MAXimizing player is assured of
    /// beta  = the MAXimum score that the MINimizing player is assured of
    #[allow(unused_doc_comments)]
    pub fn quiescence2(
        &self,
        ts:             &Tables,
        g:              &Game,
        // mut ms:         Vec<Move>,
        k:              i16,
        mut alpha:      i32,
        mut beta:       i32,
        maximizing:     bool,
        mut stats:      &mut SearchStats,
    ) -> Score {

        stats.qt_nodes += 1;

        let in_check = g.state.checkers.is_not_empty();

        let stand_pat = g.evaluate(&ts).sum();
        let mut stand_pat = if self.side == Black { -stand_pat } else { stand_pat };
        // return stand_pat;

        // if k > self.max_depth as i16 * 2 {
        //     return stand_pat;
        // }

        if maximizing {
            trace!("quiescence max ({}): a/b = {:?}, {:?} = {:?}", k, alpha, beta, stand_pat);
        } else {
            trace!("quiescence min ({}): a/b = {:?}, {:?} = {:?}", k, alpha, beta, stand_pat);
        }

        if maximizing {
            /// lower bound is better than the best opponent can get earlier in tree
            /// beta cutoff
            /// opponent will never make this move because better options are available
            if !in_check && stand_pat >= beta {
                // trace!("QS returning stand_pat: {}", stand_pat);
                return stand_pat;
            }
        } else {
            if !in_check && stand_pat <= alpha {
                // trace!("QS returning stand_pat: {}", stand_pat);
                return stand_pat;
            }
        }

        // /// Delta prune
        // let mut big_delta = Queen.score();
        // if m0.filter_promotion() {
        //     big_delta += Queen.score() - Pawn.score();
        // }
        // if maximizing {
        //     if stand_pat >= (beta + big_delta) {
        //         return stand_pat;
        //     }
        // }
        // // return stand_pat; // correct

        if maximizing {
            if alpha < stand_pat {
                alpha = stand_pat;
                // trace!("QS new alpha 0: {}", alpha);
            }
        } else {
            if beta < stand_pat {
                beta = stand_pat;
                // trace!("QS new beta 0: {}", alpha);
            }
        }

        let mut val = if maximizing { i32::MIN } else { i32::MAX };

        // for m in ms.iter() {
        //     if !m.filter_all_captures() {
        //         panic!("non capture in QS: {:?}, g = {:?}\n{:?}", m, g, g.zobrist);
        //     }
        // }

        let mut ms = match g.search_only_captures(&ts) {
            Outcome::Moves(ms) => ms,
            _                  => {
                if in_check {
                    trace!("QS in check, no capture evasions: {:?}\n{}", g, g.to_fen());
                    match g.search_all(&ts) {
                        Outcome::Moves(ms) => {
                            ms
                        },
                        Outcome::Checkmate(win) => {
                            let score = 100_000_000 - k as Score;
                            if self.side == win {
                                return score;
                            } else {
                                return -score;
                            }
                        }
                        Outcome::Stalemate => {
                            return 0;
                        }
                    }
                } else {
                    vec![]
                }
            },
        };

        order_mvv_lva(&mut ms);

        // ms.par_sort_by(|a,b| {
        //     _order_mvv_lva(a, b).reverse()
        //     // _order_mvv_lva(&a.1, &b.1)
        // });

        // let ms0 = ms.into_iter()
        //     .flat_map(|mv| g.make_move_unchecked(&ts, mv).ok().map(|g2| (mv,g2)));

        // for (mv,g2) in ms0 {
        for mv in ms.into_iter() {
            if let Ok(g2) = g.make_move_unchecked(&ts, mv) {

                // let score = match g2.search_only_captures(&ts) {
                //     Outcome::Moves(ms2)    => {
                //         self.quiescence(
                //             &ts, &g2, ms2, k + 1, alpha, beta, !maximizing, &mut stats)
                //     },
                //     Outcome::Checkmate(_) => {
                //         let score = 100_000_000 - k as Score;
                //         if maximizing { -score } else { score }
                //     },
                //     Outcome::Stalemate    => {
                //         let score = -100_000_000 + k as Score;
                //         score
                //     },
                // };

                let score = self.quiescence2(
                    &ts, &g2, k + 1, alpha, beta, !maximizing, &mut stats);

                if maximizing {
                    val   = Score::max(val, score);
                    alpha = Score::max(alpha, val);
                    if val >= beta {
                        // trace!("QS breaking max: val, beta: {}, {}", val, beta);
                        break;
                    }
                } else {
                    val = Score::min(val, score);
                    beta = Score::min(beta, score);
                    if val <= alpha {
                        // trace!("QS breaking min: val, beta: {}, {}", val, beta);
                        // trace!("mv = {:?}", mv);
                        break;
                    }
                }
            }
        }

        // trace!("QS returning alpha: {}", alpha);
        alpha
    }

    #[allow(unused_doc_comments)]
    #[allow(unreachable_code)]
    pub fn quiescence3(
        &self,
        ts:             &Tables,
        g:              &Game,
        ms:             Vec<Move>,
        k:              i16,
        mut alpha:      i32,
        mut beta:       i32,
        maximizing:     bool,
        mut stats:      &mut SearchStats,
        // m0:             Move,
    ) -> Score {
        // trace!("quiescence {}", k);
        // eprintln!("quiescence starting {}", k);

        stats.qt_nodes += 1;
        // let stand_pat = g.evaluate(&ts).sum();
        // stats.leaves += 1;
        // return stand_pat;

        // let stand_pat = if maximizing {
        let stand_pat = if self.side == Black {
            g.evaluate(&ts).sum()
        } else {
            -g.evaluate(&ts).sum()
        };

        if stand_pat >= beta {
            // debug!("quiescence beta cutoff: {}", k);
            // return score; // fail soft
            trace!("Quiescence returning beta: {}", beta);
            // eprintln!("Quiescence returning beta 0: {}", beta);
            return beta; // fail hard
        }
        // return stand_pat;

        // /// Delta prune
        // let mut big_delta = Queen.score();
        // if m0.filter_promotion() {
        //     big_delta += Queen.score() - Pawn.score();
        // }
        // if !maximizing {
        //     if stand_pat >= (beta + big_delta) {
        //         return beta;
        //     }
        // }
        // // return stand_pat; // correct

        if alpha < stand_pat {
            // eprintln!("Quiescence new alpha 0: {}", alpha);
            alpha = stand_pat;
        }
        // return stand_pat;

        let mut captures = ms.into_iter().filter(|m| m.filter_all_captures()).collect::<Vec<_>>();

        /// MVV LVA move ordering
        order_mvv_lva(&mut captures);

        for mv in captures.into_iter() {
            if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
                match g2.search_all(&ts) {
                    Outcome::Moves(ms2) => {

                        let score = -self.quiescence3(
                            &ts, &g, ms2, k + 1, -alpha, -beta, !maximizing, &mut stats);
                            // &ts, &g, ms2, k + 1, -alpha, -beta, &mut stats);

                        if score >= beta {
                            trace!("Quiescence returning beta 1: {}", beta);
                            return beta;
                        }
                        if score > alpha {
                            // trace!("Quiescence new alpha 1: {}", alpha);
                            alpha = score;
                        }
                    },
                    Outcome::Checkmate(_) => {
                        // panic!("checkmate in quiescent");
                    },
                    Outcome::Stalemate => {
                        // panic!("stalemate in quiescent");
                    },
                }
            }
        }


        trace!("quiescence return alpha: {}", alpha);
        // eprintln!("quiescence return alpha: {}", alpha);
        alpha
    }

}

