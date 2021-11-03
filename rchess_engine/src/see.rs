
use crate::alphabeta::*;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::explore::*;


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

            if depth >= 32 {
                panic!("see depth too great? {:?}\n{:?}\n{}", mv, self, self.to_fen());
            }
            // gain[depth] = score0;
            score += score0;

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

        Some(score)
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

}


