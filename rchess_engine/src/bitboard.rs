
use crate::types::*;
// pub use self::bits::*;

#[derive(Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct BitBoard(pub u64);

impl BitBoard {

    pub fn empty() -> BitBoard {
        BitBoard(0)
    }
    pub fn new(cs: &[Coord]) -> BitBoard {
        let mut b = BitBoard::empty();
        for c in cs.iter() {
            b.flip_mut(*c);
        }
        b
    }

    pub fn single(c: Coord) -> BitBoard {
        let mut b = BitBoard::empty();
        b.flip_mut(c);
        b
    }

    pub fn get(&self, c: Coord) -> bool {
        let p = Self::index_square(c);
        let k = (self.0 >> p) & 1;
        k == 1
    }

    pub fn set_one(&self, c: Coord) -> Self {
        let b = Self::empty().flip(c);
        *self | b
    }

    pub fn set_zero(&self, c: Coord) -> Self {
        let b = Self::empty().flip(c);
        *self & !b
    }

    pub fn flip(&self, c: Coord) -> Self {
        let p = Self::index_square(c);
        let k = 1u64.overflowing_shl(p as u32).0;
        BitBoard(self.0 | k)
    }

    pub fn flip_mut(&mut self, c: Coord) {
        let p = Self::index_square(c);
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

    pub fn mask_rank(r: u32) -> BitBoard {
        assert!(r < 8);
        let k = (!0u8) as u64;
        BitBoard(k.overflowing_shl(r * 8).0)
    }

    pub fn mask_file(f: u32) -> BitBoard {
        assert!(f < 8);
        let k = 0x0101010101010101u64;
        BitBoard(k.overflowing_shl(f).0)
    }

    // pub fn mask_diagonal_ltr()

    pub fn bitscan(&self) -> u32 {
        // Bitscan Forward
        // self.0.leading_zeros()
        self.0.trailing_zeros()
    }

    pub fn bitscan_isolate(&self) -> Self {
        let x = self.bitscan();
        Self::single(x.into())
    }

    pub fn bitscan_reset(&self) -> (Self, u32) {
        let x = self.bitscan();
        // (*self & BitBoard(self.0.overflowing_sub(1).0),x)
        (*self & !Self::single(x.into()),x)
    }

    pub fn bitscan_reset_mut(&mut self) -> u32 {
        let (b,x) = self.bitscan_reset();
        *self = b;
        x
    }

    pub fn bitscan_rev(&self) -> u32 {
        // Bitscan Reverse
        self.0.leading_zeros()
        // self.0.trailing_zeros()
    }

    pub fn bitscan_rev_reset(&self) -> (Self, u32) {
        let x = self.bitscan_rev();
        (*self & !Self::single(x.into()),x)
    }

    pub fn bitscan_rev_reset_mut(&mut self) -> u32 {
        let (b,x) = self.bitscan_rev_reset();
        *self = b;
        x
    }

    pub fn serialize(&self) -> Vec<Coord> {
        let mut b = *self;
        let mut out = vec![];
        let mut x;
        loop {
            x = b.bitscan_reset_mut();
            // out.push(Self::index_bit(x as u64));
            out.push(x.into());
            if b.0 == 0 {
                break;
            }
        }
        out
    }

}

impl BitBoard {
    pub fn iter_bitscan<F>(&self, mut f: F)
    where F: FnMut(u32) {
        let mut b = *self;
        while b.0 != 0 {
            let p = b.bitscan_reset_mut();
            f(p);
        }
    }

    pub fn iter_rev_bitscan<F>(&self, mut f: F)
    where F: FnMut(u32) {
        let mut b = *self;
        while b.0 != 0 {
            let p = b.bitscan_rev_reset_mut();
            f(p);
        }
    }

}

impl BitBoard {

    pub fn index_square(c: Coord) -> u64 {
        // Little Endian Rank File Mapping
        // Least Significant File Mapping
        let p: u64 = c.0 as u64 + 8 * c.1 as u64;
        p
    }

    pub fn index_bit<T: Into<u64> + Copy>(s: T) -> Coord {
        Coord(Self::index_file(s.into()) as u8,Self::index_rank(s.into()) as u8)
    }

    pub fn index_rank(s: u64) -> u64 {
        s >> 3
    }

    pub fn index_file(s: u64) -> u64 {
        s & 7
    }

}

impl BitBoard {

    pub fn flip_diag(&self) -> Self {
        let mut x = self.0;
        let x = x.reverse_bits();
        Self(x)
    }

    pub fn shift(&self, d: D) -> Self {

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
                    & (!BitBoard::mask_file(0)).0
            },
            D::E  => {
                self.0.overflowing_shl(1 as u32).0
                    & (!BitBoard::mask_file(0)).0
            },
            D::SE => {
                self.0.overflowing_shr(7 as u32).0
                    & (!BitBoard::mask_file(0)).0
            },
            D::S  => {
                self.0.overflowing_shr(8 as u32).0
            },
            D::SW => {
                self.0.overflowing_shr(9 as u32).0
                    & (!BitBoard::mask_file(7)).0
            },
            D::W  => {
                self.0.overflowing_shr(1 as u32).0
                    & (!BitBoard::mask_file(7)).0
            },
            D::NW => {
                self.0.overflowing_shl(7 as u32).0
                    & (!BitBoard::mask_file(7)).0
            },
        };

        BitBoard(b)
        // unimplemented!()
    }

    pub fn shift_mult(&self, ds: &[D]) -> Self {
        ds.iter()
            .fold(*self, |acc, d| acc.shift(*d))
    }

    // pub fn shift_left(&self, k: u32) -> Self {
    //     BitBoard(self.0.overflowing_shl(k).0 & (!BitBoard::mask_file(0)).0)
    // }

    // pub fn shift_right(&self, k: u32) -> Self {
    //     BitBoard(self.0.overflowing_shr(k).0 & (!BitBoard::mask_file(7)).0)
    // }

}

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ss: &str = &format!("{:0>64b}", self.0);
        f.write_str(&format!("\nBitBoard:\n"))?;
        for y in 0..8 {
            let (c, rest) = ss.split_at(8);
            let c = c.chars().rev().collect::<String>();
            f.write_str(&format!("{}\n", c))?;
            ss = rest;
        }
        Ok(())
    }
}

pub mod bits {

    use super::BitBoard;
    use std::ops::{BitAnd,BitAndAssign,BitOr,BitOrAssign,BitXor,BitXorAssign,Not};

    impl BitAnd for BitBoard {
        type Output = Self;
        fn bitand(self, rhs: Self) -> Self::Output {
            Self(self.0 & rhs.0)
        }
    }

    impl BitAndAssign for BitBoard {
        fn bitand_assign(&mut self, rhs: Self) {
            *self = Self(self.0 & rhs.0)
        }
    }

    impl BitOr for BitBoard {
        type Output = Self;
        fn bitor(self, rhs: Self) -> Self::Output {
            Self(self.0 | rhs.0)
        }
    }

    impl BitOrAssign for BitBoard {
        fn bitor_assign(&mut self, rhs: Self) {
            *self = Self(self.0 | rhs.0)
        }
    }

    impl BitXor for BitBoard {
        type Output = Self;
        fn bitxor(self, rhs: Self) -> Self::Output {
            Self(self.0 ^ rhs.0)
        }
    }

    impl BitXorAssign for BitBoard {
        fn bitxor_assign(&mut self, rhs: Self) {
            *self = Self(self.0 ^ rhs.0)
        }
    }


    impl Not for BitBoard {
        type Output = Self;
        fn not(self) -> Self::Output {
            Self(!self.0)
        }
    }

}
