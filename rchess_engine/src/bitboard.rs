
use crate::types::*;
use crate::tables::*;

use serde::{Serialize,Deserialize};
use evmap_derive::ShallowCopy;
use derive_more::*;

// #[derive(Hash,Eq,PartialEq,PartialOrd,Clone,Copy)]
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

// impl serde::Serialize for BitBoard {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         serializer.serialize_newtype_struct("BitBoard", &self.0)
//     }
// }

// impl serde::Serialize for [BitBoard; 64] {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         // serializer.serialize_newtype_struct("BitBoard", &self.0)
//         unimplemented!()
//     }
// }

/// creation
impl BitBoard {

    pub fn new<T>(cs: &[T]) -> BitBoard where
        T: Into<Coord> + Copy,
    {
        let mut b = BitBoard::empty();
        for c in cs.iter() {
            b.flip_mut((*c).into());
        }
        b
    }

    pub fn empty() -> BitBoard {
        BitBoard(0)
    }

    pub fn _single<T: Into<Coord>>(c: T) -> BitBoard {
        Self::single(c.into())
    }

    pub fn single(c: Coord) -> BitBoard {
        let mut b = BitBoard::empty();
        b.flip_mut(c);
        b
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

    pub fn is_zero_at<T: Into<Coord>>(&self, c0: T) -> bool {
        (*self & BitBoard::single(c0.into())).is_empty()
    }

    // pub fn is_one_at<T: Into<Coord>>(&self, c0: T) -> bool {
        // (*self & BitBoard::single(c0.into())).is_not_empty()
    #[inline(always)]
    pub fn is_one_at(&self, c0: Coord) -> bool {
        (*self & BitBoard::single(c0)).is_not_empty()
    }

    pub fn more_than_one(&self) -> bool {
        let b = self.bitscan_reset().0;
        b.is_not_empty()
    }

    // pub fn get(&self, c: Coord) -> bool {
    //     // let p = Self::index_square(c);
    //     let p = c.inner();
    //     let k = (self.0 >> p) & 1;
    //     k == 1
    // }

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
        // let p = Self::index_square(c);
        let p = c.inner();
        let k = 1u64.overflowing_shl(p as u32).0;
        BitBoard(self.0 | k)
    }

    pub fn flip_mut(&mut self, c: Coord) {
        // let p = Self::index_square(c);
        let p = c.inner();
        // eprintln!("c, p = {:?}, {:?}", c, p);
        // let k = 1 << p;
        let k = 1u64.overflowing_shl(p as u32).0;
        self.0 |= k;
    }

    pub fn flip_mut_mult(&mut self, cs: &[Coord]) {
        for c in cs.iter() {
            self.flip_mut(*c);
        }
    }

}

/// Masks, Bitscan
impl BitBoard {

    // #[inline]
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

    pub fn bitscan_safe(&self) -> Option<Coord> {
        if self.is_empty() {
            None
        } else {
            Some(self.bitscan())
        }
    }

    pub fn bitscan(&self) -> Coord {
        // Bitscan Forward
        // self.0.leading_zeros()
        // self.0.trailing_zeros() as u8
        Coord::new_int(self.0.trailing_zeros() as u8)

        // // XXX: No Improvement
        // std::intrinsics::cttz(self.0) as u8
        // unsafe { core::arch::x86_64::_tzcnt_u64(self.0) as u8 }
    }

    pub fn bitscan_isolate(&self) -> Self {
        // let x = self.bitscan() as u32;
        // Self::single(x.into())
        Self::single(self.bitscan())
    }

    // pub fn bitscan_rev_isolate(&self) -> Self {
    //     let x = self.bitscan_rev() as u32;
    //     Self::single(x.into())
    // }

    pub fn bitscan_reset(&self) -> (Self, Coord) {
        let x = self.bitscan();
        // (*self & BitBoard(self.0.overflowing_sub(1).0),x)
        (*self & !Self::single((x).into()),x)
    }

    pub fn bitscan_reset_mut(&mut self) -> Coord {
        let (b,x) = self.bitscan_reset();
        *self = b;
        x
    }

    pub fn bitscan_rev(&self) -> Coord {
        // Bitscan Reverse
        Coord::new_int(63 - self.0.leading_zeros() as u8)
    }

    pub fn bitscan_rev_reset(&self) -> (Self, Coord) {
        let x = self.bitscan_rev();
        (*self & !Self::single((x).into()),x)
    }

    pub fn bitscan_rev_reset_mut(&mut self) -> Coord {
        let (b,x) = self.bitscan_rev_reset();
        *self = b;
        x
    }

    // pub fn serialize(&self) -> Vec<Coord> {
    //     let mut b = *self;
    //     let mut out = vec![];
    //     let mut x;
    //     loop {
    //         x = b.bitscan_reset_mut();
    //         // out.push(Self::index_bit(x as u64));
    //         out.push(x.into());
    //         if b.0 == 0 {
    //             break;
    //         }
    //     }
    //     out
    // }

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

    // pub fn popcount2(&self) -> u8 {
    //     let k = unsafe { core::arch::x86_64::_popcnt64(self.0 as i64) };
    //     k as u8
    // }

}

/// iter_bitscan
impl BitBoard {

    // pub fn iter_bitscan<F>(&self, mut f: F)
    // where F: FnMut(u32) {
    //     let mut b = *self;
    //     while b.0 != 0 {
    //         let p = b.bitscan_reset_mut();
    //         f(p);
    //     }
    // }

    // pub fn into_iter_rev(&mut self) -> impl Iterator<Item = u32> 

    // fn next(&mut self) -> Option<Self::Item> {
    //     if self.is_empty() {
    //         None
    //     } else {
    //         Some(self.bitscan_reset_mut())
    //     }
    // }

    // pub fn iter_bitscan<F>(self, mut f: F)
    // // where F: FnMut(u32) {
    // where F: FnMut(u8) {
    //     self.into_iter().for_each(|p| { f(p); })
    // }

    // pub fn iter_bitscan_rev<F>(&self, mut f: F)
    // where F: FnMut(u8) {
    //     let mut b = *self;
    //     while b.0 != 0 {
    //         let p = b.bitscan_rev_reset_mut();
    //         f(p);
    //     }
    // }

    // pub fn iter_subsets(&self) -> impl Iterator<Item = BitBoard> {
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

    // pub fn rank<T: Into<Coord>>(sq: T) -> u8 {
    // }

    pub fn relative_rank(side: Color, sq: Coord) -> u8 {
        match side {
            // White => Self::index_rank(sq.into()),
            // Black => 7 - Self::index_rank(sq.into()),
            White => sq.rank(),
            Black => 7 - sq.rank(),
        }
    }

    // pub fn relative_square<T: Into<u8> + Copy>(side: Color, sq: T) -> u8 {
    //     Self::_index_square(Self::relative_rank(side, sq), Self::index_file(sq.into()))
    // }
    pub fn relative_square(side: Color, sq: Coord) -> Coord {
        Coord::new(Self::relative_rank(side, sq), sq.file())
    }

    // pub fn index_square(c: Coord) -> u8 {
    //     // Little Endian Rank File Mapping
    //     // Least Significant File Mapping
    //     // let p = c.0 as u8 + 8 * c.1 as u8;
    //     // p
    //     c.inner()
    // }

    // pub fn _index_square(rank: u8, file: u8) -> u8 {
    //     rank * 8 + file
    // }

    // pub fn index_bit<T: Into<u8> + Copy>(s: T) -> Coord {
    //     Coord::new(Self::index_file(s.into()) as u8,Self::index_rank(s.into()) as u8)
    //     // Coord
    // }

    // pub fn index_rank(s: u8) -> u8 {
    //     s >> 3
    // }

    // pub fn index_file(s: u8) -> u8 {
    //     s & 7
    // }

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

/// Shift, Rotate, Mirror, etc
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

    #[cfg(feature = "nope")]
    pub fn shift_dir(&self, d: D) -> Self {
        // d.shift_coord(x);
        unimplemented!()
    }

    // #[cfg(feature = "nope")]
    pub fn shift_dir(&self, d: D) -> Self {

        // let k = d.shift();
        // let b = if k > 0 {
        //     self.shift_left(k as u32)
        // } else {
        //     self.shift_right(k.abs() as u32)
        // };

        // let k = d.shift();
        // // let b = self.0 << k;
        // let b = if k > 0 {
        //     self.0.overflowing_shl(k.abs() as u32).0
        //         & (!BitBoard::mask_file(0)).0
        // } else {
        //     self.0.overflowing_shr(k.abs() as u32).0
        //         & (!BitBoard::mask_file(7)).0
        // };
        // TODO: unwrap

        let b = match d {
            D::N  => {
                self.0.overflowing_shl(8 as u32).0
            },
            D::NE => {
                self.0.overflowing_shl(9 as u32).0
                    & (!MASK_FILES[0]).0
            },
            D::E  => {
                self.0.overflowing_shl(1 as u32).0
                    & (!MASK_FILES[0]).0
            },
            D::SE => {
                self.0.overflowing_shr(7 as u32).0
                    & (!MASK_FILES[0]).0
            },
            D::S  => {
                self.0.overflowing_shr(8 as u32).0
            },
            D::SW => {
                self.0.overflowing_shr(9 as u32).0
                    & (!MASK_FILES[7]).0
            },
            D::W  => {
                self.0.overflowing_shr(1 as u32).0
                    & (!MASK_FILES[7]).0
            },
            D::NW => {
                self.0.overflowing_shl(7 as u32).0
                    & (!MASK_FILES[7]).0
            },
        };

        BitBoard(b)
        // unimplemented!()
    }

    #[cfg(feature = "nope")]
    pub fn shift_mult(&self, d: D, n: u64) -> Self {
        unimplemented!()
    }

    // #[cfg(feature = "nope")]
    pub fn shift_mult(&self, d: D, n: u64) -> Self {
        let mut out = *self;
        for _ in 0..n {
            out = out.shift_dir(d);
        }
        out
    }

    // pub fn shift_vec(&self, ds: &[D]) -> Self {
    //     ds.iter()
    //         .fold(*self, |acc, d| acc.shift_dir(*d))
    // }

    // pub fn shift_left(&self, k: u32) -> Self {
    //     BitBoard(self.0.overflowing_shl(k).0 & (!BitBoard::mask_file(0)).0)
    // }

    // pub fn shift_right(&self, k: u32) -> Self {
    //     BitBoard(self.0.overflowing_shr(k).0 & (!BitBoard::mask_file(7)).0)
    // }

}

/// Display
impl BitBoard {
    pub fn print_hex(&self) -> String {
        format!("hex: {:#8x}", self.0)
    }
}

impl Default for BitBoard {
    fn default() -> Self {
        Self::empty()
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

pub mod bits {

    // use super::BitBoard;
    // use std::ops::{BitAnd,BitAndAssign,BitOr,BitOrAssign,BitXor,BitXorAssign,Not};

    // impl BitAnd for BitBoard {
    //     type Output = Self;
    //     fn bitand(self, rhs: Self) -> Self::Output {
    //         Self(self.0 & rhs.0)
    //     }
    // }

    // impl BitAndAssign for BitBoard {
    //     fn bitand_assign(&mut self, rhs: Self) {
    //         *self = Self(self.0 & rhs.0)
    //     }
    // }

    // impl BitOr for BitBoard {
    //     type Output = Self;
    //     fn bitor(self, rhs: Self) -> Self::Output {
    //         Self(self.0 | rhs.0)
    //     }
    // }

    // impl BitOrAssign for BitBoard {
    //     fn bitor_assign(&mut self, rhs: Self) {
    //         *self = Self(self.0 | rhs.0)
    //     }
    // }

    // impl BitXor for BitBoard {
    //     type Output = Self;
    //     fn bitxor(self, rhs: Self) -> Self::Output {
    //         Self(self.0 ^ rhs.0)
    //     }
    // }

    // impl BitXorAssign for BitBoard {
    //     fn bitxor_assign(&mut self, rhs: Self) {
    //         *self = Self(self.0 ^ rhs.0)
    //     }
    // }


    // impl Not for BitBoard {
    //     type Output = Self;
    //     fn not(self) -> Self::Output {
    //         Self(!self.0)
    //     }
    // }

}
