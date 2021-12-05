
use std::collections::VecDeque;

use crate::alphabeta::*;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::explore::*;

/// Null Move
impl ExHelper {

    pub fn prune_null_move_negamax(
        &self,
        ts:                 &Tables,
        mut g:              &Game,
        mut cfg:            ABConfig,
        depth:              Depth,
        ply:                Depth,
        // (alpha,beta):       (i32,i32),
        (alpha,beta):       (Score,Score),
        mut stats:          &mut SearchStats,
        mut tracking:       &mut ExTracking,
    ) -> bool {

        cfg.root        = false;
        cfg.do_null     = false;
        cfg.inside_null = true;

        let mv = Move::NullMove;

        let r = 2;

        // if depth <= (1 + r) { return false; }
        if depth < (1 + r) { return false; }
        let mut stop_counter = 0;

        if let Ok(g2) = g.make_move_unchecked(ts, mv) {
            // let mut pms = prev_mvs.clone();
            // pms.push_back((g.zobrist,mv));

            if let ABResults::ABSingle(mut res) = self._ab_search_negamax(
                &ts, &g2, cfg,
                depth - 1 - r, ply + 1, &mut stop_counter,
                (-beta, -beta + 1),
                // (-beta, -alpha), // XXX: doesn't work
                &mut stats, &mut tracking,
            ) {

                // res.moves.push_front(mv);
                res.neg_score(mv);

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


