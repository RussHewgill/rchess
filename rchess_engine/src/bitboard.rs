
use crate::types::*;
use crate::tables::*;

pub use self::lookups::*;

use serde::{Serialize,Deserialize};
use evmap_derive::ShallowCopy;
use derive_more::*;

mod lookups {
    use crate::types::*;
    use super::BitBoard;

    use lazy_static::lazy_static;

    pub const MASK_FILES: [BitBoard; 8] = [
        BitBoard(0x0101010101010101),
        BitBoard(0x0202020202020202),
        BitBoard(0x0404040404040404),
        BitBoard(0x0808080808080808),
        BitBoard(0x1010101010101010),
        BitBoard(0x2020202020202020),
        BitBoard(0x4040404040404040),
        BitBoard(0x8080808080808080),
    ];

    pub const MASK_RANKS: [BitBoard; 8] = [
        BitBoard(0x00000000000000ff),
        BitBoard(0x000000000000ff00),
        BitBoard(0x0000000000ff0000),
        BitBoard(0x00000000ff000000),
        BitBoard(0x000000ff00000000),
        BitBoard(0x0000ff0000000000),
        BitBoard(0x00ff000000000000),
        BitBoard(0xff00000000000000),
    ];

    pub const WHITE_SQUARES: BitBoard = BitBoard(0x55AA55AA55AA55AA);
    pub const BLACK_SQUARES: BitBoard = BitBoard(0xAA55AA55AA55AA55);

    pub const DIAG_A1_H8: BitBoard = BitBoard(0x8040201008040201);
    pub const DIAG_A8_H1: BitBoard = BitBoard(0x0102040810204080);

    const fn forward_ranks_bb(side: Color, sq: Coord) -> BitBoard {
        match side {
            White => BitBoard((!MASK_RANKS[0].0) << (8 * BitBoard::relative_rank(White, sq) as u32)),
            Black => BitBoard((!MASK_RANKS[7].0) >> (8 * BitBoard::relative_rank(Black, sq) as u32)),
        }
    }

    // pub const FORWARD_FILE_BB: [BitBoard; 64] = 

    pub const SQUARE_BB: [BitBoard; 64] = [
        BitBoard(1 << 0),
        BitBoard(1 << 1),
        BitBoard(1 << 2),
        BitBoard(1 << 3),
        BitBoard(1 << 4),
        BitBoard(1 << 5),
        BitBoard(1 << 6),
        BitBoard(1 << 7),
        BitBoard(1 << 8),
        BitBoard(1 << 9),
        BitBoard(1 << 10),
        BitBoard(1 << 11),
        BitBoard(1 << 12),
        BitBoard(1 << 13),
        BitBoard(1 << 14),
        BitBoard(1 << 15),
        BitBoard(1 << 16),
        BitBoard(1 << 17),
        BitBoard(1 << 18),
        BitBoard(1 << 19),
        BitBoard(1 << 20),
        BitBoard(1 << 21),
        BitBoard(1 << 22),
        BitBoard(1 << 23),
        BitBoard(1 << 24),
        BitBoard(1 << 25),
        BitBoard(1 << 26),
        BitBoard(1 << 27),
        BitBoard(1 << 28),
        BitBoard(1 << 29),
        BitBoard(1 << 30),
        BitBoard(1 << 31),
        BitBoard(1 << 32),
        BitBoard(1 << 33),
        BitBoard(1 << 34),
        BitBoard(1 << 35),
        BitBoard(1 << 36),
        BitBoard(1 << 37),
        BitBoard(1 << 38),
        BitBoard(1 << 39),
        BitBoard(1 << 40),
        BitBoard(1 << 41),
        BitBoard(1 << 42),
        BitBoard(1 << 43),
        BitBoard(1 << 44),
        BitBoard(1 << 45),
        BitBoard(1 << 46),
        BitBoard(1 << 47),
        BitBoard(1 << 48),
        BitBoard(1 << 49),
        BitBoard(1 << 50),
        BitBoard(1 << 51),
        BitBoard(1 << 52),
        BitBoard(1 << 53),
        BitBoard(1 << 54),
        BitBoard(1 << 55),
        BitBoard(1 << 56),
        BitBoard(1 << 57),
        BitBoard(1 << 58),
        BitBoard(1 << 59),
        BitBoard(1 << 60),
        BitBoard(1 << 61),
        BitBoard(1 << 62),
        BitBoard(1 << 63),
    ];

}

#[derive(Serialize,Deserialize,Hash,Eq,PartialEq,PartialOrd,Clone,Copy,ShallowCopy,
         Index,Add,Mul,Div,Sum,AddAssign,MulAssign,
         BitAnd,BitAndAssign,BitOr,BitOrAssign,BitXor,BitXorAssign,Not,
         From,Into,AsRef,AsMut
)]
pub struct BitBoard(pub u64);

impl Iterator for BitBoard {
    type Item = Coord;
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(self.bitscan_reset_mut())
        }
    }
}

/// new
impl BitBoard {
    pub fn new<T: Into<Coord> + Copy>(cs: &[T]) -> Self {
        let mut b = BitBoard(0);
        for &c in cs.iter() {
            b |= BitBoard::single(c.into());
        }
        b
    }

    pub const fn empty() -> BitBoard {
        BitBoard(0)
    }

    pub fn single(c: Coord) -> BitBoard {
        SQUARE_BB[c]
        // let k = 1u64.overflowing_shl(c.inner() as u32).0;
        // BitBoard(k)
    }

}

/// Queries
impl BitBoard {

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn is_not_empty(&self) -> bool {
        self.0 != 0
    }

    pub fn is_one_at(&self, c0: Coord) -> bool {
        (*self & BitBoard::single(c0)).is_not_empty()
    }

    pub fn is_zero_at<T: Into<Coord>>(&self, c0: T) -> bool {
        (*self & BitBoard::single(c0.into())).is_empty()
    }

    pub fn more_than_one(&self) -> bool {
        if self.is_empty() { return false; }
        let b = self.bitscan_reset().0;
        b.is_not_empty()
    }

}

/// Masks
impl BitBoard {

    pub fn mask_rank(r: u8) -> BitBoard {
        // assert!(r < 8);
        // let k = (!0u8) as u64;
        // BitBoard(k.overflowing_shl(r * 8).0)
        MASK_RANKS[r as usize]
    }

    pub fn mask_file(f: u8) -> BitBoard {
        // assert!(f < 8);
        // let k = 0x0101010101010101u64;
        // BitBoard(k.overflowing_shl(f).0)
        MASK_FILES[f as usize]
    }

    pub fn mask_between(ts: &Tables, c0: Coord, c1: Coord) -> BitBoard {
    // pub fn obstructed(&self, ts: &Tables, c0: Coord, c1: Coord) -> BitBoard {

        let (x0,y0) = c0.to_rankfile();
        let (x1,y1) = c1.to_rankfile();

        if x0 == x1 {
            // File
            let (x0,x1) = (x0.min(x1),x0.max(x1));
            let (y0,y1) = (y0.min(y1),y0.max(y1));
            let b0 = BitBoard::single(Coord::new(x0,y0));
            let b1 = BitBoard::single(Coord::new(x1,y1));
            let b = BitBoard(2 * b1.0 - b0.0);
            let m = BitBoard::mask_file(x0.into());
            (b & m) & !(b0 | b1)
        } else if y0 == y1 {
            // Rank
            let (x0,x1) = (x0.min(x1),x0.max(x1));
            let (y0,y1) = (y0.min(y1),y0.max(y1));
            let b0 = BitBoard::single(Coord::new(x0,y0));
            let b1 = BitBoard::single(Coord::new(x1,y1));
            let b = BitBoard(2 * b1.0 - b0.0);
            let m = BitBoard::mask_rank(y0.into());
            (b & m) & !(b0 | b1)
        // } else if (x1 - x0) == (y1 - y0) {
        } else if (x1 as i64 - x0 as i64).abs() == (y1 as i64 - y0 as i64).abs() {
            // Diagonal
            let b0 = BitBoard::single(Coord::new(x0,y0));
            let b1 = BitBoard::single(Coord::new(x1,y1));
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
            let m = ts.get_bishop(c0);

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


}

/// Bitscan
impl BitBoard {

    // fn bitscan_unchecked(&self) -> Coord {
    pub fn bitscan(&self) -> Coord {
        // assert!(self.is_not_empty());
        Coord::new_int(self.0.trailing_zeros() as u8)
    }

    pub fn bitscan_checked(&self) -> Option<Coord> {
        if self.is_empty() {
            None
        } else {
            Some(self.bitscan())
        }
    }

    pub fn bitscan_reset(&self) -> (Self, Coord) {
        // let x = self.bitscan().unwrap();
        let x = self.bitscan();
        // (*self & BitBoard(self.0.overflowing_sub(1).0),x)
        (*self & !Self::single((x).into()),x)
    }

    pub fn bitscan_reset_mut(&mut self) -> Coord {
        let (b,x) = self.bitscan_reset();
        *self = b;
        x
    }

}

/// Modification
impl BitBoard {

    pub fn extend_mut<T: IntoIterator<Item = C>, C: Into<Coord>>(&mut self, iter: T) {
        for c0 in iter {
            self.set_one_mut(c0.into());
        }
    }

    #[must_use]
    pub fn set_one(&self, c: Coord) -> Self {
        let b = Self::single(c);
        *self | b
    }

    #[must_use]
    pub fn set_zero(&self, c: Coord) -> Self {
        let b = Self::single(c);
        *self & !b
    }

    pub fn set_one_mut(&mut self, c: Coord) {
        let b = Self::single(c);
        *self |= b;
    }

    pub fn set_zero_mut(&mut self, c: Coord) {
        let b = Self::single(c);
        *self &= !b;
    }

    pub fn flip(&self, c: Coord) -> Self {
        let b = Self::single(c);
        *self ^ b
    }

    pub fn flip_mut(&mut self, c: Coord) {
        let b = Self::single(c);
        *self ^= b;
    }

}

/// popcount
impl BitBoard {

    #[cfg(target_feature = "popcnt")]
    pub fn popcount(&self) -> u8 {
        let k = unsafe { core::arch::x86_64::_popcnt64(self.0 as i64) };
        k as u8
    }

    #[cfg(not(target_feature = "popcnt"))]
    pub fn popcount(&self) -> u8 {

        const K1: u64 = 0x5555555555555555; /*  -1/3   */
        const K2: u64 = 0x3333333333333333; /*  -1/5   */
        const K4: u64 = 0x0f0f0f0f0f0f0f0f; /*  -1/17  */
        const KF: u64 = 0x0101010101010101; /*  -1/255 */
        let mut x: u64 = self.0;
        x =  x       - ((x >> 1)  & K1); /* put count of each 2 bits into those 2 bits */
        x = (x & K2) + ((x >> 2)  & K2); /* put count of each 4 bits into those 4 bits */
        x = (x       +  (x >> 4)) & K4 ; /* put count of each 8 bits into those 8 bits */
        /* returns 8 most significant bits of x + (x<<8) + (x<<16) + (x<<24) + ...  */
        x = (x.overflowing_mul(KF)).0 >> 56;
        x as u8

    }

}

/// iter_subsets
impl BitBoard {
    /// Used for magic gen
    pub fn iter_subsets(&self) -> Vec<BitBoard> {
        let mut out = vec![];
        let mut n: u64 = 0;

        loop {
            out.push(BitBoard(n));

            // n = (n - self.0) & self.0;
            n = (n.overflowing_sub(self.0).0) & self.0;

            if n == 0 { break; }
        }

        out
    }
}

/// Indexing
impl BitBoard {

    pub const fn relative_rank(side: Color, sq: Coord) -> u8 {
        match side {
            White => sq.rank(),
            Black => 7 - sq.rank(),
        }
    }

    pub fn relative_square(side: Color, sq: Coord) -> Coord {
        Coord::new(Self::relative_rank(side, sq), sq.file())
    }

}

/// Fills
impl BitBoard {

    pub fn fill_north(&self) -> Self {
        let mut b = self.0;
        b |= b.overflowing_shl(8).0;
        b |= b.overflowing_shl(16).0;
        b |= b.overflowing_shl(32).0;
        BitBoard(b)
    }

    pub fn fill_south(&self) -> Self {
        let mut b = self.0;
        b |= b.overflowing_shr(8).0;
        b |= b.overflowing_shr(16).0;
        b |= b.overflowing_shr(32).0;
        BitBoard(b)
    }

}

/// Shift
impl BitBoard {

    pub fn shift_dir(&self, d: D) -> Self {
        match d {
            D::N  => BitBoard(self.0.overflowing_shl(8).0),
            D::NE => BitBoard(self.0.overflowing_shl(9).0),
            D::E  => BitBoard(self.0.overflowing_shl(1).0)
                & !MASK_FILES[0],
            D::SE => BitBoard(self.0.overflowing_shr(7).0)
                & !MASK_FILES[0],
            D::S  => BitBoard(self.0.overflowing_shr(8).0),
            D::SW => BitBoard(self.0.overflowing_shr(9).0)
                & !MASK_FILES[7],
            D::W  => BitBoard(self.0.overflowing_shr(1).0)
                & !MASK_FILES[7],
            D::NW => BitBoard(self.0.overflowing_shl(7).0)
                & !MASK_FILES[7],
        }
    }

    pub fn shift_mult(&self, d: D, n: u64) -> Self {
        let mut out = *self;
        for _ in 0..n {
            out = out.shift_dir(d);
        }
        out
    }

}

/// Rotate, Mirror, etc
impl BitBoard {

    pub fn mirror_vert(&self) -> Self {
        Self(self.0.swap_bytes())
    }

    pub fn mirror_horiz(&self) -> Self {
        // https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#Horizontal
        let mut x = self.0;
        const K1: u64 = 0x5555555555555555;
        const K2: u64 = 0x3333333333333333;
        const K4: u64 = 0x0f0f0f0f0f0f0f0f;
        x = ((x >> 1) & K1) +  2*(x & K1);
        x = ((x >> 2) & K2) +  4*(x & K2);
        x = ((x >> 4) & K4) + 16*(x & K4);
        Self(x)
    }

    pub fn rotate_90_cw(&self) -> Self {
        self.flip_diag().mirror_vert()
    }

    pub fn rotate_90_ccw(&self) -> Self {
        self.mirror_vert().flip_diag()
    }

    pub fn rotate_45_cw(&self) -> Self {
        const K1: u64 = 0xAAAAAAAAAAAAAAAA;
        const K2: u64 = 0xCCCCCCCCCCCCCCCC;
        const K4: u64 = 0xF0F0F0F0F0F0F0F0;
        let mut x = self.0;
        x ^= K1 & (x ^ x.rotate_right(8));
        x ^= K2 & (x ^ x.rotate_right(16));
        x ^= K4 & (x ^ x.rotate_right(32));
        Self(x)
        // unimplemented!()
    }

    pub fn rotate_45_ccw(&self) -> Self {
        const K1: u64 = 0x5555555555555555;
        const K2: u64 = 0x3333333333333333;
        const K4: u64 = 0x0f0f0f0f0f0f0f0f;
        let mut x = self.0;
        x ^= K1 & (x ^ x.rotate_right(8));
        x ^= K2 & (x ^ x.rotate_right(16));
        x ^= K4 & (x ^ x.rotate_right(32));
        Self(x)
        // unimplemented!()
    }

    pub fn rotate_180(&self) -> Self {
        Self(self.0.reverse_bits())
    }

    pub fn flip_antidiag(&self) -> Self {
        const K1: u64 = 0xaa00aa00aa00aa00;
        const K2: u64 = 0xcccc0000cccc0000;
        const K4: u64 = 0xf0f0f0f00f0f0f0f;
        let mut x = self.0;
        let mut t  = x ^ (x << 36) ;
        x ^= K4 & (t ^ (x >> 36));
        t  = K2 & (x ^ (x << 18));
        x ^=       t ^ (t >> 18) ;
        t  = K1 & (x ^ (x <<  9));
        x ^=       t ^ (t >>  9) ;
        Self(x)
    }

    pub fn flip_diag(&self) -> Self {
        const K1: u64 = 0x5500550055005500;
        const K2: u64 = 0x3333000033330000;
        const K4: u64 = 0x0f0f0f0f00000000;
        let mut x  = self.0;
        let mut t  = K4 & (x ^ (x << 28));
        x ^= t ^ (t >> 28) ;
        t = K2 & (x ^ (x << 14));
        x ^= t ^ (t >> 14) ;
        t = K1 & (x ^ (x <<  7));
        x ^= t ^ (t >>  7) ;
        Self(x)
    }

}

/// Display
impl BitBoard {
    pub fn print_hex(&self) -> String {
        format!("hex: {:#8x}", self.0)
    }
}

impl Default for BitBoard {
    fn default() -> Self {
        Self(0)
    }
}

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ss: &str = &format!("{:0>64b}", self.0);
        f.write_str(&format!("BitBoard:\n"))?;
        for y in 0..8 {
            let (c, rest) = ss.split_at(8);
            let c = c.chars().rev().collect::<String>();
            if y == 7 {
                f.write_str(&format!("{}", c))?;
            } else {
                f.write_str(&format!("{}\n", c))?;
            }
            ss = rest;
        }
        Ok(())
    }
}


