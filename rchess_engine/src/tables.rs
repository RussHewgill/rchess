
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

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct MoveSetBishop {
    pub ne: BitBoard,
    pub nw: BitBoard,
    pub se: BitBoard,
    pub sw: BitBoard,
}

impl MoveSetBishop {
    pub fn to_vec(&self) -> Vec<(D,BitBoard)> {
        vec![(NE,self.ne),(NW,self.nw),(SE,self.se),(SW,self.sw)]
    }
    pub fn empty() -> Self {
        Self {
            ne: BitBoard::empty(),
            nw: BitBoard::empty(),
            se: BitBoard::empty(),
            sw: BitBoard::empty(),
        }
    }
    pub fn get_dir(&self, d: D) -> &BitBoard {
        match d {
            NE => &self.ne,
            NW => &self.nw,
            SE => &self.se,
            SW => &self.sw,
            _ => panic!("MoveSetBishop::get Rank or File Bishop?")
        }
    }
}

pub struct Tables {
    // pub knight_moves: HashMap<Coord, BitBoard>,
    // pub rook_moves:   HashMap<Coord, MoveSetRook>,
    knight_moves: [[BitBoard; 8]; 8],
    rook_moves:   [[MoveSetRook; 8]; 8],
    bishop_moves: [[MoveSetBishop; 8]; 8],
    // endgames: 
}

impl Tables {
    pub fn get_rook(&self, Coord(x,y): Coord) -> &MoveSetRook {
        &self.rook_moves[x as usize][y as usize]
    }
    pub fn get_bishop(&self, Coord(x,y): Coord) -> &MoveSetBishop {
        &self.bishop_moves[x as usize][y as usize]
    }
    pub fn get_knight(&self, Coord(x,y): Coord) -> &BitBoard {
        &self.knight_moves[x as usize][y as usize]
    }
}

impl Tables {

    pub fn new() -> Self {
        Self {
            knight_moves: Self::gen_knights(),
            rook_moves:   Self::gen_rooks(),
            bishop_moves: Self::gen_bishops(),
        }
    }

}

impl Tables {

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

    fn gen_bishops() -> [[MoveSetBishop; 8]; 8] {
        let m0 = MoveSetBishop::empty();
        let mut out = [[m0; 8]; 8];
        for y in 0..8 {
            for x in 0..8 {
                out[x as usize][y as usize] = Self::gen_bishop_move(Coord(x,y));
            }
        }
        out
    }

    fn gen_bishop_move(c: Coord) -> MoveSetBishop {
        let sq: u32 = c.into();

        let ne = Self::gen_diagonal(c, true);
        let sw = Self::gen_diagonal(c, false);

        let se = Self::gen_antidiagonal(c, true);
        let nw = Self::gen_antidiagonal(c, false);

        MoveSetBishop {ne, nw, se, sw}
    }

    pub fn gen_diagonal(c0: Coord, positive: bool) -> BitBoard {
        let mut out = BitBoard::single(c0);
        let mut c = c0;
        let d = if positive { NE } else { SW };
        while let Some(k) = d.shift_coord(c) {
            c = k;
            out.flip_mut(c)
        }
        out &= !BitBoard::single(c0);
        out
    }

    pub fn gen_antidiagonal(c0: Coord, positive: bool) -> BitBoard {
        let mut out = BitBoard::single(c0);
        let mut c = c0;
        let d = if positive { SE } else { NW };
        while let Some(k) = d.shift_coord(c) {
            c = k;
            out.flip_mut(c)
        }
        out &= !BitBoard::single(c0);
        out
    }

    // fn gen_antidiagonal(c: Coord, positive: bool) -> BitBoard {
    //     // let mut out = BitBoard::single(c);
    //     let mut out = BitBoard::empty();
    //     if positive {
    //         // out |= out.shift(SE);
    //         for k in c.0..8 {
    //             out.flip_mut(Coord(k+1,c.1));
    //             eprintln!("{:?}\n", out);
    //         }
    //     } else {
    //     }
    //     out
    // }

    // fn gen_diagonal(c: Coord) -> BitBoard {
    //     let v: Vec<Coord> = (0..8).map(|x| Coord(x,x)).collect();
    //     let b0 = BitBoard::new(&v);
    //     if c.0 == c.1 {
    //         b0
    //     } else if c.0 > c.1 {
    //         b0.shift_mult(E, c.0.into())
    //     } else {
    //         b0.shift_mult(N, c.1.into())
    //     }
    // }

    // fn gen_antidiagonal(c: Coord) -> BitBoard {
    //     let v: Vec<Coord> = (0..8).map(|x| Coord(x,7-x)).collect();
    //     let b0 = BitBoard::new(&v);
    //     if (7 - c.0) == c.1 {
    //         b0
    //     } else if (7 - c.0) > c.1 {
    //         b0.shift_mult(W, (7 - c.0).into())
    //     } else {
    //         b0.shift_mult(N, c.1.into())
    //     }
    // }

    fn index_diagonal(Coord(x,y): Coord) -> u8 {
        y.overflowing_sub(x).0 & 15
    }

    fn index_antidiagonal(Coord(x,y): Coord) -> u8 {
        (y + x) ^ 7
    }

    // pub fn gen_bishop_block_mask(c: Coord) -> BitBoard {
    //     unimplemented!()
    // }

    // pub fn gen_bishop_block_board(c: Coord) -> BitBoard {
    //     unimplemented!()
    // }


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

impl Tables {

    fn gen_pawns() -> [[(BitBoard,BitBoard); 8]; 8] {
        const X: usize = 8;

        // let mut out = [[(BitBoard::empty()); 8]; 8];
        let mut out: [[(BitBoard,BitBoard); X]; X];

        for y in 0..X {
            for x in 0..X {
                out[x as usize][y as usize] = Self::gen_pawn_move(Coord(x as u8,y as u8));
            }
        }
        out
    }

    fn gen_pawn_move(c0: Coord) -> (BitBoard,BitBoard) {
        let mut bw = BitBoard::empty();
        if let Some(w0) = NE.shift_coord(c0) { bw.set_one(w0); }
        if let Some(w1) = NW.shift_coord(c0) { bw.set_one(w1); }

        let mut bb = BitBoard::empty();
        if let Some(b0) = SE.shift_coord(c0) { bb.set_one(b0); }
        if let Some(b1) = SW.shift_coord(c0) { bb.set_one(b1); }

        (bw,bb)
    }

}

