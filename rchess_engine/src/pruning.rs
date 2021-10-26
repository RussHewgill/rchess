
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::explore::*;


/// Null Move
impl Explorer {

    pub fn prune_null_move(&self,
                           ts: &Tables
    ) -> bool {



        unimplemented!()
    }


}

/// Static Exchange
impl Explorer {

    fn get_smallest_attacker(ts: &Tables, g: &Game, c0: Coord, side: Color) -> Option<(Move, Piece)> {
        let attackers = g.find_attackers_to(ts, c0, side);
        if attackers.is_empty() { return None; }
        let pawns = attackers & g.get(Pawn, side);
        if pawns.is_not_empty() {
            unimplemented!()
        }
        unimplemented!()
    }

    pub fn static_exchange(ts: &Tables, g: &Game, c0: Coord, side: Color) -> Score {
        let mut value = 0;

        if let Some((mv, pc)) = Self::get_smallest_attacker(ts, g, c0, side) {
            if let Ok(g2) = g.make_move_unchecked(ts, mv) {
                value = i32::max(0, pc.score() - Self::static_exchange(ts, &g2, c0, !side));
            }
        }
        value
    }

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


