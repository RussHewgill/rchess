
use std::collections::VecDeque;

use crate::alphabeta::*;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::explore::*;


/// Null Move
impl Explorer {

    pub fn prune_null_move_negamax(
        &self,
        ts:                 &Tables,
        mut g:              &Game,
        mut cfg:            ABConfig,
        depth:              Depth,
        ply:                Depth,
        alpha:              i32,
        beta:               i32,
        mut stats:          &mut SearchStats,
        prev_mvs:           VecDeque<(Zobrist,Move)>,
        mut history:        &mut [[[Score; 64]; 64]; 2],
        tt_r:               &TTRead,
        tt_w:               TTWrite,
    ) -> bool {

        cfg.root    = false;
        cfg.do_null = false;

        let mv = Move::NullMove;

        let r = 2;

        // if depth <= (1 + r) { return false; }
        if depth < (1 + r) { return false; }
        let mut stop_counter = 0;

        if let Ok(g2) = g.make_move_unchecked(ts, mv) {
            let mut pms = prev_mvs.clone();
            pms.push_back((g.zobrist,mv));

            if let ABResults::ABSingle(mut res) = self._ab_search_negamax(
                &ts, &g2, cfg,
                depth - 1 - r, ply + 1, &mut stop_counter,
                (-beta, -beta + 1),
                // -beta, -alpha,
                &mut stats, pms, &mut history,
                tt_r, tt_w) {

                res.moves.push_front(mv);
                res.neg_score();

                if res.score >= beta { // Beta cutoff
                    // trace!("null move beta cutoff, a/b: {}, {}, score = {}\n{:?}",
                    //        -beta, -beta + 1, res.score, g2);
                    stats!(stats.null_prunes += 1);
                    return true;
                }
            }
        }
        false
    }

    pub fn prune_null_move(
        &self,
        ts:                 &Tables,
        mut g:              &Game,
        max_depth:          Depth,
        depth:              Depth,
        k:                  i16,
        alpha:              i32,
        beta:               i32,
        maximizing:         bool,
        mut stats:          &mut SearchStats,
        // prev_mvs:           Vec<Move>,
        prev_mvs:           VecDeque<(Zobrist,Move)>,
        mut history:        &mut [[[Score; 64]; 64]; 2],
        tt_r:               &TTRead,
        tt_w:               TTWrite,
    ) -> bool {

        let mv = Move::NullMove;

        let r = 2;

        // if depth <= (1 + r) { return false; }
        if depth < (1 + r) { return false; }

        if let Ok(g2) = g.make_move_unchecked(ts, mv) {
            let mut pms = prev_mvs.clone();
            pms.push_back((g.zobrist,mv));

            if let Some(((_,score),_)) = self._ab_search(
                &ts, &g2, max_depth,
                depth - 1 - r, k + 1,
                alpha, beta, !maximizing, &mut stats, pms,
                &mut history,
                tt_r, tt_w) {

                if maximizing {
                    if score >= beta { // Beta cutoff
                        stats!(stats.null_prunes += 1);
                        return true;
                    }
                } else {
                    if score <= alpha {
                        stats!(stats.null_prunes += 1);
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Cycle prevention
impl Explorer {


    pub fn cycle_prevention(
        &self,
        ts:           &Tables,
        (mv,g2):      (&Move, &Game),
        prev_mvs:     &VecDeque<(Zobrist, Move)>,
    ) -> bool {

        for (zb,mv) in prev_mvs.iter().rev() {
        }

        // if Some(&(g2.zobrist,*mv)) == prev_mvs.get(prev_mvs.len() - 3) {
        //     panic!("wat: {:?}\n {:?}", mv, g2)
        // }

        // unimplemented!()
        false
    }

}


