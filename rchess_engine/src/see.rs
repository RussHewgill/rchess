
use crate::alphabeta::*;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::explore::*;


impl Game {

    pub fn static_exchange_ge(&self, ts: &Tables, mv: Move, threshold: Score) -> bool {
        self._static_exchange_ge(ts, mv, threshold).unwrap_or(false)
    }

    fn _static_exchange_ge(&self, ts: &Tables, mv: Move, threshold: Score) -> Option<bool> {

        let side = self.state.side_to_move;
        let (from,to) = (mv.sq_from(),mv.sq_to());

        // if mv.filter_castle() | mv.filter_promotion() | mv.filter_en_passant() {
        if !mv.filter_all_captures() || mv.filter_en_passant() {
            return Some(0 >= threshold);
        }

        let mut swap: Score = self.get_at(to)?.1.score() - threshold;
        // eprintln!("swap 0 = {:?}", swap);
        if swap < 0 {
            return Some(false);
        }

        swap = self.get_at(from)?.1.score() - swap;
        // eprintln!("swap 1 = {:?}", swap);
        if swap <= 0 {
            return Some(true);
        }

        let mut occ = self.all_occupied() ^ BitBoard::single(from) ^ BitBoard::single(to);

        let mut attackers_own = self.find_attackers_to(&ts, to, !side);
        // attackers_own &= !(BitBoard::single(mv.sq_from()));
        let mut attackers_other = self.find_attackers_to(&ts, to, side);

        let mut attackers = attackers_own | attackers_other;
        let mut stm_attackers: BitBoard;
        let mut bb: BitBoard;

        let mut res = 1;
        let mut stm = side;

        let sides = [self.get_color(White),self.get_color(Black)];

        loop {
            stm = !stm;
            attackers &= occ;

            stm_attackers = attackers & sides[stm];
            if stm_attackers.is_empty() { break; }

            if (self.get_pinners(!stm) & occ).is_not_empty() {
                stm_attackers &= !self.get_pins(stm);
            }

            if stm_attackers.is_empty() { break; }

            res ^= 1;

            bb = stm_attackers & self.get_piece(Pawn);
            if bb.is_not_empty() {
                swap = Pawn.score() - swap;
                if swap < res { break; }
                occ ^= SQUARE_BB[bb.bitscan()];
                attackers |= ts.attacks_rook(to, occ) & (self.get_piece(Rook) | self.get_piece(Queen));
                continue;
            }

            bb = stm_attackers & self.get_piece(Knight);
            if bb.is_not_empty() {
                swap = Knight.score() - swap;
                if swap < res { break; }
                occ ^= SQUARE_BB[bb.bitscan()];
                continue;
            }

            bb = stm_attackers & self.get_piece(Bishop);
            if bb.is_not_empty() {
                swap = Bishop.score() - swap;
                if swap < res { break; }
                occ ^= SQUARE_BB[bb.bitscan()];
                attackers |= ts.attacks_bishop(to, occ) & (self.get_piece(Bishop) | self.get_piece(Queen));
                continue;
            }

            bb = stm_attackers & self.get_piece(Rook);
            if bb.is_not_empty() {
                swap = Rook.score() - swap;
                if swap < res { break; }
                occ ^= SQUARE_BB[bb.bitscan()];
                attackers |= ts.attacks_rook(to, occ) & (self.get_piece(Rook) | self.get_piece(Queen));
                continue;
            }

            bb = stm_attackers & self.get_piece(Queen);
            if bb.is_not_empty() {
                swap = Rook.score() - swap;
                if swap < res { break; }
                occ ^= SQUARE_BB[bb.bitscan()];
                attackers |= (ts.attacks_rook(to, occ) & (self.get_piece(Rook) | self.get_piece(Queen)))
                    | ts.attacks_bishop(to, occ) & (self.get_piece(Bishop) | self.get_piece(Queen));
                continue;
            }

            // king
            if (attackers & !sides[stm]).is_not_empty() {
                res ^= 1;
            }
            return Some(res == 1);

        }

        return Some(res == 1);
    }
}

impl Game {

    pub fn static_exchange(&self, ts: &Tables, mv: Move) -> Option<Score> {

        if !mv.filter_all_captures() {
            return None;
        }

        // let mut gain  = [0i32; 32];
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

        // eprintln!("attackers_own   = {:?}", attackers_own);
        // eprintln!("attackers_other = {:?}", attackers_other);

        let mut pc = mv.piece().unwrap();
        let mut last_cap = mv.victim().unwrap();
        let mut score = 0;

        loop {
            depth += 1;

            let score0 = if side0 == side {
                last_cap.score()
            } else {
                -last_cap.score()
            };

            if depth >= 32 {
                panic!("see depth too great? {:?}, {:?},{:?}\n{:?}\n{}",
                       mv, mv.piece(), mv.victim(), self, self.to_fen());
            }
            // gain[depth] = score0;
            score += score0;
            // eprintln!("score = {:?}", score);

            // eprintln!("depth, side, pc = {:?}, {:?}, {:?}", depth, side, pc);
            // eprintln!("gain = {:?}", &gain[..8]);

            // // XXX: OOB
            // if depth > 0 && i32::max(-gain[depth-1], gain[depth]) < 0 {
            //     break;
            // }

            attadef ^= from_set;
            occ ^= from_set;

            side = !side;
            if (from_set & may_xray).is_not_empty() {
                let b0 = self.consider_xrays(ts, mv.sq_to(), side, occ);
                attadef |= b0;
                // eprintln!("attadef = {:?}", attadef);
            }

            // trace!("side, depth = {:?}, {:>3?}", side, depth);
            // trace!("attadef = {:?}", attadef);

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

        Some(score)
    }

    fn consider_xrays(&self, ts: &Tables, c0: Coord, side: Color, occ: BitBoard) -> BitBoard {
        let moves_r = ts.attacks_rook(c0, occ) & occ;
        let moves_b = ts.attacks_bishop(c0, occ) & occ;

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

}


