
use std::io::Write;

use crate::types::*;

pub use crate::tuning::*;

pub use self::movesets::*;
pub use self::magics::*;
pub use self::opening_book::*;
pub use self::endgames::*;
pub use self::eval::*;

use rand::Rng;
use lazy_static::lazy_static;
use itertools::Itertools;
// use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Serialize,Deserialize};
use serde_big_array::BigArray;

lazy_static! {
    pub static ref SQUAREDIST: [[u8; 64]; 64] = {
        let mut out = [[0; 64]; 64];
        for s1 in 0u32..8 {
            for s2 in 0u32..8 {
                let (c1,c2): (Coord,Coord) = (s1.into(),s1.into());

                let dist = {
                    let r = c1.rank_dist(c2);
                    let f = c1.file_dist(c2);
                    r.max(f)
                };

                // out[s1 as usize][s2 as usize] = c1.square_dist(c2);
                out[s1 as usize][s2 as usize] = dist;
            }
        }
        out
    };

}

pub static CENTERDIST: [u8; 64] = [
    3, 3, 3, 3, 3, 3, 3, 3,
    3, 2, 2, 2, 2, 2, 2, 3,
    3, 2, 1, 1, 1, 1, 2, 3,
    3, 2, 1, 0, 0, 1, 2, 3,
    3, 2, 1, 0, 0, 1, 2, 3,
    3, 2, 1, 1, 1, 1, 2, 3,
    3, 2, 2, 2, 2, 2, 2, 3,
    3, 3, 3, 3, 3, 3, 3, 3
];

pub static MASK_FILES: [BitBoard; 8] = [
    BitBoard(0x0101010101010101),
    BitBoard(0x0202020202020202),
    BitBoard(0x0404040404040404),
    BitBoard(0x0808080808080808),
    BitBoard(0x1010101010101010),
    BitBoard(0x2020202020202020),
    BitBoard(0x4040404040404040),
    BitBoard(0x8080808080808080),
];

pub static MASK_RANKS: [BitBoard; 8] = [
    BitBoard(0x00000000000000ff),
    BitBoard(0x000000000000ff00),
    BitBoard(0x0000000000ff0000),
    BitBoard(0x00000000ff000000),
    BitBoard(0x000000ff00000000),
    BitBoard(0x0000ff0000000000),
    BitBoard(0x00ff000000000000),
    BitBoard(0xff00000000000000),
];

fn def_line_bb()    -> [[BitBoard; 64]; 64] {
    let bishops = Tables::gen_bishops();
    Tables::gen_linebb(bishops)
}
fn def_between_bb()    -> [[BitBoard; 64]; 64] {
    let bishops = Tables::gen_bishops();
    Tables::gen_betweenbb(bishops)
}

// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Serialize,Deserialize,Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Tables {
    #[serde(skip,default = "Tables::gen_pawns")]
    // #[serde(serialize_with = "<[_]>::serialize")]
    pub pawn_moves:    [[MoveSetPawn; 8]; 8],
    #[serde(skip,default = "Tables::gen_rooks")]
    pub rook_moves:    [[MoveSetRook; 8]; 8],
    // #[serde(skip)]
    #[serde(with = "BigArray")]
    pub knight_moves:  [BitBoard; 64],
    #[serde(skip,default = "Tables::gen_bishops")]
    pub bishop_moves:  [[MoveSetBishop; 8]; 8],
    #[serde(skip,default = "Tables::gen_kings")]
    pub king_moves:    [[BitBoard; 8]; 8],
    #[serde(skip,default = "def_line_bb")]
    pub line_bb:       [[BitBoard; 64]; 64],
    #[serde(skip,default = "def_between_bb")]
    pub between_bb:    [[BitBoard; 64]; 64],
    #[serde(with = "BigArray")]
    magics_rook:   [Magic; 64],
    #[serde(with = "BigArray")]
    table_rook:    [BitBoard; 0x19000],
    #[serde(with = "BigArray")]
    magics_bishop: [Magic; 64],
    #[serde(with = "BigArray")]
    table_bishop:  [BitBoard; 0x1480],
    #[serde(skip)]
    pub piece_tables:  PcTables,
    #[serde(skip)]
    pub zobrist_tables: ZbTable,
    // endgames: 
}

// impl serde::Serialize for Tables {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut s = serializer.serialize_struct("Table", 13)?;
//         s.serialize_field("pawn_moves",   &self.pawn_moves[..])?;
//         s.serialize_field("rook_moves",   &self.rook_moves[..])?;
//         s.serialize_field("knight_moves", &self.knight_moves[..])?;
//         s.serialize_field("bishop_moves", &self.bishop_moves[..])?;
//         s.serialize_field("king_moves",   &self.king_moves[..])?;
//         // s.serialize_field("line_bb",   &self.line_bb[..])?;
//         // line_bb:       [[BitBoard; 64]; 64],
//         // between_bb:    [[BitBoard; 64]; 64],
//         // magics_rook:   [Magic; 64],
//         // table_rook:    [BitBoard; 0x19000],
//         // magics_bishop: [Magic; 64],
//         // table_bishop:  [BitBoard; 0x1480],
//         // pub piece_tables:  PcTables,
//         // pub zobrist_tables: ZbTable,
//         s.end()
//     }
// }

/// Piece getters
impl Tables {
    // pub fn get_rook(&self, Coord(x,y): Coord) -> &MoveSetRook {
    pub fn get_rook<T: Into<Coord>>(&self, c: T) -> &MoveSetRook {
        let Coord(x,y) = c.into();
        if (x > 7) | (y > 7) {
            panic!("x,y = {}, {}", x, y);
        }
        &self.rook_moves[x as usize][y as usize]
    }
    // pub fn get_bishop(&self, Coord(x,y): Coord) -> &MoveSetBishop {
    pub fn get_bishop<T: Into<Coord>>(&self, c: T) -> &MoveSetBishop {
        let Coord(x,y) = c.into();
        &self.bishop_moves[x as usize][y as usize]
    }
    // pub fn get_knight(&self, Coord(x,y): Coord) -> &BitBoard {
    pub fn get_knight<T: Into<Coord>>(&self, c: T) -> &BitBoard {
        // let Coord(x,y) = c.into();
        // &self.knight_moves[x as usize][y as usize]
        &self.knight_moves[c.into()]
    }
    // pub fn get_pawn(&self, Coord(x,y): Coord) -> &MoveSetPawn {
    pub fn get_pawn<T: Into<Coord>>(&self, c: T) -> &MoveSetPawn {
        let Coord(x,y) = c.into();
        &self.pawn_moves[x as usize][y as usize]
    }
    // pub fn get_king(&self, Coord(x,y): Coord) -> &BitBoard {
    pub fn get_king<T: Into<Coord>>(&self, c: T) -> &BitBoard {
        let Coord(x,y) = c.into();
        &self.king_moves[x as usize][y as usize]
    }
}

/// init
impl Tables {

    pub fn new() -> Self {
        Self::_new(true)
    }

    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {

        let b: Vec<u8> = bincode::serialize(&self).unwrap();

        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .truncate(true)
            .read(true)
            .create(true)
            .write(true)
            .open(path)
            .unwrap();

        file.write_all(&b)
    }

    pub fn read_from_file(path: &str) -> std::io::Result<Self> {

        let b: Vec<u8> = std::fs::read(&path)?;

        let ts: Tables = bincode::deserialize(&b).unwrap();

        Ok(ts)
    }

    pub(crate) fn _new(magics: bool) -> Self {
        let rook_moves   = Self::gen_rooks();
        let bishop_moves = Self::gen_bishops();

        let (magics_rook, table_rook) = if magics {
            Tables::_gen_magics(false).unwrap_err()
        } else {
            ([Magic::new(0, BitBoard::empty(), BitBoard::empty(), 0); 64],
             [BitBoard::empty(); 0x19000])
        };

        let (magics_bishop, table_bishop) = if magics {
            Tables::_gen_magics(true).unwrap()
        } else {
            ([Magic::new(0, BitBoard::empty(), BitBoard::empty(), 0); 64],
            [BitBoard::empty(); 0x1480])
        };

        // let (piece_tables_midgame,piece_tables_endgame) = PcTables::new();
        let piece_tables = PcTables::new();

        Self {
            knight_moves: Self::gen_knights(),
            rook_moves,
            bishop_moves,
            pawn_moves:   Self::gen_pawns(),
            king_moves:   Self::gen_kings(),
            line_bb:      Self::gen_linebb(bishop_moves),
            between_bb:   Self::gen_betweenbb(bishop_moves),
            magics_rook,
            table_rook,
            magics_bishop,
            table_bishop,
            piece_tables,
            // piece_tables_midgame,
            // piece_tables_endgame,

            zobrist_tables: ZbTable::new(),
        }
    }

}

/// Lookup table getters
impl Tables {

    /// Excludes s1, Includes s2
    /// When not aligned, returns s2
    pub fn between<T: Into<Coord>>(&self, s1: T, s2: T) -> BitBoard {
        let (s1,s2): (Coord,Coord) = (s1.into(),s2.into());
        let (s1,s2): (u32,u32) = (s1.into(),s2.into());
        self.between_bb[s1 as usize][s2 as usize]
    }

    pub fn between_exclusive<T: Into<Coord>>(&self, s1: T, s2: T) -> BitBoard {
        let (s1,s2): (Coord,Coord) = (s1.into(),s2.into());
        let (s1,s2): (u32,u32) = (s1.into(),s2.into());
        self.between_bb[s1 as usize][s2 as usize].set_zero(s2.into())
    }

    pub fn line<T: Into<Coord>>(&self, s1: T, s2: T) -> BitBoard {
        let (s1,s2): (Coord,Coord) = (s1.into(),s2.into());
        let (s1,s2): (u32,u32) = (s1.into(),s2.into());
        self.line_bb[s1 as usize][s2 as usize]
    }

    pub fn aligned<T: Into<Coord>>(&self, s1: T, s2: T, s3: T) -> BitBoard {
        self.line(s1, s2) & BitBoard::single(s3.into())
    }

}

/// Lookup table Generators
impl Tables {

    fn gen_betweenbb(bishops: [[MoveSetBishop; 8]; 8]) -> [[BitBoard; 64]; 64] {
        let mut out = [[BitBoard::empty(); 64]; 64];
        for x in 0u32..64 {
            for y in 0u32..64 {
                let b = Self::mask_between(bishops, x.into(), y.into());
                let b = b | BitBoard::single(y.into());
                out[x as usize][y as usize] = b;
            }
        }
        out
    }

    fn mask_between(bishops: [[MoveSetBishop; 8]; 8], c0: Coord, c1: Coord) -> BitBoard {

        let Coord(x0,y0) = c0;
        let Coord(x1,y1) = c1;

        if x0 == x1 {
            // File
            let (x0,x1) = (x0.min(x1),x0.max(x1));
            let (y0,y1) = (y0.min(y1),y0.max(y1));
            let b0 = BitBoard::single(Coord(x0,y0));
            let b1 = BitBoard::single(Coord(x1,y1));
            let b = 2u64.overflowing_mul(b1.0).0;
            let b = b.overflowing_sub(b0.0).0;
            let b = BitBoard(b);
            // let b = BitBoard(2 * b1.0 - b0.0);
            let m = BitBoard::mask_file(x0.into());
            (b & m) & !(b0 | b1)
        } else if y0 == y1 {
            // Rank
            let (x0,x1) = (x0.min(x1),x0.max(x1));
            let (y0,y1) = (y0.min(y1),y0.max(y1));
            let b0 = BitBoard::single(Coord(x0,y0));
            let b1 = BitBoard::single(Coord(x1,y1));
            let b = 2u64.overflowing_mul(b1.0).0;
            let b = b.overflowing_sub(b0.0).0;
            let b = BitBoard(b);
            // let b = BitBoard(2 * b1.0 - b0.0);
            let m = BitBoard::mask_rank(y0.into());
            (b & m) & !(b0 | b1)
        // } else if (x1 - x0) == (y1 - y0) {
        } else if (x1 as i64 - x0 as i64).abs() == (y1 as i64 - y0 as i64).abs() {
            // Diagonal
            let b0 = BitBoard::single(Coord(x0,y0));
            let b1 = BitBoard::single(Coord(x1,y1));
            // let b = BitBoard::new(&[Coord(x0,y0),Coord(x1,y1)])

            let (bb0,bb1) = (b0.0.min(b1.0),b0.0.max(b1.0));

            // eprintln!("b0 = {:?}", b0);
            // eprintln!("b1 = {:?}", b1);

            // eprintln!("bb0 = {:?}", b0.bitscan());
            // eprintln!("bb1 = {:?}", b1.bitscan());

            let b = BitBoard(2u64.overflowing_mul(bb1).0.overflowing_sub(bb0).0);
            // let b = BitBoard(2 * b0.0 - b1.0);
            // eprintln!("b = {:?}", b);
            // let m = BitBoard::mask_rank(y0.into());
            let m = {
                let Coord(x,y) = c0.into();
                bishops[x as usize][y as usize]
            };

            let xx = x1 as i64 - x0 as i64;
            let yy = y1 as i64 - y0 as i64;

            let m = if xx.signum() == yy.signum() {
                m.ne | m.sw
            } else {
                m.nw | m.se
            };

            (b & m) & !(b0 | b1)
        } else {
            // println!("wat 2");
            // unimplemented!()
            BitBoard::empty()
        }
    }

    fn gen_linebb(bishops: [[MoveSetBishop; 8]; 8]) -> [[BitBoard; 64]; 64] {
        let mut out = [[BitBoard::empty(); 64]; 64];
        for x in 0u32..64 {
            for y in 0u32..64 {

                let Coord(x0,y0) = x.into();
                let Coord(x1,y1) = y.into();

                let f = BitBoard::mask_file(x0.into());
                let r = BitBoard::mask_rank(y0.into());

                let ds = bishops[x0 as usize][y0 as usize];
                let dp = (ds.ne | ds.sw).set_one(x.into());
                let dn = (ds.nw | ds.se).set_one(x.into());

                if (f & BitBoard::single(y.into())).0 != 0 {
                    out[x as usize][y as usize] = f;
                } else if (r & BitBoard::single(y.into())).0 != 0 {
                    out[x as usize][y as usize] = r;
                } else if (dp & BitBoard::single(y.into())).0 != 0 {
                    out[x as usize][y as usize] = dp;
                } else if (dn & BitBoard::single(y.into())).0 != 0 {
                    out[x as usize][y as usize] = dn;
                }

            }
        }
        out
    }

}

/// Rooks
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

/// Bishops
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

    pub fn gen_bishop_move(c: Coord) -> MoveSetBishop {
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
            out.flip_mut(c);
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

/// Knights
impl Tables {

    // fn gen_knights() -> HashMap<Coord, BitBoard> {
    // fn gen_knights() -> [[BitBoard; 8]; 8] {
    fn gen_knights() -> [BitBoard; 64] {
        let mut out = [BitBoard::empty(); 64];

        for y in 0..8 {
            for x in 0..8 {
                out[Coord(x,y)] = Self::gen_knight_move(Coord(x,y));
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

/// Kings
impl Tables {

    pub fn gen_kings() -> [[BitBoard; 8]; 8] {
        let mut out = [[BitBoard::empty(); 8]; 8];

        for y in 0..8 {
            for x in 0..8 {
                out[x as usize][y as usize] = Self::gen_king_move(Coord(x as u8,y as u8));
            }
        }
        out
    }

    fn gen_king_move(c0: Coord) -> BitBoard {
        let b0 = BitBoard::single(c0);
        let b1 = b0
            | b0.shift(W)
            | b0.shift(E);
        let b2 = b1
            | b1.shift(N)
            | b1.shift(S);

        b2 & !b0
    }

}

/// Pawns
impl Tables {

    fn gen_pawns() -> [[MoveSetPawn; 8]; 8] {
        let mut out = [[MoveSetPawn::empty(); 8]; 8];

        for y in 0..8 {
            for x in 0..8 {
                out[x as usize][y as usize] = Self::gen_pawn_move(Coord(x as u8,y as u8));
            }
        }
        out
    }

    fn gen_pawn_move(c0: Coord) -> MoveSetPawn {

        let mut wq = BitBoard::empty();
        if let Some(b) = N.shift_coord(c0) { wq = wq.set_one(b); }

        let mut bq = BitBoard::empty();
        if let Some(b) = S.shift_coord(c0) { bq = bq.set_one(b); }

        let mut wc = BitBoard::empty();
        if let Some(w0) = NE.shift_coord(c0) { wc = wc.set_one(w0); }
        if let Some(w1) = NW.shift_coord(c0) { wc = wc.set_one(w1); }

        let mut bc = BitBoard::empty();
        if let Some(b0) = SE.shift_coord(c0) { bc = bc.set_one(b0); }
        if let Some(b1) = SW.shift_coord(c0) { bc = bc.set_one(b1); }

        MoveSetPawn::new(wq, bq, wc, bc)
    }

}

mod eval {
    use crate::types::*;
    use crate::tables::*;
    use crate::evaluate::*;
    use crate::tuning::*;

    use serde_big_array::BigArray;

    #[derive(Serialize,Deserialize,Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct Wat {
        // #[serde(with = "BigArray")]
        // wat: [Score; 64],
        // #[serde(serialize_with = "<[BigArray]>::serialize")]
        // wat: [[Score; 64]; 2],
        // #[serde(serialize_with = "<[_]>::serialize")]
        // wat: [Score; 64],
        // #[serde(flatten)]


        // #[serde(with = "BigArray",flatten)]
        // wat: [[Score; 2]; 64],

        // #[serde(with = "<[_]>::serialize")]
        // #[serde(with = "BigArray",flatten)]
        // wat: [[Score; 64]; 6],

    }

    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    // #[derive(Serialize,Deserialize,Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct PcTables {
        // #[serde(with = "BigArray",flatten)]
        tables_mid:         [[Score; 64]; 6],
        // #[serde(with = "BigArray",flatten)]
        tables_end:         [[Score; 64]; 6],
        pub ev_pawn:        EvPawn,
        pub ev_rook:        EvRook,
        pub ev_knight:      EvKnight,
        pub ev_bishop:      EvBishop,
        pub ev_queen:       EvQueen,
        pub ev_king:        EvKing,
    }

    impl Default for PcTables {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PcTables {

        pub fn print_table(ss: [Score; 64]) {
            for y in 0..8 {
                let y = 7 - y;
                for x in 0..8 {
                    // println!("(x,y) = ({},{}), coord = {:?}", x, y, Coord(x,y));
                    // print!("{:>3?},", ps.get(Pawn, Coord(x,y)));
                    let s = ss[Coord(x,y)];
                    print!("{:>3?},", s);
                }
                print!("\n");
            }
        }

        pub fn get_mid<T: Into<Coord>>(&self, pc: Piece, col: Color, c0: T) -> Score {
            let c1: Coord = c0.into();
            let c1 = if col == White { c1 } else { Coord(c1.0,7-c1.1) };
            self.tables_mid[pc.index()][c1]
        }

        pub fn get_end<T: Into<Coord>>(&self, pc: Piece, col: Color, c0: T) -> Score {
            let c1: Coord = c0.into();
            let c1 = if col == White { c1 } else { Coord(c1.0,7-c1.1) };
            self.tables_end[pc.index()][c1]
        }

        // [
        //     0,  0,  0,  0,  0,  0,  0,  0,
        //     50, 50, 50, 50, 50, 50, 50, 50,
        //     0,  0,  0,  0,  0,  0,  0,  0,
        //     0,  0,  0,  0,  0,  0,  0,  0,
        //     0,  0,  0,  0,  0,  0,  0,  0,
        //     5, -5,-10,  0,  0,-10, -5,  5,
        //     5, 10, 10,-20,-20, 10, 10,  5,
        //     0,  0,  0,  0,  0,  0,  0,  0,
        // ]

    }

    /// Generate
    impl PcTables {

        pub fn new() -> Self {
            let pawns = Self::gen_pawns();
            let knights = Self::gen_knights();
            let kings = Self::gen_kings_opening();

            let out = Self {
                tables_mid: [pawns,
                             [0; 64], // Rook
                             knights,
                             [0; 64], // b
                             [0; 64], // q
                             kings,
                ],
                tables_end: [pawns,
                            [0; 64], // Rook
                            knights,
                            [0; 64], // b
                            [0; 64], // q
                            kings,
                ],
                ev_pawn:   EvPawn::new(),
                ev_rook:   EvRook::new(),
                ev_knight: EvKnight::new(),
                ev_bishop: EvBishop::new(),
                ev_queen:  EvQueen::new(),
                ev_king:   EvKing::new(),
            };

            out
            // (opening,endgame)
        }

        fn gen_pawns() -> [Score; 64] {
            // let mut out = [0; 64];
            let mut scores: Vec<(&str,Score)> = vec![];

            // Castles
            scores.push(("A2",5));
            scores.push(("B2",10));
            scores.push(("C2",10));

            // Castle holes
            scores.push(("A3",5));
            scores.push(("B3",-5));
            scores.push(("C3",-10));

            // King/Queen Pawns
            scores.push(("D2",-20));

            // Center pawns
            scores.push(("D4",20));

            // Rank 5 pawns
            scores.push(("A5",5));
            scores.push(("B5",5));
            scores.push(("C5",10));
            scores.push(("D5",25));

            // Rank 6 pawns
            scores.push(("A6",10));
            scores.push(("B6",10));
            scores.push(("C6",20));
            scores.push(("D6",30));

            // Rank 7 pawns
            scores.push(("A7",50));
            scores.push(("B7",50));
            scores.push(("C7",50));
            scores.push(("D7",50));

            let mut out = [0; 64];

            for (c,s) in scores.into_iter() {
                let c0: Coord = c.into();
                let sq: usize = c0.into();
                out[sq] = s;
                let c1 = Coord(7-c0.0,c0.1);
                let sq: usize = c1.into();
                out[sq] = s;
            }

            out
        }

        pub fn gen_knights() -> [Score; 64] {
            let mut scores: Vec<(&str,Score)> = vec![];

            let out = [
                0,  0,  0,  0,  0,  0,  0,  0,
                0,  0,  0,  0,  0,  0,  0,  0,
                0,  0,  0,  0,  0,  0,  0,  0,
                0,  0,  0,  0,  0,  0,  0,  0,
                0,  0,  0,  0,  0,  0,  0,  0,
                0,  0,  0,  0,  0,  0,  0,  0,
                0,  0,  0,  0,  0,  0,  0,  5,
                0,  0,  0,  0,  0,  0,  0,  0,
            ];

            // for sq in 0u32..64 {
            //     let c0: Coord = sq.into();
            //     let s: Score = c0.center_distance().into();
            //     let s = (3 - s) * 1;
            //     out[c0] = s;
            // }

            // let mut out = [0; 64];

            // for (c,s) in scores.into_iter() {
            //     let c0: Coord = c.into();
            //     let sq: usize = c0.into();
            //     out[sq] = s;
            //     let c1 = Coord(7-c0.0,c0.1);
            //     let sq: usize = c1.into();
            //     out[sq] = s;
            // }

            out
        }

        pub fn gen_kings_opening() -> [Score; 64] {
            [
                -30,-40,-40,-50,-50,-40,-40,-30,
                -30,-40,-40,-50,-50,-40,-40,-30,
                -30,-40,-40,-50,-50,-40,-40,-30,
                -30,-40,-40,-50,-50,-40,-40,-30,
                -20,-30,-30,-40,-40,-30,-30,-20,
                -10,-20,-20,-20,-20,-20,-20,-10,
                20, 20,  0,  0,  0,  0, 20, 20,
                20, 30, 10,  0,  0, 10, 30, 20
            ]
        }

    }

}

mod opening_book {
    use crate::types::*;
    use crate::tables::*;

}

mod endgames {
    use crate::types::*;
    use crate::tables::*;


}

mod magics {
    use itertools::iproduct;
    use rand::Rng;

    use crate::types::*;
    use crate::tables::*;

    // #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    #[derive(Serialize,Deserialize,Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct Magic {
        pub attacks:   usize,
        pub mask:      BitBoard,
        pub magic:     BitBoard,
        pub shift:     u8,
    }

    impl Magic {
        pub fn new(attacks: usize, mask: BitBoard, magic: BitBoard, shift: u8) -> Self {
            Self {
                attacks,
                // mask: BitBoard(0xff818181818181ff),
                mask,
                magic,
                shift,
            }
        }

        pub fn index(mask: BitBoard, magic: BitBoard, shift: u32, occ: BitBoard) -> u64 {
            // unsigned lo = unsigned(occupied) & unsigned(mask);
            let lo = occ.0 & mask.0;
            // unsigned hi = unsigned(occupied >> 32) & unsigned(mask >> 32);
            let hi = occ.0.overflowing_shr(32).0 & mask.0.overflowing_shr(32).0;

            // return (lo * unsigned(magic) ^ hi * unsigned(magic >> 32)) >> shift;
            let k0 = lo.overflowing_mul(magic.0.overflowing_pow(hi as u32).0).0
                .overflowing_mul(magic.0.overflowing_shr(32).0).0;
            let k1 = k0.overflowing_shr(shift).0;

            k1
        }

    }

    impl Tables {

        pub fn attacks_rook(&self, c0: Coord, occ: BitBoard) -> BitBoard {
            let sq: u32 = c0.into();
            let m = self.magics_rook[sq as usize];
            if m.magic.0 == 0 {
                panic!("Magics not initialized");
            }
            let mut occ = occ;
            let occ = (occ & m.mask).0;
            let occ = occ.overflowing_mul(m.magic.0).0;
            let occ = occ.overflowing_shr(m.shift as u32).0;
            self.table_rook[m.attacks + occ as usize]
        }

        pub fn attacks_bishop(&self, c0: Coord, occ: BitBoard) -> BitBoard {
            let sq: u32 = c0.into();
            let m = self.magics_bishop[sq as usize];
            if m.magic.0 == 0 {
                panic!("Magics not initialized");
            }
            let mut occ = occ;
            let occ = (occ & m.mask).0;
            let occ = occ.overflowing_mul(m.magic.0).0;
            let occ = occ.overflowing_shr(m.shift as u32).0;
            self.table_bishop[m.attacks + occ as usize]
        }

        fn sparse_rand(rng: &mut rand::rngs::ThreadRng) -> u64 {
            let x0: u64 = rng.gen();
            let x1: u64 = rng.gen();
            let x2: u64 = rng.gen();
            // let x3: u64 = rng.gen();
            // x0 & x1 & x2 & x3
            x0 & x1 & x2
        }

        pub fn gen_magics() -> (([Magic; 64], [BitBoard; 0x1480]), ([Magic; 64], [BitBoard; 0x19000])) {

            let (magics_b,table_b) = Self::_gen_magics(true).unwrap();
            if let Err((magics_r,table_r)) = Self::_gen_magics(false) {
                ((magics_b,table_b),(magics_r,table_r))
            } else { panic!("gen_magics") }

            // unimplemented!()
        }

        // pub fn gen_magics_rook2() -> ([Magic; 64], [BitBoard; 0x19000]) {
        //     let mut rng = rand::thread_rng();
        //     let mut reference: [BitBoard; 4096] = [BitBoard::empty(); 4096];
        //     let mut table: [BitBoard; 0x19000] = [BitBoard::empty(); 0x19000];
        //     let mut magics: [Option<Magic>; 64] = [None; 64];
        //     let mut size: usize = 0;
        //     let (r1bb,r8bb) = (BitBoard::mask_rank(0),BitBoard::mask_rank(7));
        //     let (f1bb,f8bb) = (BitBoard::mask_file(0),BitBoard::mask_file(7));
        //     let mut epoch = [0; 4096];
        //     let mut cnt   = 0;
        //     let mut size: usize = 0;

        //     for sq in 0u64..64 {
        //         let c0: Coord = sq.into();


        //         // let mask = Self::gen_blockermask_rook(c0) & !edges;


        //         // let m = Magic::new(attacks, mask, BitBoard(mm), shift);
        //         // magics[sq as usize] = Some(m);

        //     }

        //     // let magics: [Magic; 64] = array_init::array_init(|x| magics[x].unwrap());
        //     // (magics, table)
        //     unimplemented!()
        // }

        pub fn _gen_magics(bishop: bool)
                           -> std::result::Result<([Magic; 64], [BitBoard; 0x1480]), ([Magic; 64], [BitBoard; 0x19000])>
        {
            let mut rng = rand::thread_rng();
            let mut reference: [BitBoard; 4096] = [BitBoard::empty(); 4096];

            let mut table_b: [BitBoard; 0x1480]  = [BitBoard::empty(); 0x1480];
            let mut table_r: [BitBoard; 0x19000] = [BitBoard::empty(); 0x19000];

            let mut magics: [Option<Magic>; 64] = [None; 64];
            let (r1bb,r8bb) = (BitBoard::mask_rank(0),BitBoard::mask_rank(7));
            let (f1bb,f8bb) = (BitBoard::mask_file(0),BitBoard::mask_file(7));
            let mut epoch = [0; 4096];
            let mut cnt   = 0;
            let mut size: usize = 0;

            for sq in 0u8..64 {
            // for sq in 0..1 {
                // let c0: Coord = "A1".into();
                // let sq: u32 = c0.into();

                let c0: Coord = sq.into();
                // eprintln!("c0 = {:?}", c0);

                let edges: BitBoard =
                    ((BitBoard::mask_rank(0) | BitBoard::mask_rank(7)) & !BitBoard::mask_rank(c0.1 as u8))
                    | ((BitBoard::mask_file(0) | BitBoard::mask_file(7)) & !BitBoard::mask_file(c0.0 as u8));

                let mask = if bishop {
                    Self::gen_blockermask_bishop(c0) & !edges
                } else {
                    Self::gen_blockermask_rook(c0) & !edges
                };

                let shift = 64 - mask.popcount();

                let attacks = if sq == 0 {
                    0
                } else {
                    magics[sq as usize - 1].unwrap().attacks + size
                };

                let mbs = mask.iter_subsets();

                let n = mask.popcount();

                for (s,b) in mbs.iter().enumerate() {
                    if bishop {
                        reference[s] = Self::gen_moveboard_bishop(*b, c0);
                    } else {
                        reference[s] = Self::gen_moveboard_rook(*b, c0);
                    }
                    size = s + 1;
                }
                let mut mm: u64;

                // let mut xs = vec![];
                let mut done;
                'outer: loop {
                // 'outer: for _ in 0..1_000_000 {
                    // mm = rng.gen();
                    // mm = 0x48FFFE99FECFAA00;
                    // mm = 0x90a207c5e7ae23ff;

                    loop {
                        mm = Self::sparse_rand(&mut rng);
                        let k0 = mm.overflowing_mul(mask.0).0;
                        let k1 = k0.overflowing_shr(56).0;
                        if BitBoard(k1).popcount() < 6 {
                            break;
                        }
                    }

                    done = true;
                    cnt += 1;
                    'inner: for (s,b) in mbs.iter().enumerate() {
                        let result = reference[s];

                        let idx = b.0.overflowing_mul(mm).0;
                        let idx = idx.overflowing_shr((64 - n) as u32).0 as usize;

                        let tb = if bishop {
                            table_b[attacks + idx]
                        } else {
                            table_r[attacks + idx]
                        };
                        if epoch[idx] < cnt {
                            epoch[idx] = cnt;
                            if bishop {
                                table_b[attacks + idx] = result;
                            } else {
                                table_r[attacks + idx] = result;
                            }
                            // xs.push(attacks + idx);
                        } else if tb.0 != result.0 {
                            done = false;
                            break 'inner;
                        }

                        // let tb = table[attacks + idx];
                        // if tb.is_empty() {
                        //     table[attacks + idx] = result;
                        //     xs.push(attacks + idx);
                        // } else if tb.0 != result.0 {
                        //     done = false;
                        //     break 'inner;
                        // }

                    };

                    if done {
                        break 'outer
                    }

                    // if done {
                    //     break 'outer
                    // } else {
                    //     for i in xs.iter() {
                    //         table[*i] = BitBoard::empty();
                    //     }
                    //     xs.clear()
                    // }

                }

                if mm == 0 {
                    panic!("wot");
                }

                let m = Magic::new(attacks, mask, BitBoard(mm), shift);
                magics[sq as usize] = Some(m);

                // for (idx, result) in results.into_iter() {
                //     table[idx] = result;
                // }

                // eprintln!("n = {:?}", n);
                // eprintln!("2^n = {:?}", 2u64.pow(n));

                // eprintln!("mbs.len() = {:?}", mbs.len());

                // let mbs = vec![
                //     BitBoard::new(&["A6","D1"]),
                // ];

                // let mut rng = rand::thread_rng();
                // let mut mm: u64;
                // for (s,b) in mbs.iter().enumerate() {

                    // loop {
                    //     mm = rng.gen();
                    //     let k0 = b.0.overflowing_mul(mm).0;
                    //     let k1 = 2u64.pow(mask.popcount());
                    //     let k2 = 64u64.overflowing_sub(k1).0;
                    //     let k3 = k0.overflowing_shr(k2 as u32).0;
                    //     if k3 != 0 { break; }
                    // }

                    // // eprintln!("magic? = {:?}", BitBoard(mm));
                    // m.magic = BitBoard(mm);
                    // cnt += 1;
                    // for i in 0..size {
                    //     let idx = Magic::index(mask, BitBoard(mm), shift, occupancy[i]) as usize;
                    //     if epoch[idx] < cnt {
                    //         epoch[idx] = cnt;
                    //         table[attacks + idx] = reference[i];
                    //     } else if table[attacks + idx] != reference[i] {
                    //         break;
                    //     }
                    // }

                // }

            }

            let magics: [Magic; 64] = array_init::array_init(|x| magics[x].unwrap());
            if bishop {
                Ok((magics, table_b))
            } else {
                Err((magics, table_r))
            }

            // unimplemented!()

        }

        pub fn gen_blockermask_rook(c0: Coord) -> BitBoard {
            // let b0 = BitBoard(0xff818181818181ff);
            let b1 = BitBoard::mask_file(c0.0 as u8)
                | BitBoard::mask_rank(c0.1 as u8);
            // (!b0 & b1).set_zero(c0)
            b1.set_zero(c0)
        }

        pub fn gen_blockermask_bishop(c0: Coord) -> BitBoard {
            // let b0 = BitBoard(0xff818181818181ff);

            let b1 = Self::gen_diagonal(c0, true)
                | Self::gen_diagonal(c0, false)
                | Self::gen_antidiagonal(c0, true)
                | Self::gen_antidiagonal(c0, false);

            // (!b0 & b1).set_zero(c0)
            b1.set_zero(c0)
        }

        pub fn gen_moveboard_rook(occ: BitBoard, c0: Coord) -> BitBoard {
            let mut out = BitBoard::empty();
            let ds_rook   = [N,E,W,S];
            for d in ds_rook.iter() {
                let mut c1 = c0;
                loop {
                    if let Some(c2) = d.shift_coord(c1) {
                        if (c1.square_dist(c2) <= 2) & occ.is_zero_at(c1) {
                            out.set_one_mut(c2);
                            c1 = c2;
                        } else { break; }
                    } else { break; }
                }
            }
            out
        }

        pub fn gen_moveboard_bishop(occ: BitBoard, c0: Coord) -> BitBoard {
            let mut out = BitBoard::empty();
            let ds_bishop = [NE,NW,SE,SW];
            for d in ds_bishop.iter() {
                let mut c1 = c0;
                loop {
                    if let Some(c2) = d.shift_coord(c1) {
                        if (c1.square_dist(c2) <= 2) & occ.is_zero_at(c1) {
                            out.set_one_mut(c2);
                            c1 = c2;
                        } else { break; }
                    } else { break; }
                }
            }
            out
        }

        fn gen_blockerboard(blockermask: BitBoard, index: usize) -> BitBoard {
            unimplemented!()
        }

    }

}

mod movesets {
    use crate::types::*;

    use serde::ser::{Serialize, SerializeStruct, Serializer};

    // #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    // pub struct MoveIter {
    //     n:     Option<usize>,
    //     moves: [(D,BitBoard); 8],
    // }

    // impl Iterator for MoveIter {
    //     type Item = (D,BitBoard);
    //     fn next(&mut self) -> Option<Self::Item> {
    //         match self.n {
    //             None => None,
    //             Some(0) => {
    //                 let out = self.moves[0];
    //                 self.n = None;
    //                 Some(out)
    //             },
    //             Some(nn) => {
    //                 let out = self.moves[7 - nn];
    //                 self.n = Some(nn - 1);
    //                 Some(out)
    //             },
    //         }
    //     }
    // }

    // impl MoveIter {
    //     pub fn new_4(ms: [(D,BitBoard); 4]) -> Self {
    //         Self {
    //             n:     Some(3),
    //             moves: 
    //         }
    //     }
    // }

    /// pub n: BitBoard,
    /// pub e: BitBoard,
    /// pub w: BitBoard,
    /// pub s: BitBoard,
    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct MoveSetRook {
        pub n: BitBoard,
        pub e: BitBoard,
        pub w: BitBoard,
        pub s: BitBoard,
    }

    impl serde::Serialize for MoveSetRook {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut state = serializer.serialize_struct("MoveSetRook", 4)?;
            state.serialize_field("n", &self.n)?;
            state.serialize_field("e", &self.e)?;
            state.serialize_field("w", &self.w)?;
            state.serialize_field("s", &self.s)?;
            state.end()
        }
    }

    impl MoveSetRook {

        // pub fn to_iter<'a>(&'a self) -> impl Iterator<Item = (D,BitBoard)> + 'a {
        //     let xs: [(D,BitBoard); 4] = [(N,self.n),(E,self.e),(W,self.w),(S,self.s)];
        //     xs.into_iter().cloned()
        // }

        // pub fn to_iter(&self) -> impl Iterator<Item = (D,BitBoard)> {
        //     unimplemented!()
        // }

        pub fn to_vec(&self) -> [(D,BitBoard); 4] {
            [(N,self.n),(E,self.e),(W,self.w),(S,self.s)]
        }

        pub fn to_vec_with_bishop(&self, ms: [(D,BitBoard); 4]) -> [(D,BitBoard); 8] {
            [(N,self.n),(E,self.e),(W,self.w),(S,self.s),ms[0],ms[1],ms[2],ms[3]]
        }

        // pub fn to_vec(&self) -> Vec<(D,BitBoard)> {
        //     vec![(N,self.n),(E,self.e),(W,self.w),(S,self.s)]
        // }

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
        pub fn concat(&self) -> BitBoard {
            self.n | self.e | self.w | self.s
        }
    }

    /// pub ne: BitBoard,
    /// pub nw: BitBoard,
    /// pub se: BitBoard,
    /// pub sw: BitBoard,
    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct MoveSetBishop {
        pub ne: BitBoard,
        pub nw: BitBoard,
        pub se: BitBoard,
        pub sw: BitBoard,
    }

    impl serde::Serialize for MoveSetBishop {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut state = serializer.serialize_struct("MoveSetBishop", 4)?;
            state.serialize_field("ne", &self.ne)?;
            state.serialize_field("nw", &self.nw)?;
            state.serialize_field("se", &self.se)?;
            state.serialize_field("sw", &self.sw)?;
            state.end()
        }
    }

    impl MoveSetBishop {

        // pub fn to_iter(&self) -> MoveIter {
        // }

        pub fn to_vec(&self) -> [(D,BitBoard); 4] {
            [(NE,self.ne),(NW,self.nw),(SE,self.se),(SW,self.sw)]
        }

        pub fn to_vec_with_rook(&self, ms: [(D,BitBoard); 4]) -> [(D,BitBoard); 8] {
            [(NE,self.ne),(NW,self.nw),(SE,self.se),(SW,self.sw),ms[0],ms[1],ms[2],ms[3]]
        }

        // pub fn to_vec(&self) -> Vec<(D,BitBoard)> {
        //     vec![(NE,self.ne),(NW,self.nw),(SE,self.se),(SW,self.sw)]
        // }

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
        pub fn concat(&self) -> BitBoard {
            self.ne | self.nw | self.se | self.sw
        }
    }

    /// pub white_quiet:   BitBoard,
    /// pub black_quiet:   BitBoard,
    /// pub white_capture: BitBoard,
    /// pub black_capture: BitBoard,
    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct MoveSetPawn {
        pub white_quiet:   BitBoard,
        pub black_quiet:   BitBoard,
        pub white_capture: BitBoard,
        pub black_capture: BitBoard,
    }

    impl serde::Serialize for MoveSetPawn {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut state = serializer.serialize_struct("MoveSetPawn", 4)?;
            state.serialize_field("wq", &self.white_quiet)?;
            state.serialize_field("bq", &self.black_quiet)?;
            state.serialize_field("wc", &self.white_capture)?;
            state.serialize_field("bc", &self.black_capture)?;
            state.end()
        }
    }

    impl MoveSetPawn {
        pub fn empty() -> Self {
            Self {
                white_quiet:   BitBoard::empty(),
                black_quiet:   BitBoard::empty(),
                white_capture: BitBoard::empty(),
                black_capture: BitBoard::empty(),
            }
        }
        pub fn new(white_quiet:   BitBoard,
                black_quiet:   BitBoard,
                white_capture: BitBoard,
                black_capture: BitBoard) -> Self {
            Self {
                white_quiet,
                black_quiet,
                white_capture,
                black_capture,
            }
        }
        pub fn get_quiet(&self, c: Color) -> &BitBoard {
            match c {
                White => &self.white_quiet,
                Black => &self.black_quiet,
            }
        }
        pub fn get_capture(&self, c: Color) -> &BitBoard {
            match c {
                White => &self.white_capture,
                Black => &self.black_capture,
            }
        }
    }

}

