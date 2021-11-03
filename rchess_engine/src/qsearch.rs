
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
    pub fn qsearch(
        &self,
        ts:             &Tables,
        g:              &Game,
        (ply,qply):     (i16,i16),
        mut alpha:      i32,
        mut beta:       i32,
        mut stats:      &mut SearchStats,
    ) -> Score {
        trace!("qsearch, ply {}, a/b: {:?},{:?}", ply, alpha, beta);

        let stand_pat = g.evaluate(&ts).sum();
        // let stand_pat = if self.side == Black { stand_pat } else { -stand_pat };
        let stand_pat = if g.state.side_to_move == Black { -stand_pat } else { stand_pat };

        let mut allow_stand_pat = true;

        stats.qt_nodes += 1;
        stats!(stats.q_max_depth = stats.q_max_depth.max(ply as u8));

        if stand_pat >= beta && allow_stand_pat {
            trace!("qsearch returning beta 0: {:?}, sp = {}", beta, stand_pat);
            return beta;
        }

        if stand_pat > alpha {
            alpha = stand_pat;
            trace!("qsearch new alpha: {:?}", alpha);
        }

        let mut moves = if qply > 6 && g.state.last_capture.is_some() {
            let cap = g.state.last_capture.unwrap();

            match g.search_all_single(&ts, cap, None) {
                Outcome::Moves(ms) => ms,
                _                  => vec![],
            }
        } else if qply > 6 {
            return stand_pat;
        } else {
                match g.search_only_captures(&ts) {
                Outcome::Moves(ms) => ms,
                _                  => {
                    trace!("qsearch no legal capture moves");
                    if !g.in_check() {
                        vec![]
                    } else {
                        match g.search_all(&ts) {
                            Outcome::Moves(ms) => {
                                allow_stand_pat = false;
                                ms
                            },
                            Outcome::Checkmate(c) => {
                                trace!("qsearch checkmate");
                                let score = 100_000_000 - ply as Score;
                                return -score;
                            },
                            Outcome::Stalemate => {
                                trace!("qsearch stalemate");
                                let score = -200_000_000 + ply as Score;
                                return -score;
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
            trace!("qsearch: delta prune: {}", alpha);
            return alpha;
        }

        order_mvv_lva(&mut moves);

        // let ms = moves.into_iter()
        //     .flat_map(|m| g.make_move_unchecked(&ts, m).ok().map(|x| (m,x)));

        // for (mv,g2) in ms {
        for mv in moves.into_iter() {

            if let Ok(g2) = g.make_move_unchecked(&ts, mv) {

                // trace!("qsearch: mv = {:?}, g = {:?}\n, g2 = {:?}", mv, g, g2);
                trace!("qsearch: mv = {:?}", mv);

                if let Some(see) = g.static_exchange(&ts, mv) {
                    if see < 0 {
                        // trace!("fen = {}", g.to_fen());
                        // trace!("qsearch: SEE negative: {} {:?}", see, g);
                        trace!("qsearch: SEE negative: {}", see);
                        continue;
                    }
                }

                let score = -self.qsearch(&ts, &g2, (ply + 1,qply + 1), -beta, -alpha, &mut stats);

                if score >= beta && allow_stand_pat {
                    trace!("qsearch returning beta 1: {:?}", beta);
                    return beta;
                }

                if score > alpha {
                    alpha = score;
                }

            }
        }

        trace!("qsearch returning alpha 0: {:?}", alpha);
        alpha
    }

}

