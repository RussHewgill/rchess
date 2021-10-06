
use crate::types::*;

use std::collections::HashMap;

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct MoveSetRook {
    pub n: BitBoard,
    pub e: BitBoard,
    pub w: BitBoard,
    pub s: BitBoard,
}

impl MoveSetRook {
    pub fn to_vec(&self) -> Vec<(D,BitBoard)> {
        vec![(N,self.n),(E,self.e),(W,self.w),(S,self.s)]
    }
    pub fn empty() -> Self {
        Self {
            n: BitBoard::empty(),
            e: BitBoard::empty(),
            w: BitBoard::empty(),
            s: BitBoard::empty(),
        }
    }

    pub fn get_dir(&self, d: D) -> &BitBoard {
        match d {
            N => &self.n,
            E => &self.e,
            W => &self.w,
            S => &self.s,
            _ => panic!("MoveSetRook::get Diagonal rook?")
        }
    }
}

pub struct Tables {
    // pub knight_moves: HashMap<Coord, BitBoard>,
    // pub rook_moves:   HashMap<Coord, MoveSetRook>,
    knight_moves: [[BitBoard; 8]; 8],
    rook_moves:   [[MoveSetRook; 8]; 8],
    // endgames: 
}

impl Tables {
    pub fn get_rook(&self, Coord(x,y): Coord) -> &MoveSetRook {
        &self.rook_moves[x as usize][y as usize]
    }
}

impl Tables {

    pub fn new() -> Self {
        Self {
            knight_moves: Self::gen_knights(),
            rook_moves:   Self::gen_rooks(),
        }
    }

    // fn gen_rooks() -> HashMap<Coord, MoveSetRook> {
    fn gen_rooks() -> [[MoveSetRook; 8]; 8] {
        let m0 = MoveSetRook::empty();
        let mut out = [[m0; 8]; 8];

        for y in 0..8 {
            for x in 0..8 {
                out[x as usize][y as usize] = Self::gen_rook_move(Coord(x,y));
            }
        }

        out
    }

    fn gen_rook_move(c: Coord) -> MoveSetRook {

        let sq = BitBoard::index_square(c) as u32;

        let n = Self::rook_n(sq);
        let e = Self::rook_e(sq);
        let s = Self::rook_s(sq);
        let w = Self::rook_w(sq);

        // n | e | s | w
        MoveSetRook { n,e,w,s }
    }

    fn rook_n(sq: u32) -> BitBoard {
        let n0 = 0x0101010101010100u64;
        BitBoard(n0.overflowing_shl(sq).0)
            // & !(BitBoard::mask_file(7))
    }

    fn rook_e(sq: u32) -> BitBoard {
        BitBoard(2 * ( (1u64.overflowing_shl(sq | 7).0) - (1u64.overflowing_shl(sq).0)))
            // & !(BitBoard::mask_rank(0))
    }

    fn rook_s(sq: u32) -> BitBoard {
        let n0 = 0x0080808080808080u64;
        BitBoard(n0.overflowing_shr(sq ^ 63).0)
            // & !(BitBoard::mask_file(7))
    }

    fn rook_w(sq: u32) -> BitBoard {
        BitBoard(1u64.overflowing_shl(sq).0 - 1u64.overflowing_shl(sq & 56).0)
            // & !(BitBoard::mask_rank(0))
    }

}

impl Tables {
    // fn gen_bishops() -> 
}

impl Tables {

    // fn gen_knights() -> HashMap<Coord, BitBoard> {
    fn gen_knights() -> [[BitBoard; 8]; 8] {
        let mut out = [[BitBoard::empty(); 8]; 8];

        for y in 0..8 {
            for x in 0..8 {
                out[x as usize][y as usize] = Self::gen_knight_move(Coord(x,y));
            }
        }
        out

        // (0..8).into_iter()
        //     .zip(0..8)
        //     .for_each(|(x,y)| out[x as usize][y as usize] = Self::gen_knight_move(Coord(x,y)));
        // (0..9).into_iter()
        //     .zip(0..9)
        //     .map(|(x,y)| (Coord(x,y), Self::gen_knight_move(Coord(x,y))))
        //     .collect()
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

