
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
        max_depth:          Depth,
        depth:              Depth,
        ply:                i16,
        alpha:              i32,
        beta:               i32,
        mut stats:          &mut SearchStats,
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

            if let ABResults::ABSingle(mut res) = self._ab_search_negamax(
                &ts, &g2, max_depth,
                depth - 1 - r, ply + 1,
                -beta, -beta + 1,
                // -beta, -alpha,
                &mut stats, pms, &mut history,
                tt_r, tt_w, false, false) {

                res.moves.push_front(mv);
                res.neg_score();

                if res.score >= beta { // Beta cutoff
                    // trace!("null move beta cutoff, a/b: {}, {}, score = {}", -beta, -beta + 1, res.score);
                    stats.null_prunes += 1;
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
                        stats.null_prunes += 1;
                        return true;
                    }
                } else {
                    if score <= alpha {
                        stats.null_prunes += 1;
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

/// Static Exchange
impl Game {

    pub fn static_exchange(&self, ts: &Tables, mv: Move) -> Score {

        let mut gain  = [0i32; 32];
        let mut depth = 0;

        let may_xray =
            self.get_piece(Pawn) | self.get_piece(Bishop) | self.get_piece(Rook) | self.get_piece(Queen);

        let side0 = self.state.side_to_move;
        let mut side = side0;

        let mut occ = self.all_occupied();

        let mut from_set = {
            let f: u32 = mv.sq_from().into();
            BitBoard(1 << f)
        };

        let mut attackers_own = self.find_attackers_to(&ts, mv.sq_to(), !side);
        // attackers_own &= !(BitBoard::single(mv.sq_from()));
        let mut attackers_other = self.find_attackers_to(&ts, mv.sq_to(), side);

        let mut attadef = attackers_own | attackers_other;

        // gain[depth] = mv.victim().unwrap().score();
        let mut pc = mv.piece().unwrap();
        // let mut score = mv.victim().unwrap().score();
        let mut last_cap = mv.victim().unwrap();
        let mut score = 0;

        loop {
            depth += 1;

            let score0 = if side0 == side {
                last_cap.score()
            } else {
                -last_cap.score()
            };

            gain[depth] = score0;
            score += score0;

            // eprintln!("depth, side, pc = {:?}, {:?}, {:?}", depth, side, pc);
            // eprintln!("gain = {:?}", &gain[..8]);

            // // XXX: OOB
            // if depth > 0 && i32::max(-gain[depth-1], gain[depth]) < 0 { break; }

            attadef ^= from_set;
            occ ^= from_set;

            side = !side;
            if (from_set & may_xray).is_not_empty() {
                attadef |= self.consider_xrays(ts, mv.sq_to(), side, occ);
                // eprintln!("attadef = {:?}", attadef);
            }

            if let Some((pc2,fs)) = self.least_val_piece(attadef, side) {
                last_cap = pc;
                from_set = fs;
                pc = pc2;
            } else {
                // debug!("breaking, side, attadef, {:?}, \n{:?}", side, attadef);
                // debug!("breaking, side, attadef, {:?}", side);
                break;
            }

        }

        // eprintln!("gain = {:?}", gain);

        score
    }

    fn consider_xrays(&self, ts: &Tables, c0: Coord, side: Color, occ: BitBoard) -> BitBoard {
        let moves_r = ts.attacks_rook(c0, occ);
        let moves_b = ts.attacks_bishop(c0, occ);

        let qs = self.get(Queen, side);
        let mut attackers = moves_r & (self.get(Rook, side) | qs);
        attackers |= moves_b & (self.get(Bishop, side) | qs);

        attackers
    }

    fn least_val_piece(&self, attadef: BitBoard, side: Color) -> Option<(Piece,BitBoard)> {
        for pc in Piece::iter_pieces() {
            let subset = attadef & self.get(pc, side);
            if subset.is_not_empty() {
                return Some((pc,subset));
            }
        }
        None
    }

    // fn get_smallest_attacker(ts: &Tables, g: &Game, c0: Coord, side: Color) -> Option<(Move, Piece)> {
    //     let attackers = g.find_attackers_to(ts, c0, side);
    //     if attackers.is_empty() { return None; }
    //     let pawns = attackers & g.get(Pawn, side);
    //     if pawns.is_not_empty() {
    //         unimplemented!()
    //     }
    //     unimplemented!()
    // }

    // pub fn static_exchange(ts: &Tables, g: &Game, c0: Coord, side: Color) -> Score {
    //     let mut value = 0;
    //     if let Some((mv, pc)) = Self::get_smallest_attacker(ts, g, c0, side) {
    //         if let Ok(g2) = g.make_move_unchecked(ts, mv) {
    //             value = i32::max(0, pc.score() - Self::static_exchange(ts, &g2, c0, !side));
    //         }
    //     }
    //     value
    // }

    // pub fn static_exchange(ts: &Tables, g: &Game, c0: Coord) -> Option<Score> {

    //     let mut gain: [Score; 32] = [0; 32];
    //     let mut d = 0;

    //     let mayxray = g.get_piece(Pawn) | g.get_piece(Bishop) | g.get_piece(Rook) | g.get_piece(Queen);

    //     let fromset = BitBoard::empty();
    //     let occ = g.all_occupied();

    //     let attacks_to = g.find_attackers_to(&ts, c0, White) | g.find_attackers_to(&ts, c0, Black);

    //     gain[d] = if let Some((_,pc)) = g.get_at(c0) {
    //         pc.score()
    //     } else { 0 };

    //     loop {
    //         d += 1;

    //         if let Some(attacker) = Self::get_least_valuable_piece(ts, g, c0) {
    //             gain[d] = attacker.1.score();
    //             if i32::max(-gain[d-1], gain[d]) < 0 { break; } // prune

    //         }
    //         if fromset.is_empty() { break; }
    //     }

    //     unimplemented!()
    // }

    // pub fn static_exchange2(&self, ts: &Tables, g: &Game, c0: Coord) -> Option<Score> {
    //     let mut val = 0;

    //     let attackers_own   = g.find_attackers_to(&ts, c0, !g.state.side_to_move);
    //     if attackers_own.is_empty() { return None; }

    //     let attackers_other = g.find_attackers_to(&ts, c0, g.state.side_to_move);

    //     // let attackers = attackers_own | attackers_other;

    //     // let mut attackers_own = attackers_own.into_iter()
    //     //     .flat_map(|sq| {
    //     //         let c1: Coord = sq.into();
    //     //         if let Some((col,pc)) = g.get_at(c1) {
    //     //             Some((c1,pc))
    //     //         } else { None }
    //     //     }).collect::<Vec<_>>();
    //     // attackers_own.sort_by(|a,b| a.1.score().cmp(&b.1.score()));

    //     // for (c1,pc) in attackers_own.iter() {
    //     //     eprintln!("(c1,pc) = {:?}", (c1,pc));
    //     // }


    //     unimplemented!()
    // }

}


