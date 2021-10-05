
use crate::types::*;

use std::collections::HashMap;


pub struct Tables {
    knight_moves: HashMap<Coord, BitBoard>,
    // endgames: 
}

impl Tables {

    pub fn new() -> Self {
        Self {
            knight_moves: Self::gen_knights(),
        }
    }

    pub fn gen_knights() -> HashMap<Coord, BitBoard> {
        (0..9).into_iter()
            .zip(0..9)
            .map(|(x,y)| (Coord(x,y), Self::gen_knight_move(Coord(x,y))))
            .collect()
    }

    fn gen_knight_move(c: Coord) -> BitBoard {
        let b = BitBoard::new(&vec![c]);

        let l1 = b.0.overflowing_shr(1).0 & !BitBoard::mask_file(7).0;
        let l2 = b.0.overflowing_shr(2).0 & !(BitBoard::mask_file(7).0 | BitBoard::mask_file(6).0);

        let r1 = b.0.overflowing_shl(1).0 & !BitBoard::mask_file(0).0;
        let r2 = b.0.overflowing_shl(2).0 & !(BitBoard::mask_file(0).0 | BitBoard::mask_file(1).0);

        let h1 = l1 | r1;
        let h2 = l2 | r2;

        BitBoard(h1.overflowing_shl(16).0
                 | h1.overflowing_shr(16).0
                 | h2.overflowing_shl(8).0
                 | h2.overflowing_shr(8).0
        )
    }

}

